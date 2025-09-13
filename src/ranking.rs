#[derive(Clone)]
pub enum RankingType {
    /// The basic TF-IDF algorithm
    Tfidf,

    /// The BM25 ranking algorithm
    Bm25,
}

// For a beginners buide to BM25 read this article from ElasticSearch
// https://www.elastic.co/blog/practical-bm25-part-2-the-bm25-algorithm-and-its-variables

// K1 is a variable which helps determine term frequency saturation characteristics.
// That is, it limits how much a single query term can affect the score of a given document.
const K1: f64 = 1.2;

// B is a variable that controls the effect of the length of the document compared to the average length in the index
// If B is bigger documents that are longer have their score reduced. Imagine an document with 1 match for a query
// term and 300 words, vs a document qith 10 words and 1 match: in normal tfidf these two docs get ranked simlarly,
// but with bm25 the smaller doc gets ranked higher
const B: f64 = 0.75;

pub(crate) fn score(
    tf: f64,
    total_docs: i32,
    docs_with_term: i32,
    doc_length: i32,
    average_doc_length: f64,
    ranking_type: &RankingType,
) -> f64 {
    match ranking_type {
        RankingType::Tfidf => {
            let idf = (total_docs as f64 / docs_with_term as f64).log10();
            tf * idf
        }
        RankingType::Bm25 => {
            let idf = (1.0
                + (total_docs as f64 - docs_with_term as f64 + 0.5)
                    / (docs_with_term as f64 + 0.5))
                .ln();

            idf * ((tf * (K1 + 1.0))
                / (tf + K1 * (1.0 - B + B * (doc_length as f64 / average_doc_length))))
        }
    }
}
