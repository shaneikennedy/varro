use std::path::Path;
use std::time::Duration;

use pyo3::prelude::*;
use varro::Varro;

#[pyclass]
pub struct PyVarro {
    varro: Varro,
}

#[pyclass]
#[derive(Clone)]
pub struct Document {
    doc: varro::Document,
}

#[pymethods]
impl PyVarro {
    #[new]
    pub fn new(
        min_segment_size: Option<usize>,
        compaction_frequency: Option<Duration>,
        max_buffer_size: Option<usize>,
    ) -> PyResult<PyVarro> {
        let mut varro = Varro::new(Path::new("./index")).unwrap();
        if let Some(min_segment_size) = min_segment_size {
            varro = varro.with_min_segment_size(min_segment_size);
        }
        if let Some(compation_freq) = compaction_frequency {
            varro = varro.with_compaction_frequency(compation_freq);
        }
        if let Some(max_buffer_size) = max_buffer_size {
            varro = varro.with_max_buffer_size(max_buffer_size);
        }
        Ok(PyVarro { varro })
    }

    /// Index a document, this takes a Document, stores it, adds the index to the document buffer, and returns whether it was successfull or not
    pub fn index(&self, doc: Document) -> PyResult<()> {
        self.varro.index(doc.doc).unwrap();
        Ok(())
    }

    /// The total number of docs in the Varro index
    pub fn index_size(&self) -> usize {
        self.varro.index_size()
    }
}
