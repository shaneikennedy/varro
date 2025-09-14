use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Document {
    pub(crate) doc: varro::Document,
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
    pub(crate) field: varro::Field,
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
