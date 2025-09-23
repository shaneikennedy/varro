use wasm_bindgen::prelude::*;
use std::path::Path;
use std::time::Duration;
use varro::{FileSystemType, Varro};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Document {
    pub(crate) doc: varro::Document,
}

#[wasm_bindgen]
impl Document {
    #[wasm_bindgen(constructor)]
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
#[wasm_bindgen]
#[derive(Clone)]
pub struct Field {
    pub(crate) field: varro::Field,
}

#[wasm_bindgen]
impl Field {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, contents: &str) -> Field {
        Field {
            field: varro::Field::new(name, contents, true),
        }
    }
    pub fn name(&self) -> String {
        self.field.name()
    }
    pub fn contents(&self) -> String {
        self.field.contents()
    }
}

#[wasm_bindgen]
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

#[wasm_bindgen]
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
#[wasm_bindgen]
#[allow(dead_code)]
pub struct SearchOptions {
    options: varro::SearchOptions,
}

impl From<SearchOptions> for varro::SearchOptions {
    fn from(val: SearchOptions) -> varro::SearchOptions {
        val.options
    }
}

#[wasm_bindgen]
impl SearchOptions {
    #[wasm_bindgen(constructor)]
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

#[wasm_bindgen]
pub struct VarroJS {
    varro: Varro,
}

#[wasm_bindgen]
impl VarroJS {
    /// Create a new instance of Varro
    /// `min_segment_size` is used to control compaction, it determines how big the segment files should be
    /// `compaction_frequency_millis` controls how often compaction should happen, in milliseconds
    /// `max_buffer_size` controls when Varro will automatically trigger a flush
    #[wasm_bindgen(constructor)]
    pub fn new(
        min_segment_size: Option<usize>,
        compaction_frequency_millis: Option<u64>,
        max_buffer_size: Option<usize>,
    ) -> VarroJS {
        let mut varro = Varro::new(Path::new(".index"), FileSystemType::Local).unwrap();
        if let Some(min_segment_size) = min_segment_size {
            varro = varro.with_min_segment_size(min_segment_size);
        }
        if let Some(compation_freq) = compaction_frequency_millis {
            varro = varro.with_compaction_frequency(Duration::from_millis(compation_freq));
        }
        if let Some(max_buffer_size) = max_buffer_size {
            varro = varro.with_max_buffer_size(max_buffer_size);
        }
        VarroJS { varro }
    }

    /// Index a document, this takes a Document, stores it, adds the index to the document buffer, and returns whether it was successfull or not
    pub fn index(&self, doc: Document) {
        self.varro.index(doc.doc).unwrap();
    }

    /// The total number of docs in the Varro index
    pub fn index_size(&self) -> usize {
        self.varro.index_size()
    }

    /// Flush the indexes to disk, this needs to happen before a document is searchable
    pub fn flush(&self)  {
        self.varro.flush().unwrap();
    }

    /// Retrive a document by it's Document.id, returns an Option type wrapping a Document
    pub fn retrieve(&self, id: String) -> Option<Document> {
        let result = self.varro.retrieve(id)?;
        Some(result.into())
    }

    /// Text search, given an input string query the index and return a list of Document Ids
    /// and their corresponding TDIDF score (higher is better) that match the search
    pub fn search(&self, query: String, options: Option<SearchOptions>) -> Vec<SearchResult> {
        let search_options = options.unwrap_or(SearchOptions::new(None, None, None));
        self.varro
            .search(query, Some(search_options.into()))
            .map(|(doc, score)| SearchResult{doc:doc.into(), socre: score })
            .collect()
    }
}

#[wasm_bindgen]
pub struct SearchResult {
    pub doc: Document,
    pub socre: Score,
}

pub type Score = f64;
