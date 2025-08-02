use std::collections::{HashMap, HashSet};
use std::fs::{create_dir, read, write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use bincode::{Decode, Encode, config};
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
}

impl Varro {
    /// Contruct a new instance of Varro
    pub fn new(path: PathBuf) -> Result<Varro> {
        let documents_path = path.join("documents");
        let varro = Varro {
            index_path: path.clone(),
            documents_path: documents_path.clone(),
            buffer: Mutex::new(Vec::new()),
        };
        match path.exists() {
            true => println!("Index dir exists"),
            false => create_dir(path)?,
        };
        match documents_path.exists() {
            true => println!("Documents subdir dir exists"),
            false => create_dir(documents_path)?,
        };
        Ok(varro)
    }

    /// Index a document, this takes a Document, stores it, adds the index to the document buffer, and returns whether it was successfull or not
    pub fn index(&self, doc: Document) -> Result<()> {
        // First things first get this thing written to disk
        self.write_doc(&doc)?;

        // Then add it to the varro buffer to be indexed
        let mut docs = self.buffer.lock().unwrap();
        let handle = thread::spawn(move || tokenize(&doc));
        docs.push(handle);
        Ok(())
    }

    /// Write a Document to the documents_path for durability and retrieval
    fn write_doc(&self, doc: &Document) -> Result<()> {
        let id = doc.id.clone();
        println!("about to save file: {}", id);
        let p = self.documents_path.join(id.clone());
        println!("about to save file to dir {}", id);
        let config = config::standard();
        let bytes = bincode::encode_to_vec(doc, config)?;
        Ok(write(p, bytes)?)
    }

    /// Text search, given an input string query the index and return a list of Documents that match the search
    pub fn search(&self, query: String) -> Vec<Document> {
        println!("Searching for {}", query);
        Vec::new()
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
    /// How will this work, the varro.index method will spawn a thread that does tokenize(doc) and retruns a segment. Then during flush, we take all the DocumentSegment objects in the buffer, reduce over them to generate a single Segment object that has a term list, with each term mapping to a TF list (doc_id + frequency) and a DF total number of docs with this term. Flush will write the aggregated Segment to disk
    pub fn flush(&self) -> Result<()> {
        // TODO at this poiint we have a list of "indexing jobs". We need to acquire the lock,
        // wait for any jobs that haven't finished, and reduce all of the DocumentSegments into
        // a single SegmentInfo
        let mut docs = self.buffer.lock().unwrap();
        for doc in docs.drain(0..) {
            let doc = doc.join();
            match doc {
                Ok(d) => println!("Received DocumentSegment for document: {}", d.document_id),
                Err(_) => panic!("Problem indexing document ????????"),
            };
        }

        // TODO write the segment info to disk
        println!("Successfully wrote segment file ");
        Ok(())
    }
}

struct DocumentSegment {
    document_id: String,

    #[allow(dead_code)]
    terms: HashMap<String, i32>,
}

impl DocumentSegment {
    pub fn new(doc: &Document) -> Self {
        DocumentSegment {
            document_id: doc.id(),
            terms: HashMap::new(),
        }
    }
}

fn tokenize(doc: &Document) -> DocumentSegment {
    DocumentSegment::new(doc)
}

#[cfg(test)]
mod tokenize_tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let doc = Document::new();
        let doc_seg = tokenize(&doc);
        assert_eq!(doc.id(), doc_seg.document_id);
    }
}
