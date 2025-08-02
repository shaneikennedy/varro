use std::collections::HashSet;
use std::fs::{create_dir, read, write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use bincode::{config, Decode, Encode};
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
    buffer: Mutex<Vec<JoinHandle<Document>>>,
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
        let mut docs = self.buffer.lock().unwrap();
        let handle = thread::spawn(|| doc);
        docs.push(handle);
        Ok(())
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

    /// Flush the documents and indexs to disk, this needs to happen before a document is searchable
    pub fn flush(&self) -> Result<()> {
        let mut docs = self.buffer.lock().unwrap();
        for doc in docs.drain(0..) {
            let doc = doc.join();
            let doc = match doc {
                Ok(d) => d,
                Err(_) => panic!("Problem indexing document ????????"),
            };
            let id = doc.id.clone();
            println!("about to save file: {}", id);
            let p = self.documents_path.join(id.clone());
            println!("about to save file to dir {}", id);
            let config = config::standard();
            let bytes = bincode::encode_to_vec(doc, config)?;
            write(p, bytes)?;
            println!("Successfully indexed document: {}", id.clone());
        }
        Ok(())
    }
}
