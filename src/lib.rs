use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Error, Result};
use bincode::config;
use log::{debug, error, info, warn};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

mod compaction;
pub mod document;
mod manifest;
mod segment;
mod tokens;

pub use document::{Document, Field};

use crate::compaction::SegmentCompactor;
use crate::manifest::{Manifest, SegmentId};
use crate::segment::{DocumentSegment, Segment};

/// The model for indexing, querying and retrieveing documents
pub struct Varro {
    /// Where on the filesystem to store files and their indexes
    index_path: PathBuf,

    /// Where the full document objects are actually stored
    documents_path: PathBuf,

    /// Append only in-memory buffer before flushing to disk
    buffer: Mutex<Vec<JoinHandle<DocumentSegment>>>,

    /// Total documents in the index, used for IDF calculations
    total_docs: AtomicUsize,

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
}

impl Varro {
    /// Contruct a new instance of Varro
    pub fn new(path: &Path) -> Result<Varro> {
        let documents_path = path.join("documents");
        match path.exists() {
            true => info!("Index dir exists"),
            false => fs::create_dir(path)?,
        };
        match documents_path.exists() {
            true => info!("Documents subdir dir exists"),
            false => fs::create_dir(documents_path.clone())?,
        };

        // For now we can be dumb and literally just count the files in the document_path
        let total_docs = fs::read_dir(documents_path.clone())?;
        let total_docs = total_docs.count();
        info!("Initializing with {total_docs} docs in the index.");

        // Read manifest file into memory if there is one.
        let contents = fs::read(path.join("manifest.varro"));
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
                }))
            }
        };

        let min_segment_size = Arc::new(Mutex::new(64000000));

        // Setup the segment compactor thread
        let stop = Arc::new(Mutex::new(false));
        let compaction_freq = Arc::new(Mutex::new(Duration::from_secs(2)));
        let segment_compactor = SegmentCompactor::new(
            stop.clone(),
            manifest.clone(),
            path.to_path_buf(),
            min_segment_size.clone(),
            compaction_freq.clone(),
        );
        let segment_compactor = Mutex::new(Some(thread::spawn(move || segment_compactor.run())));

        let varro = Varro {
            index_path: path.to_path_buf(),
            documents_path: documents_path.clone(),
            buffer: Mutex::new(Vec::new()),
            total_docs: AtomicUsize::new(total_docs),
            stop,
            segment_compactor,
            manifest,
            compaction_freq,
            min_segment_size,
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

    /// The total number of docs in the Varro index
    pub fn index_size(&self) -> usize {
        self.total_docs.load(SeqCst)
    }

    /// The current configured path for Varro to store it's index files
    pub fn index_path(&self) -> &Path {
        self.index_path.as_path()
    }

    /// Index a document, this takes a Document, stores it, adds the index to the document buffer, and returns whether it was successfull or not
    pub fn index(&self, doc: Document) -> Result<()> {
        // First things first get this thing written to disk
        self.write_doc(&doc)?;

        // Then add it to the varro buffer to be indexed
        let mut docs = self.buffer.lock().unwrap();
        let handle = thread::spawn(move || DocumentSegment::new(&doc));
        docs.push(handle);

        // TODO automatically flush if the buffer hits a certain size, which is configurable

        Ok(())
    }

    /// Write a Document to the documents_path for durability and retrieval
    fn write_doc(&self, doc: &Document) -> Result<()> {
        let id = doc.id().clone();
        let p = self.documents_path.join(id.clone());
        let config = config::standard();
        let bytes = bincode::encode_to_vec(doc, config)?;
        Ok(fs::write(p, bytes)?)
    }

    /// Text search, given an input string query the index and return a list of Document Ids
    /// and their corresponding TDIDF score (higher is better) that match the search
    pub fn search(&self, query: String) -> impl Iterator<Item = DocumentScore> {
        info!("Searching for {query}");
        let tokens = tokens::tokenize(query.as_str());

        // Get all the segment files and load them into memory, merging them all into a master segment
        let segment_files = &self.manifest.read().unwrap().segments;
        let mut master_segment = Segment::new();
        debug!("Searching through segment files: {:#?}", segment_files);
        for f in segment_files.keys() {
            let segment_file = format!("{f}.seg");
            let segment_path = self.index_path.join(&segment_file);
            let contents = fs::read(&segment_path);
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
                None => warn!("Unable to read segment file {:#?}", segment_path),
            }
        }

        // Collect any doc where any token in the query exist and caluclate the tfidf
        let mut matching_docs: HashSet<DocumentScore> = HashSet::new();
        debug!("Total docs in index: {}", self.total_docs.load(SeqCst));
        for token in tokens {
            if let Some(tfdf) = master_segment.term_index.get(&token) {
                let docs_with_term = tfdf.term_freq.len();
                debug!("Total docs for term {token}: {docs_with_term}");
                tfdf.term_freq.iter().for_each(|(doc_id, tf)| {
                    let idf =
                        (self.total_docs.load(SeqCst) as f64 / docs_with_term as f64).log(10.0);
                    let tfidf = tf * idf;
                    matching_docs.insert(DocumentScore {
                        document_id: doc_id.to_string(),
                        score: tfidf,
                    });
                });
            }
        }
        matching_docs.into_iter()
    }

    /// Retrive a document by it's Document.id, returns an Option type wrapping a Document
    pub fn retrieve(&self, id: String) -> Option<Document> {
        let file = fs::read(self.documents_path.join(id.clone()));
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

    /// Flush the indexes to disk, this needs to happen before a document is searchable
    pub fn flush(&self) -> Result<()> {
        let mut segment = Segment::new();
        let mut docs = self.buffer.lock().unwrap();
        for doc_seg in docs.drain(0..) {
            let doc_seg = doc_seg.join();
            if doc_seg.is_err() {
                error!("Problem indexing document ????????");
                return Err(Error::msg("problem indexing this document"));
            }
            let doc_seg = doc_seg.unwrap();
            segment.add_docucment_segment(&doc_seg);

            // TODO: this wraps around on overflow
            self.total_docs.fetch_add(1, SeqCst);
        }
        let (segment_id, segment_size) = self.write_segment(&segment)?;

        // Update the manifest file
        debug!("Start update manifest file");
        let mut manifest_guard = self.manifest.write().unwrap();
        manifest_guard
            .segments
            .insert(segment_id.clone(), segment_size);
        debug!(
            "Manifest object now contains segments: {:#?}",
            manifest_guard.segments
        );
        let config = config::standard();
        drop(manifest_guard);
        let manifest_guard = self.manifest.read().unwrap();
        let bytes = bincode::encode_to_vec(&*manifest_guard, config)?;
        fs::write(self.index_path().join("manifest.varro"), bytes)?;
        Ok(())
    }

    fn write_segment(&self, seg: &Segment) -> Result<(SegmentId, usize)> {
        let config = config::standard();
        let bytes = bincode::encode_to_vec(seg, config)?;
        let segment_id = Uuid::new_v4().to_string();
        fs::write(self.index_path().join(segment_id.clone() + ".seg"), &bytes)?;
        Ok((segment_id, bytes.len()))
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

pub struct DocumentScore {
    pub document_id: String,
    pub score: f64,
}

impl PartialEq for DocumentScore {
    fn eq(&self, other: &Self) -> bool {
        self.document_id == other.document_id
    }
}

impl Eq for DocumentScore {}

impl Hash for DocumentScore {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.document_id.hash(state);
    }
}
