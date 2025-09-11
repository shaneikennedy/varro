#[derive(Clone)]
pub enum RankingType {
    /// The basic TF-IDF algorithm
    Tfidf,
}

pub(crate) fn score(
    tf: f64,
    total_docs: i32,
    docs_with_term: i32,
    ranking_type: &RankingType,
) -> f64 {
    match ranking_type {
        RankingType::Tfidf => {
            let idf = (total_docs as f64 / docs_with_term as f64).log10();
            tf * idf
        }
    }
}
