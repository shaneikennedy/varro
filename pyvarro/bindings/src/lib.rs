#[pyo3::pymodule]
#[pyo3(name = "lib")]
mod pyvarro {
    use pyo3::prelude::*;
    use std::path::Path;
    use std::time::Duration;
    use varro::{FileSystemType, Varro};

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

    impl From<varro::Document> for Document {
        fn from(value: varro::Document) -> Self {
            let mut doc = Document::new(value.id());
            for field in value.fields() {
                doc.add_field(field.name(), field.contents(), false);
            }
            doc
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

    #[pyclass]
    #[derive(Clone)]
    pub enum SearchOperator {
        Or,
        And,
    }

    impl From<SearchOperator> for varro::SearchOperator {
        fn from(val: SearchOperator) -> varro::SearchOperator {
            match val {
                SearchOperator::Or => varro::SearchOperator::OR,
                SearchOperator::And => varro::SearchOperator::AND,
            }
        }
    }

    #[pyclass]
    #[derive(Clone)]
    pub enum RankingType {
        Tfidf,
        Bm25,
    }

    impl From<RankingType> for varro::RankingType {
        fn from(val: RankingType) -> varro::RankingType {
            match val {
                RankingType::Tfidf => varro::RankingType::Tfidf,
                RankingType::Bm25 => varro::RankingType::Bm25,
            }
        }
    }

    #[derive(Clone)]
    #[pyclass]
    #[allow(dead_code)]
    pub struct SearchOptions {
        options: varro::SearchOptions,
    }

    impl From<SearchOptions> for varro::SearchOptions {
        fn from(val: SearchOptions) -> varro::SearchOptions {
            val.options
        }
    }

    #[pymethods]
    impl SearchOptions {
        #[new]
        pub fn new(
            include_documents: Option<bool>,
            operator: Option<SearchOperator>,
            ranking_type: Option<RankingType>,
        ) -> SearchOptions {
            let mut opts = varro::SearchOptions::default();
            if let Some(include_documents) = include_documents {
                opts = opts.with_include_documents(include_documents);
            }
            if let Some(operator) = operator {
                opts = opts.with_search_operator(operator.into());
            }
            if let Some(ranking_type) = ranking_type {
                opts = opts.with_ranking_type(ranking_type.into());
            }
            SearchOptions { options: opts }
        }
    }

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
            let mut varro = Varro::new(Path::new(".index"), FileSystemType::Local).unwrap();
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
