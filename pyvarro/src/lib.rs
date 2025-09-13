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
impl Document {
    #[new]
    pub fn new(id: String) -> Self {
        Self {
            doc: varro::Document::new(id),
        }
    }

    /// Add a field to the document
    pub fn add_field(&mut self, name: String, contents: String, index: bool) {
        self.doc.add_field(name, contents, index);
    }

    /// Return the number of bytes allocated by a document
    pub fn size(&self) -> usize {
        self.doc.size()
    }
}

/// The model representing a field in a document
#[pyclass]
#[derive(Clone)]
pub struct Field {
    field: varro::Field,
}

#[pymethods]
impl Field {
    #[new]
    pub fn new(name: &str, contents: &str) -> PyResult<Field> {
        Ok(Field {
            field: varro::Field::new(name, contents),
        })
    }
    pub fn name(&self) -> String {
        self.field.name()
    }
    pub fn contents(&self) -> String {
        self.field.contents()
    }
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

    /// Flush the indexes to disk, this needs to happen before a document is searchable
    pub fn flush(&self) -> PyResult<()> {
        self.varro.flush().unwrap();
        Ok(())
    }
}
