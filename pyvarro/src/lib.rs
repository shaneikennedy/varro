mod document;
mod search;

#[pyo3::pymodule]
mod pyvarro {
    use crate::document::Document;
    use crate::search::SearchOptions;
    use pyo3::prelude::*;
    use std::path::Path;
    use std::time::Duration;
    use varro::Varro;

    #[pyclass]
    pub struct PyVarro {
        varro: Varro,
    }

    #[pymethods]
    impl PyVarro {
        /// Create a new instance of Varro
        /// `min_segment_size` is used to control compaction, it determines how big the segment files should be
        /// `compaction_frequency` controls how often compaction should happen
        /// `max_buffer_size` controls when Varro will automatically trigger a flush
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

        /// Retrive a document by it's Document.id, returns an Option type wrapping a Document
        pub fn retrieve(&self, id: String) -> Option<Document> {
            let result = self.varro.retrieve(id)?;
            Some(result.into())
        }

        /// Text search, given an input string query the index and return a list of Document Ids
        /// and their corresponding TDIDF score (higher is better) that match the search
        pub fn search(
            &self,
            query: String,
            options: Option<SearchOptions>,
        ) -> Vec<(Document, Score)> {
            let search_options = options.unwrap_or(SearchOptions::new(None, None, None));
            self.varro
                .search(query, Some(search_options.into()))
                .map(|(doc, score)| (doc.into(), score))
                .collect()
        }
    }

    pub type Score = f64;
}
