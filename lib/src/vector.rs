use log::debug;
use std::path::Path;
use std::{collections::HashMap, sync::Mutex};
use zerocopy::IntoBytes;

use anyhow::{Context, Result};
use fastembed::TextEmbedding;
use rusqlite::{Connection, ffi::sqlite3_auto_extension};
use sqlite_vec::sqlite3_vec_init;

use crate::{Document, Score};

#[allow(dead_code)]
pub struct VectorStore {
    embedding_model: Mutex<TextEmbedding>,
    db: Mutex<Connection>,
}

// add a default and make new() accept a model and db for easier testing
impl VectorStore {
    pub fn new(path: &Path) -> Self {
        unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let db = Connection::open(path.join("vectorstore.sqlite")).unwrap();
        let model = TextEmbedding::try_new(Default::default()).unwrap();
        db.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(embedding float[384], document_id TEXT NOT NULL, field TEXT NOT NULL)",
            [],
        )
        .unwrap();

        Self {
            embedding_model: Mutex::new(model),
            db: Mutex::new(db),
        }
    }

    #[allow(dead_code)]
    pub fn search(&self, query: &str) -> Result<HashMap<Document, Score>> {
        let search = vec![query];
        let mut model = self.embedding_model.lock().unwrap();
        let embedding_query = model.embed(search.clone(), None).unwrap();
        let embedding_query = embedding_query.first().unwrap();

        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            r"
          SELECT
            document_id,
            distance
          FROM vec_items
          WHERE embedding MATCH ?1
          ORDER BY distance
          LIMIT 100
        ",
        )?;
        let result: Result<HashMap<Document, Score>, rusqlite::Error> = stmt
            .query_map([embedding_query.as_bytes()], |r| {
                Ok((Document::new(r.get(0)?), r.get(1)?))
            })?
            .collect();
        result.context("Error executing query")
    }

    #[allow(dead_code)]
    pub fn search_with_field(
        &self,
        query: &str,
        field_name: &str,
    ) -> Result<HashMap<Document, Score>> {
        let search = vec![query];
        let mut model = self.embedding_model.lock().unwrap();
        let embedding_query = model.embed(search.clone(), None).unwrap();
        let embedding_query = embedding_query.first().unwrap();

        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            r"
          SELECT
            document_id,
            distance
          FROM vec_items
          WHERE embedding MATCH ?1 AND field = ?2
          ORDER BY distance
          LIMIT 100
        ",
        )?;
        let result: Result<HashMap<Document, Score>, rusqlite::Error> = stmt
            .query_map([embedding_query.as_bytes(), field_name.as_bytes()], |r| {
                Ok((Document::new(r.get(0)?), r.get(1)?))
            })?
            .collect();
        result.context("Error executing query")
    }

    #[allow(dead_code)]
    pub fn insert_document(&self, doc: &Document) -> Result<()> {
        let mut model = self.embedding_model.lock().unwrap();
        for field in doc.fields() {
            let embeddings = model.embed(vec![field.contents()], None).unwrap();
            debug!(
                "Generated embeddings: {:#?} for field: {}",
                embeddings,
                field.name()
            );
            let db = self.db.lock().unwrap();
            let mut stmt =
                db.prepare("INSERT INTO vec_items(document_id, field, embedding) VALUES (?,?,?)")?;
            for item in embeddings {
                stmt.execute(rusqlite::params![doc.id(), field.name(), item.as_bytes()])?;
            }
        }
        Ok(())
    }
}
