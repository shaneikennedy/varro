use std::collections::HashMap;

use bincode::{Decode, Encode};

pub type SegmentId = String;
pub type SegmentSize = usize;
#[derive(Encode, Decode, Debug)]
pub(crate) struct Manifest {
    pub segments: HashMap<SegmentId, SegmentSize>,
}
