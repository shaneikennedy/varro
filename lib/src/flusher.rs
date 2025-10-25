use anyhow::{Error, Result};
use bincode::config;
use log::{debug, error};
use std::sync::{Arc, RwLock};

use crate::{
    filesystem::FileSystem,
    manifest::Manifest,
    segment::{DocumentSegment, Segment},
    vector::VectorStore,
};

pub(crate) struct Flusher {
    manifest: Arc<RwLock<Manifest>>,
    filesystem: Arc<Box<dyn FileSystem>>,
    vector_store: Arc<VectorStore>,
}

impl Flusher {
    pub(crate) fn new(
        manifest: Arc<RwLock<Manifest>>,
        filesystem: Arc<Box<dyn FileSystem>>,
        vector_store: Arc<VectorStore>,
    ) -> Self {
        Self {
            manifest,
            filesystem,
            vector_store,
        }
    }

    pub(crate) fn flush_new_docs(
        &self,
        events: impl Iterator<Item = std::thread::JoinHandle<DocumentSegment>>,
    ) -> Result<()> {
        let mut segment = Segment::new();
        for doc_seg in events {
            let doc_seg = doc_seg.join();
            if doc_seg.is_err() {
                error!("Problem indexing document ????????");
                return Err(Error::msg("problem indexing this document"));
            }
            let doc_seg = doc_seg.unwrap();
            segment.add_docucment_segment(&doc_seg);
            self.vector_store.insert_document(&doc_seg.document())?;
            self.manifest.write().unwrap().total_docs += 1;
        }
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
            + segment.token_count() as f64)
            / (manifest_guard.total_docs + segment.documents().len()) as f64;
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
