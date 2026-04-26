use anyhow::{Error, Result};
use bincode::config;
use log::{debug, error};
use std::{
    collections::HashSet,
    path::Path,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{
    Document,
    filesystem::FileSystem,
    manifest::Manifest,
    options,
    segment::{DocumentSegment, Segment},
    vector::VectorStore,
};

pub(crate) enum FlushEventType {
    Delete,
    Update,
    Insert,
}

enum FlushEvent {
    Delete(DeleteEvent),
    Update(UpdateEvent),
    Insert(InsertEvent),
}

pub(crate) struct DeleteEvent {
    doc_seg_to_delete: DocumentSegment,
}

impl DeleteEvent {
    pub(crate) fn new(document: Document) -> Self {
        Self {
            doc_seg_to_delete: DocumentSegment::new(&document),
        }
    }
}

pub(crate) struct UpdateEvent {
    new_doc_seg: DocumentSegment,
}

impl UpdateEvent {
    pub(crate) fn new(document: Document) -> Self {
        Self {
            new_doc_seg: DocumentSegment::new(&document),
        }
    }
}

pub(crate) struct InsertEvent {
    doc_seg: DocumentSegment,
}

impl InsertEvent {
    pub(crate) fn new(doc: Document) -> Self {
        Self {
            doc_seg: DocumentSegment::new(&doc),
        }
    }
}

#[allow(dead_code)]
pub(crate) struct Flusher {
    manifest: Arc<RwLock<Manifest>>,
    filesystem: Arc<Box<dyn FileSystem>>,
    vector_store: Arc<VectorStore>,
    buffer: Mutex<Vec<JoinHandle<FlushEvent>>>,
    /// Internal counter for how big the buffer is for flushing purposes.
    buffer_size: AtomicUsize,
    max_buffer_size: usize,
}

impl Flusher {
    pub(crate) fn new(
        manifest: Arc<RwLock<Manifest>>,
        filesystem: Arc<Box<dyn FileSystem>>,
        vector_store: Arc<VectorStore>,
        opts: options::FlushOptions,
    ) -> Self {
        Self {
            manifest,
            filesystem,
            vector_store,
            buffer: Mutex::new(Vec::new()),
            buffer_size: AtomicUsize::new(0),
            max_buffer_size: opts.max_buffer_size,
        }
    }

    pub(crate) fn submit(&self, doc: Document, event_type: FlushEventType) -> Result<()> {
        let mut buffer_guard = self.buffer.lock().unwrap();
        let doc_for_thread = doc.clone();
        let handle = match event_type {
            FlushEventType::Delete => {
                thread::spawn(|| FlushEvent::Delete(DeleteEvent::new(doc_for_thread)))
            }
            FlushEventType::Update => {
                thread::spawn(|| FlushEvent::Update(UpdateEvent::new(doc_for_thread)))
            }
            FlushEventType::Insert => {
                thread::spawn(|| FlushEvent::Insert(InsertEvent::new(doc_for_thread)))
            }
        };

        buffer_guard.push(handle);
        drop(buffer_guard);
        self.buffer_size.fetch_add(doc.size(), Ordering::SeqCst);
        if self.buffer_size.load(Ordering::SeqCst) > self.max_buffer_size {
            self.flush()?;
        }

        Ok(())
    }

    pub(crate) fn flush(&self) -> Result<()> {
        let mut buffer_guard = self.buffer.lock().unwrap();
        let mut insert_events = Vec::new();
        let mut delete_events = Vec::new();
        let mut update_events = Vec::new();
        for event in buffer_guard.drain(0..) {
            let event = event.join();
            match event {
                Ok(e) => match e {
                    FlushEvent::Delete(delete_event) => delete_events.push(delete_event),
                    FlushEvent::Update(update_event) => update_events.push(update_event),
                    FlushEvent::Insert(insert_event) => insert_events.push(insert_event),
                },
                Err(_) => error!("problem joining on flush event"),
            }
        }
        self.flush_inserts(insert_events)?;
        self.flush_updates(update_events)?;
        self.flush_deletes(delete_events)?;
        self.buffer_size.swap(0, Ordering::SeqCst);
        Ok(())
    }

    fn flush_deletes(&self, delete_events: Vec<DeleteEvent>) -> Result<()> {
        for event in delete_events {
            let document_to_delete = event.doc_seg_to_delete.document();
            let doc_seg_to_delete = DocumentSegment::new(&document_to_delete);
            let manifest_guard = self.manifest.read().unwrap();
            // loop through the segments until you find the one with docucment_id
            let mut valid_docs = HashSet::new();
            let mut segment_to_recreate: Option<String> = None;
            for (segment_id, _) in manifest_guard.segments.clone() {
                let segment = Segment::read_from_fs(&segment_id, &**self.filesystem)?;
                // create a list of all the other docs that appear in that segment
                if segment.documents().contains(&document_to_delete.id()) {
                    segment_to_recreate = Some(segment.id());
                    for doc in segment.documents() {
                        if doc != document_to_delete.id() {
                            valid_docs.insert(doc);
                        }
                    }
                    break;
                }
            }
            drop(manifest_guard);
            assert!(segment_to_recreate.is_some());

            // reconstruct the segment from those documents
            let mut new_segment = Segment::new();
            for doc in valid_docs {
                let document = Document::get_doc_by_id(doc.to_string(), &**self.filesystem);
                if let Some(d) = document {
                    let doc_seg = DocumentSegment::new(&d);
                    new_segment.add_docucment_segment(&doc_seg);
                }
            }

            // write segment
            let (_, new_seg_size) = new_segment.write_to_fs(&**self.filesystem)?;

            // update manifest
            let mut manifest_guard = self.manifest.write().unwrap();
            manifest_guard
                .segments
                .remove(&segment_to_recreate.clone().unwrap());
            manifest_guard
                .segments
                .insert(new_segment.id(), new_seg_size);

            manifest_guard.average_document_length = ((manifest_guard.average_document_length
                * manifest_guard.total_docs as f64)
                - doc_seg_to_delete.document_length() as f64)
                / (manifest_guard.total_docs - 1) as f64;
            manifest_guard.total_docs -= 1;
            debug!(
                "Manifest object now contains segments: {:#?}, total docs: {}, and avg doc length: {}",
                manifest_guard.segments,
                manifest_guard.total_docs,
                manifest_guard.average_document_length,
            );
            let config = config::standard();
            drop(manifest_guard);
            let manifest_guard = self.manifest.read().unwrap();
            let bytes = bincode::encode_to_vec(&*manifest_guard, config)?;
            drop(manifest_guard);
            self.filesystem.write_to_manifest(bytes)?;

            // Remove vector search entries
            // self.vector_store.remove_document(&document_to_delete)?;

            // remove old segment
            self.filesystem
                .remove_from_index(Path::new(&format!("{}.seg", segment_to_recreate.unwrap())))?;

            // remove doc for doc_id
            self.filesystem
                .remove_from_documents(Path::new(&document_to_delete.id()))?;
        }
        Ok(())
    }

    fn flush_updates(&self, update_events: Vec<UpdateEvent>) -> Result<()> {
        for event in update_events {
            let document = event.new_doc_seg.document();
            let old_version = Document::get_doc_by_id(document.id(), &**self.filesystem);
            if old_version.is_none() {
                return Err(Error::msg(format!(
                    "Document {} does not exist in the index",
                    document.id(),
                )));
            }
            let doc_seg_to_delete = DocumentSegment::new(&old_version.unwrap());
            let manifest_guard = self.manifest.read().unwrap();
            // loop through the segments until you find the one with docucment_id
            let mut valid_docs = HashSet::new();
            let mut segment_to_recreate: Option<String> = None;
            for (segment_id, _) in manifest_guard.segments.clone() {
                let segment = Segment::read_from_fs(&segment_id, &**self.filesystem)?;
                // create a list of all the other docs that appear in that segment
                if segment.documents().contains(&document.id()) {
                    segment_to_recreate = Some(segment.id());
                    for doc in segment.documents() {
                        if doc != document.id() {
                            valid_docs.insert(doc);
                        }
                    }
                    break;
                }
            }
            debug!("valid docs: {:#?}", valid_docs);
            drop(manifest_guard);
            assert!(segment_to_recreate.is_some());

            // reconstruct the segment from those documents
            let mut new_segment = Segment::new();
            for doc in valid_docs {
                let document = Document::get_doc_by_id(doc.to_string(), &**self.filesystem);
                if let Some(d) = document {
                    let doc_seg = DocumentSegment::new(&d);
                    new_segment.add_docucment_segment(&doc_seg);
                }
            }
            // Add the updated document to the new segment
            let updated_dog_seg = &DocumentSegment::new(&event.new_doc_seg.document());
            new_segment.add_docucment_segment(updated_dog_seg);

            // overwrite old document with new document
            Document::write_doc(&event.new_doc_seg.document(), &**self.filesystem)?;

            // write segment
            let (_, new_seg_size) = new_segment.write_to_fs(&**self.filesystem)?;

            // update manifest
            let mut manifest_guard = self.manifest.write().unwrap();
            manifest_guard
                .segments
                .remove(&segment_to_recreate.clone().unwrap());

            manifest_guard
                .segments
                .insert(new_segment.id(), new_seg_size);
            manifest_guard.average_document_length = ((manifest_guard.average_document_length
                * manifest_guard.total_docs as f64)
                - doc_seg_to_delete.document_length() as f64
                + updated_dog_seg.document_length() as f64)
                / (manifest_guard.total_docs) as f64;
            debug!(
                "Manifest object now contains segments: {:#?}, total docs: {}, and avg doc length: {}",
                manifest_guard.segments,
                manifest_guard.total_docs,
                manifest_guard.average_document_length,
            );
            let config = config::standard();
            drop(manifest_guard);
            let manifest_guard = self.manifest.read().unwrap();
            let bytes = bincode::encode_to_vec(&*manifest_guard, config)?;
            drop(manifest_guard);
            self.filesystem.write_to_manifest(bytes)?;

            // Remove vector search entries for old, and re-insert new
            // self.vector_store.remove_document(&document)?;
            // self.vector_store.insert_document(&document)?;

            // remove old segment
            self.filesystem
                .remove_from_index(Path::new(&format!("{}.seg", segment_to_recreate.unwrap())))?;
        }

        Ok(())
    }

    fn flush_inserts(&self, insert_events: Vec<InsertEvent>) -> Result<()> {
        let mut segment = Segment::new();
        for event in insert_events {
            let doc_seg = event.doc_seg;
            segment.add_docucment_segment(&doc_seg);
            // self.vector_store.insert_document(&doc_seg.document())?;
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
