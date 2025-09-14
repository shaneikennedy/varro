use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub enum SearchOperator {
    Or,
    And,
}

#[pyclass]
#[derive(Clone)]
pub enum RankingType {
    Tfidf,
    Bm25,
}

#[derive(Clone)]
#[pyclass]
#[allow(dead_code)]
pub struct SearchOptions {
    options: varro::SearchOptions,
}

#[pymethods]
impl SearchOptions {
    #[new]
    pub fn new(
        include_documents: Option<bool>,
        operator: Option<SearchOperator>,
        ranking_type: Option<RankingType>,
    ) -> PyResult<SearchOptions> {
        let mut opts = varro::SearchOptions::default();
        if let Some(include_documents) = include_documents {
            opts = opts.include_documents(include_documents);
        }
        if let Some(operator) = operator {
            opts = match operator {
                SearchOperator::Or => opts.search_operator(varro::SearchOperator::OR),
                SearchOperator::And => opts.search_operator(varro::SearchOperator::AND),
            };
        }
        if let Some(ranking_type) = ranking_type {
            opts = match ranking_type {
                RankingType::Tfidf => opts.ranking_type(varro::RankingType::Tfidf),
                RankingType::Bm25 => opts.ranking_type(varro::RankingType::Bm25),
            };
        }
        Ok(SearchOptions { options: opts })
    }
}
