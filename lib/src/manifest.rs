use std::collections::HashMap;

use bincode::{Decode, Encode};

pub type SegmentId = String;
pub type SegmentSize = usize;
#[derive(Encode, Decode, Debug)]
pub(crate) struct Manifest {
    /// The active segments in the index used for searching.
    pub(crate) segments: HashMap<SegmentId, SegmentSize>,

    /// The total number of docs in the index.
    pub(crate) total_docs: usize,

    /// Average document length in the index.
    pub(crate) average_document_length: f64,
}

impl Manifest {
    pub(crate) fn new() -> Self {
        Self {
            segments: HashMap::new(),
            total_docs: 0,
            average_document_length: 0.0,
        }
    }
}
