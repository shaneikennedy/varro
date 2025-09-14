use pyo3::prelude::*;

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
