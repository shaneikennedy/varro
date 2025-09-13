use std::collections::HashMap;

use bincode::{Decode, Encode};

pub type SegmentId = String;
pub type SegmentSize = usize;
#[derive(Encode, Decode, Debug)]
pub(crate) struct Manifest {
    /// The active segments in the index used for searching.
    pub segments: HashMap<SegmentId, SegmentSize>,

    /// The total number of docs in the index.
    pub total_docs: usize,

    /// Average document length in the index.
    pub average_document_length: f64,
}
