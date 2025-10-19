use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Error, Result};
use bincode::config;
use log::{debug, error, info, warn};

mod compaction;
pub mod document;
mod filesystem;
mod manifest;
mod ranking;
mod search;
mod segment;
mod tokens;
mod vector;
mod vql;

pub use document::{Document, Field};
pub use ranking::RankingType;

use crate::compaction::SegmentCompactor;
#[cfg(feature = "s3")]
use crate::filesystem::S3FileSystem;
use crate::filesystem::{FileSystem, LocalFileSystem, TempFileSystem};
use crate::manifest::Manifest;
use crate::search::Searcher;
use crate::segment::{DocumentSegment, Segment};
use crate::vector::VectorStore;

/// The model for indexing, querying and retrieveing documents
pub struct Varro {
    /// Append only in-memory buffer before flushing to disk
    buffer: Mutex<Vec<JoinHandle<DocumentSegment>>>,

    /// Internal counter for how big the buffer is for flushing purposes.
    buffer_size: AtomicUsize,

    /// Maximum buffer size, when the document buffer reaches this size in bytes
    /// Varro will automatically flush the buffer to disk and these documents
    /// will become searchable. Default is 50MB or 50_000_000.
    max_buffer_size: Mutex<usize>,

    /// Segment compactor is the handle to the background thread that's
    /// compacting segments when they get too big
    segment_compactor: Mutex<Option<JoinHandle<Result<()>>>>,

    /// Stop signal is how we kill the segment_compactor for Drop
    stop: Arc<Mutex<bool>>,

    /// Manifest file representation
    manifest: Arc<RwLock<Manifest>>,

    /// How often to run segment compaction, defaults to 2 seconds.
    compaction_freq: Arc<Mutex<Duration>>,

    /// Minimum segment size is used to determine when a file should be compacted.
    /// Segments are read into memory when searching, using lots of small segment files is worse
    /// for performance but better when memory is constrained. Larger segments are better for performance
    /// but cause spikes in memory on searches. Default is 64MB.
    min_segment_size: Arc<Mutex<usize>>,

    /// The filesystem abstraction to accomodate different file stores. Default is LocalFileSystem
    filesystem: Arc<Box<dyn FileSystem>>,

    /// The vector database to use for "similarity" queries
    #[allow(dead_code)]
    vector_store: Arc<VectorStore>,

    /// Internal search logic
    searcher: Searcher,
}

pub enum FileSystemType {
    Local,
    Temp,
    #[cfg(feature = "s3")]
    S3,
}

impl Varro {
    /// Contruct a new instance of Varro
    pub fn new(path: &Path, file_system_type: FileSystemType) -> Result<Varro> {
        let filesystem: Box<dyn FileSystem> = match file_system_type {
            FileSystemType::Local => Box::new(LocalFileSystem::new(path)?),
            FileSystemType::Temp => Box::new(TempFileSystem::new(Some(path))?),
            #[cfg(feature = "s3")]
            FileSystemType::S3 => Box::new(S3FileSystem::new(path)?),
        };
        let filesystem: Arc<Box<dyn FileSystem>> = Arc::new(filesystem);

        // Read manifest file into memory if there is one.
        let contents = filesystem.read_from_manifest();
        let manifest = match contents {
            Ok(c) => {
                let config = config::standard();
                let (decoded, _): (Manifest, usize) =
                    bincode::decode_from_slice(&c[..], config).unwrap();
                debug!("Manifest found on init: {:#?}", decoded);
                Arc::new(RwLock::new(decoded))
            }
            Err(_) => {
                warn!("No manifest file found, starting a new one.");
                Arc::new(RwLock::new(Manifest {
                    segments: HashMap::new(),
                    total_docs: 0,
                    average_document_length: 0.0,
                }))
            }
        };
        let vector_store = Arc::new(VectorStore::new(path));

        let searcher = Searcher::new(filesystem.clone(), manifest.clone(), vector_store.clone());

        let min_segment_size = Arc::new(Mutex::new(64000000));

        // Setup the segment compactor thread
        let stop = Arc::new(Mutex::new(false));
        let compaction_freq = Arc::new(Mutex::new(Duration::from_secs(2)));
        let segment_compactor = SegmentCompactor::new(
            stop.clone(),
            manifest.clone(),
            min_segment_size.clone(),
            compaction_freq.clone(),
            filesystem.clone(),
        );
        let segment_compactor = Mutex::new(Some(thread::spawn(move || segment_compactor.run())));

        let varro = Varro {
            buffer: Mutex::new(Vec::new()),
            buffer_size: AtomicUsize::new(0),
            max_buffer_size: Mutex::new(50_000_000),
            stop,
            segment_compactor,
            manifest,
            compaction_freq,
            min_segment_size,
            filesystem,
            searcher,
            vector_store,
        };
        Ok(varro)
    }

    /// Updates the Varro instance with a new `min_segment_size`
    pub fn with_min_segment_size(self, size: usize) -> Self {
        *self.min_segment_size.lock().unwrap() = size;
        self
    }

    /// Update the Varro instance with a new `compaction_frequency`
    pub fn with_compaction_frequency(self, duration: Duration) -> Self {
        *self.compaction_freq.lock().unwrap() = duration;
        self
    }

    /// Update the Varro instance with a new `max_buffer_size` to control when
    /// Varro flushes automatically
    pub fn with_max_buffer_size(self, size: usize) -> Self {
        *self.max_buffer_size.lock().unwrap() = size;
        self
    }

    /// The total number of docs in the Varro index
    pub fn index_size(&self) -> usize {
        self.manifest.read().unwrap().total_docs
    }

    /// Index a document, this takes a Document, stores it, adds the index to the document buffer, and returns whether it was successfull or not
    pub fn index(&self, doc: Document) -> Result<()> {
        // First things first get this thing written to disk
        self.write_doc(&doc)?;
        self.buffer_size.fetch_add(doc.size(), Ordering::SeqCst);

        // Then add it to the varro buffer to be indexed
        let mut docs = self.buffer.lock().unwrap();
        let handle = thread::spawn(move || DocumentSegment::new(&doc));
        docs.push(handle);

        // Force a flush when the buffer size gets to the set max limit
        if self.buffer_size.load(Ordering::SeqCst) > *self.max_buffer_size.lock().unwrap() {
            self.flush()?;
        }
        Ok(())
    }

    /// Write a Document to the documents_path for durability and retrieval
    fn write_doc(&self, doc: &Document) -> Result<()> {
        let id = doc.id().clone();
        let config = config::standard();
        let bytes = bincode::encode_to_vec(doc, config)?;
        self.filesystem
            .write_to_document(Path::new(&id.clone()), bytes)
    }

    /// Text search, given an input string query the index and return a list of Document Ids
    /// and their corresponding TDIDF score (higher is better) that match the search
    pub fn search(
        &self,
        query: String,
        options: Option<SearchOptions>,
    ) -> impl Iterator<Item = (Document, Score)> {
        info!("Search query: {query}");

        // Get all the segment files and load them into memory, merging them all into a master segment
        let manifest_guard = self.manifest.read().unwrap();
        let mut master_segment = Segment::new();
        debug!(
            "Searching through segment files: {:#?}",
            manifest_guard.segments.keys()
        );
        for f in manifest_guard.segments.keys() {
            let segment_file = format!("{f}.seg");
            let contents = self.filesystem.read_from_index(Path::new(&segment_file));
            let segment = match contents {
                Ok(c) => {
                    let config = config::standard();
                    let (decoded, _): (Segment, usize) =
                        bincode::decode_from_slice(&c[..], config).unwrap();
                    Some(decoded)
                }
                Err(_) => None,
            };

            // Merge the segments
            match segment {
                Some(s) => {
                    master_segment.add_segment(s);
                }
                None => warn!("Unable to read segment file {:#?}", segment_file),
            }
        }
        drop(manifest_guard);

        let opts = options.unwrap_or_default();
        // Collect a map of terms to docs for which the term appears, and it's tfidf score
        let mut matching_docs = self.searcher.search(&query, &master_segment);
        if opts.include_documents {
            matching_docs = matching_docs
                .iter()
                .map(|(d, score)| (self.get_doc_by_id(d.id()).unwrap(), *score))
                .collect::<HashMap<Document, Score>>();
        }
        matching_docs.into_iter()
    }

    fn get_doc_by_id(&self, id: String) -> Option<Document> {
        let file = self.filesystem.read_from_documents(Path::new(&id.clone()));
        match file {
            Ok(f) => {
                let config = config::standard();
                let (decoded, _): (Document, usize) =
                    bincode::decode_from_slice(&f[..], config).unwrap();
                Some(decoded)
            }
            Err(_) => None,
        }
    }

    /// Retrive a document by it's Document.id, returns an Option type wrapping a Document
    pub fn retrieve(&self, id: String) -> Option<Document> {
        self.get_doc_by_id(id)
    }

    /// Flush the indexes to disk, this needs to happen before a document is searchable
    pub fn flush(&self) -> Result<()> {
        let mut segment = Segment::new();
        let mut docs = self.buffer.lock().unwrap();
        let mut total_tokens_for_flush = 0;
        let total_docs_for_flush = docs.len();
        for doc_seg in docs.drain(0..) {
            let doc_seg = doc_seg.join();
            if doc_seg.is_err() {
                error!("Problem indexing document ????????");
                return Err(Error::msg("problem indexing this document"));
            }
            let doc_seg = doc_seg.unwrap();
            segment.add_docucment_segment(&doc_seg);
            self.vector_store.insert_document(&doc_seg.document())?;

            // Record the total number of tokens for this doc
            total_tokens_for_flush += doc_seg.document_length();
            self.manifest.write().unwrap().total_docs += 1;
        }
        // Reset the buffer size
        self.buffer_size.swap(0, Ordering::SeqCst);

        debug!("Writting new segmenet to disk");
        let (segment_id, segment_size) = segment.write_to_fs(&**self.filesystem)?;

        // Update the manifest file
        debug!("Start update manifest file");
        let mut manifest_guard = self.manifest.write().unwrap();
        manifest_guard
            .segments
            .insert(segment_id.clone(), segment_size);
        manifest_guard.average_document_length = (manifest_guard.total_docs as f64
            * manifest_guard.average_document_length
            + total_tokens_for_flush as f64)
            / (manifest_guard.total_docs + total_docs_for_flush) as f64;
        debug!(
            "Manifest object now contains segments: {:#?}",
            manifest_guard.segments
        );
        let config = config::standard();
        drop(manifest_guard);
        let manifest_guard = self.manifest.read().unwrap();
        let bytes = bincode::encode_to_vec(&*manifest_guard, config)?;
        self.filesystem.write_to_manifest(bytes)
    }
}

impl Drop for Varro {
    fn drop(&mut self) {
        *self.stop.lock().unwrap() = true;
        if let Some(h) = self.segment_compactor.lock().unwrap().take() {
            match h.join() {
                Ok(_) => debug!("Successfully shut down the compactor thread."),
                Err(_) => error!("Problem shutting down the compactor thread."),
            }
        };
    }
}

pub type Score = f64;

#[derive(Clone)]
pub enum SearchOperator {
    OR,
    AND,
}

#[derive(Clone)]
pub struct SearchOptions {
    /// Whether or not to return the full document object in the search response.
    /// By default only the Document ID is returned to be used to fetch at a later time,
    /// that is, default = false.
    include_documents: bool,

    /// How to treat multi-token search queries. By default Varro uses OR when matching
    /// and scoring documents. For example, varro.search("git docker", ...) will search
    /// for documents containing "git" _or_ "docker", while scoring documents with _both_ terms
    /// higher. When search_operator is set to AND, varro.search("git docker") will only return
    /// documents with both terms appearing in the document. Default is SearchOperator::OR.
    search_operator: SearchOperator,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions::new()
    }
}

impl SearchOptions {
    pub fn new() -> Self {
        SearchOptions {
            include_documents: false,
            search_operator: SearchOperator::OR,
        }
    }

    pub fn with_include_documents(&mut self, include_documents: bool) -> Self {
        self.include_documents = include_documents;
        self.clone()
    }

    pub fn include_documents(&self) -> bool {
        self.include_documents
    }

    pub fn with_search_operator(&mut self, operator: SearchOperator) -> Self {
        self.search_operator = operator;
        self.clone()
    }

    pub fn search_operator(&self) -> SearchOperator {
        self.search_operator.clone()
    }
}

#[cfg(test)]
mod varro_tests {
    use super::*;

    #[test]
    fn test_new() {
        Varro::new(Path::new(""), FileSystemType::Temp).unwrap();
    }

    #[test]
    fn test_index() -> Result<()> {
        let index = Varro::new(Path::new(""), FileSystemType::Temp).unwrap();
        let mut doc = Document::default();
        doc.add_field("name".into(), "varro testing".into(), false);
        index.index(doc)?;
        Ok(())
    }

    #[test]
    fn test_flush() -> Result<()> {
        let index = Varro::new(Path::new(""), FileSystemType::Temp).unwrap();
        let mut doc = Document::default();
        doc.add_field("name".into(), "varro testing".into(), false);
        index.index(doc.clone()).unwrap();
        index.flush()?;
        Ok(())
    }

    #[test]
    fn test_search() -> Result<()> {
        let index = Varro::new(Path::new(""), FileSystemType::Temp).unwrap();
        let mut doc = Document::default();
        doc.add_field("name".into(), "varro testing".into(), true);
        index.index(doc.clone()).unwrap();
        index.flush()?;

        let results = index.search("varro".into(), None);
        let results: Vec<(Document, Score)> = results.collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results.first().unwrap().0.id(), doc.id());
        Ok(())
    }
}
