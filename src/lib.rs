use std::collections::{HashMap, HashSet};
use std::fs::{create_dir, read, read_dir, write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::{self, JoinHandle};

use anyhow::{Error, Result};
use bincode::{Decode, Encode, config};
use log::{error, info, warn};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// The model representing a field in a document
#[derive(PartialEq, Eq, Hash, Encode, Decode)]
pub struct Field {
    name: String,
    contents: String,
    index: bool,
}

impl Field {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn contents(&self) -> String {
        self.contents.clone()
    }
}

/// The model representing a document that has been indexed by Varro
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct Document {
    id: String,

    /// The fields map of the document e.g "name": "Intro to git", "content": "1000 words...", and whether or not to store and index that field
    fields: HashSet<Field>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Document {
    pub fn new() -> Document {
        Document {
            id: Uuid::new_v4().to_string(),
            fields: HashSet::new(),
        }
    }

    pub fn add_field(&mut self, name: String, contents: String, index: bool) {
        self.fields.insert(Field {
            name,
            contents,
            index,
        });
    }

    pub fn get_field(&self, name: String) -> Option<&Field> {
        self.fields.iter().find(|&f| f.name == name)
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}

/// The model for indexing, querying and retrieveing documents
pub struct Varro {
    /// Where on the filesystem to store files and their indexes
    #[allow(dead_code)]
    index_path: PathBuf,

    /// Where the full document objects are actually stored
    documents_path: PathBuf,

    /// Append only in-memory buffer before flushing to disk
    buffer: Mutex<Vec<JoinHandle<DocumentSegment>>>,

    /// Total documents in the index, used for IDF calculations
    total_docs: AtomicUsize,
}

impl Varro {
    /// Contruct a new instance of Varro
    pub fn new(path: &Path) -> Result<Varro> {
        let documents_path = path.join("documents");
        // TODO actually count the docs in the index
        let docs_in_index: Vec<String> = Vec::new();
        let total_docs = docs_in_index.len();

        let varro = Varro {
            index_path: path.to_path_buf(),
            documents_path: documents_path.clone(),
            buffer: Mutex::new(Vec::new()),
            total_docs: AtomicUsize::new(total_docs),
        };
        match path.exists() {
            true => info!("Index dir exists"),
            false => create_dir(path)?,
        };
        match documents_path.exists() {
            true => info!("Documents subdir dir exists"),
            false => create_dir(documents_path)?,
        };
        Ok(varro)
    }

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
        Ok(())
    }

    /// Write a Document to the documents_path for durability and retrieval
    fn write_doc(&self, doc: &Document) -> Result<()> {
        let id = doc.id.clone();
        let p = self.documents_path.join(id.clone());
        let config = config::standard();
        let bytes = bincode::encode_to_vec(doc, config)?;
        Ok(write(p, bytes)?)
    }

    /// Text search, given an input string query the index and return a list of Document Ids
    /// and their corresponding TDIDF score (higher is better) that match the search
    pub fn search(&self, query: String) -> impl Iterator<Item = DocumentScore> {
        info!("Searching for {query}");
        let tokens = tokenize(query.as_str());

        // Get all the segment files and load them into memory, merging them all into a master segment
        let segment_files = read_dir(self.index_path.clone())
            .unwrap()
            .filter_map(|f| f.ok())
            .filter(|d| d.file_name().into_string().unwrap().contains(".seg"));
        let mut master_segment = Segment::new();
        for f in segment_files {
            if f.file_type().unwrap().is_file() {
                let contents = read(f.path());
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
                        for (term, tfdf) in s.term_index {
                            master_segment
                                .term_index
                                .entry(term)
                                .and_modify(|t| {
                                    t.doc_freq += tfdf.doc_freq;
                                    t.term_freq.extend(tfdf.term_freq.clone());
                                })
                                .or_insert(tfdf);
                        }
                    }
                    None => warn!("Unable to read segment file {:#?}", f.path()),
                }
            }
        }

        // Collect any doc where any token in the query exist, of tfidf/scoring yet

        let mut matching_docs: HashSet<DocumentScore> = HashSet::new();
        for token in tokens {
            if let Some(tfdf) = master_segment.term_index.get(&token) {
                tfdf.term_freq.iter().for_each(|(doc_id, tf)| {
                    // TODO a real IDF
                    let tfidf = tf * 0.5;
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
        let file = read(self.documents_path.join(id.clone()));
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
        self.write_segment(&segment)
    }

    fn write_segment(&self, seg: &Segment) -> Result<()> {
        let config = config::standard();
        let bytes = bincode::encode_to_vec(seg, config)?;
        Ok(write(
            self.index_path().join(Uuid::new_v4().to_string() + ".seg"),
            bytes,
        )?)
    }
}

pub struct DocumentScore {
    pub document_id: String,
    #[allow(dead_code)]
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

/// A Segment is just a map of terms to TFDFs for a given "flush".
#[derive(Encode, Decode, Debug)]
struct Segment {
    term_index: HashMap<String, Tfdf>,
}

impl Segment {
    fn new() -> Self {
        Self {
            term_index: HashMap::new(),
        }
    }

    // For a segment, update the existing term_index with all
    // the terms and corresponding TFs from DocumentSegment
    // if the term doesn't already exist in the term index, insert a new one
    fn add_docucment_segment(&mut self, seg: &DocumentSegment) {
        for (term, _) in seg.terms.iter() {
            if self.term_index.contains_key(term) {
                self.term_index
                    .entry(term.to_string())
                    .and_modify(|t| t.add_for_doc(seg));
            } else {
                let mut tfdf = Tfdf::new(term);
                tfdf.add_for_doc(seg);
                self.term_index.insert(term.to_string(), tfdf);
            }
        }
    }
}

/// A TFDF is holds the info for which documents (id, the String in the term_freq map) have a given term and it's count (the i32 in the term_freq map)
/// and the total number of documents that the term appears in
#[derive(Encode, Decode, Debug)]
struct Tfdf {
    term: String,

    // Each tuple is a document_id, and the normalized fresquency
    // for the term in this doc, that is, # occurances / total words in the doc
    term_freq: Vec<(String, f64)>,
    doc_freq: i32,
}

impl Tfdf {
    pub fn new(term: &str) -> Self {
        Self {
            term: term.into(),
            term_freq: Vec::new(),
            doc_freq: 0,
        }
    }

    pub fn add_for_doc(&mut self, doc_seg: &DocumentSegment) {
        self.term_freq.push((
            doc_seg.document_id(),
            *doc_seg.terms.get(&self.term).unwrap() as f64 / doc_seg.document_length as f64, // Normalize the TF by the document length
        ));
        self.doc_freq += 1;
    }
}

#[derive(Debug)]
struct DocumentSegment {
    #[allow(dead_code)]
    document_id: String,
    // Total number of words in the doc
    document_length: i32,
    terms: HashMap<String, i32>,
}

impl DocumentSegment {
    pub fn new(doc: &Document) -> Self {
        let mut doc_seg = DocumentSegment {
            document_id: doc.id(),
            document_length: 0,
            terms: HashMap::new(),
        };
        let mut word_count = 0;
        for field in doc.fields.iter() {
            let content = tokenize(&field.contents);
            content.for_each(|w| {
                word_count += 1;
                doc_seg.terms.entry(w).and_modify(|v| *v += 1).or_insert(1);
            });
        }
        doc_seg.document_length = word_count;
        doc_seg
    }

    pub fn document_id(&self) -> String {
        self.document_id.clone()
    }
}

// TODO consider phf crate for O(1) lookups if this grows or sucks
const STOP_WORDS: [&str; 10] = ["the", "and", "is", "in", "at", "of", "to", "a", "an", "for"];
fn tokenize(contents: &str) -> impl Iterator<Item = String> {
    contents.split_whitespace().filter_map(|w| {
        if !STOP_WORDS.contains(&w.to_lowercase().as_str()) {
            Some(w.to_lowercase())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod document_segment_tests {
    use super::*;

    #[test]
    fn test_document_segment() {
        let mut doc = Document::new();
        doc.add_field("name".into(), "wow such nice test".into(), true);
        doc.add_field("body".into(), "wow such nice test again".into(), true);
        let doc_seg = DocumentSegment::new(&doc);
        assert_eq!(doc.id(), doc_seg.document_id);
        assert_eq!(doc_seg.terms.get("wow"), Some(&2));
        assert_eq!(doc_seg.terms.get("such"), Some(&2));
        assert_eq!(doc_seg.terms.get("nice"), Some(&2));
        assert_eq!(doc_seg.terms.get("test"), Some(&2));
        assert_eq!(doc_seg.terms.get("again"), Some(&1));
    }
}

#[cfg(test)]
mod tokenize_tests {
    use super::*;

    #[test]
    fn test_tokenize_lower_cases() {
        let contents = "smAll sIlly kitTy Cat".to_string();
        let tokens: Vec<String> = tokenize(&contents).collect();
        assert_eq!(vec!["small", "silly", "kitty", "cat"], tokens);
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let contents = "For once and for all".to_string();
        let tokens: Vec<String> = tokenize(&contents).collect();
        assert_eq!(vec!["once", "all"], tokens);
    }
}
