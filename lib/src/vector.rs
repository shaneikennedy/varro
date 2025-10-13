use std::path::Path;

use fastembed::TextEmbedding;
use rusqlite::{Connection, ffi::sqlite3_auto_extension};
use sqlite_vec::sqlite3_vec_init;

#[allow(dead_code)]
pub struct VectorStore {
    embedding_model: TextEmbedding,
    db: Connection,
}

impl VectorStore {
    pub fn new(path: &Path) -> Self {
        unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let db = Connection::open(path.join("vectorstore.sqlite")).unwrap();
        let model = TextEmbedding::try_new(Default::default()).unwrap();
        db.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(embedding float[384], document_id TEXT NOT NULL)",
            [],
        )
        .unwrap();

        Self {
            embedding_model: model,
            db,
        }
    }
}
