use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use bincode::{Decode, Encode};
use uuid::Uuid;

/// The model representing a field in a document
#[derive(Eq, Encode, Decode, Clone)]
pub struct Field {
    name: String,
    contents: String,
    index: bool,
}

impl Hash for Field {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Field {
    pub fn new(name: &str, contents: &str, index: bool) -> Self {
        Self {
            name: name.to_string(),
            contents: contents.to_string(),
            index,
        }
    }

    pub fn indexed(&self) -> bool {
        self.index
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn contents(&self) -> String {
        self.contents.clone()
    }
}

/// The model representing a document that has been indexed by Varro
#[derive(PartialEq, Eq, Encode, Decode, Clone)]
pub struct Document {
    id: String,

    /// The fields map of the document e.g "name": "Intro to git", "content": "1000 words...", and whether or not to store and index that field
    fields: HashSet<Field>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new(Uuid::new_v4().to_string())
    }
}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Document {
    pub fn new(id: String) -> Document {
        Document {
            id,
            fields: HashSet::new(),
        }
    }

    pub fn add_field(&mut self, name: String, contents: String, index: bool) {
        let new_field = Field {
            name,
            contents,
            index,
        };
        if self.fields.contains(&new_field) {
            self.fields.remove(&new_field);
        }
        self.fields.insert(new_field);
    }

    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    pub fn get_field(&self, name: String) -> Option<&Field> {
        self.fields.iter().find(|&f| f.name == name)
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Return the number of bytes allocated by a document
    pub fn size(&self) -> usize {
        let mut size = self.id.len();
        for field in self.fields.iter() {
            size += field.name.len();
            size += field.contents.len();
            size += size_of_val(&field.index)
        }
        size
    }
}
