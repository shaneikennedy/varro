use std::{
    collections::HashMap,
    fmt::Display,
    path::Path,
    sync::{Arc, RwLock},
};

use bincode::config;
use log::{debug, warn};

use crate::{
    Document, Score,
    filesystem::FileSystem,
    manifest::Manifest,
    ranking,
    segment::{DocumentSegment, Segment},
    vql::{self, Engine, Node, Token},
};

pub fn or(
    existing: HashMap<Document, Score>,
    matches: HashMap<Document, Score>,
) -> HashMap<Document, Score> {
    let mut matching = existing.clone();
    for (doc, score) in matches {
        match existing.contains_key(&doc) {
            true => {
                matching.entry(doc).and_modify(|v| {
                    *v += score;
                });
            }
            false => {
                matching.insert(doc, score);
            }
        }
    }
    matching
}

#[cfg(test)]
mod search_or_tests {

    use super::*;

    #[test]
    fn test_or_existing_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 1.0);
        let matches = [(Document::new("a".into()), 1.0)].into();
        let res = or(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &2.0);
    }

    #[test]
    fn test_or_new_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 1.0);
        let matches = [(Document::new("b".into()), 1.0)].into();
        let res = or(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert!(res.contains_key(&Document::new("b".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &1.0);
        assert_eq!(res.get(&Document::new("b".into())).unwrap(), &1.0);
    }
}

pub fn and(
    existing: HashMap<Document, Score>,
    matches: HashMap<Document, Score>,
) -> HashMap<Document, Score> {
    let matching = existing.clone();
    let mut new_matching: HashMap<Document, Score> = HashMap::new();
    // Populate new_matching with only the terms that also exist
    // in the already existing matches (i.e AND)
    for (doc, score) in matches {
        if matching.contains_key(&doc) {
            let old_score = matching.get(&doc).unwrap();
            new_matching.insert(doc, score * old_score);
        }
    }
    new_matching
}

#[cfg(test)]
mod search_and_tests {

    use super::*;

    #[test]
    fn test_and_existing_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 2.0);
        let matches = [(Document::new("a".into()), 1.0)].into();
        let res = and(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &2.0);
    }

    #[test]
    fn test_or_new_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 1.0);
        let matches = [(Document::new("b".into()), 1.0)].into();
        let res = and(existing, matches);
        assert!(!res.contains_key(&Document::new("a".into())));
        assert!(!res.contains_key(&Document::new("b".into())));
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(dead_code)]
enum Op {
    Include,
    Exclude,
    Similar,
}

impl From<vql::Op> for Op {
    fn from(value: vql::Op) -> Self {
        match value {
            vql::Op::Include => Op::Include,
            vql::Op::Exclude => Op::Exclude,
            vql::Op::Similar => Op::Similar,
        }
    }
}

#[allow(dead_code)]
pub(crate) struct Selector {
    op: Op,
    tag: Option<String>,
    word: String,
}

impl Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "op: {:#?}, tag: {:#?}, word: {}",
            self.op, self.tag, self.word
        )
    }
}

#[allow(dead_code)]
pub(crate) struct Searcher {
    filesystem: Arc<Box<dyn FileSystem>>,
    manifest: Arc<RwLock<Manifest>>,
}

#[allow(dead_code)]
impl Searcher {
    pub fn new(filesystem: Arc<Box<dyn FileSystem>>, manifest: Arc<RwLock<Manifest>>) -> Self {
        Self {
            filesystem,
            manifest,
        }
    }

    fn get_matching_docs_with_score(
        &self,
        selector: &Selector,
        segment: &Segment,
    ) -> impl Iterator<Item = (Document, Score)> {
        let mut matching = HashMap::new();
        if let Some(tfdf) = segment.term_index.get(&selector.word) {
            let docs_with_term = tfdf.term_freq.len();
            debug!("Total docs for term {}: {docs_with_term}", selector.word);
            tfdf.term_freq.iter().for_each(|(doc_id, tf)| {
                // TODO better way to quickly get the stats of a doc (like length)
                let document_length =
                    DocumentSegment::new(&self.get_doc_by_id(doc_id.to_string()).unwrap())
                        .document_length();
                let manifest = self.manifest.read().unwrap();
                let score = ranking::score(
                    *tf,
                    manifest.total_docs as i32,
                    docs_with_term as i32,
                    document_length,
                    manifest.average_document_length,
                    &ranking::RankingType::Bm25,
                );
                drop(manifest);
                matching.insert(Document::new(doc_id.to_string()), score);
            });
        } else {
            debug!("No docs matching selector: {selector}");
        }
        matching.into_iter()
    }

    fn search_for_selector(
        &self,
        selector: Selector,
        segment: &Segment,
    ) -> HashMap<Document, Score> {
        match selector.op {
            Op::Include => self
                .get_matching_docs_with_score(&selector, segment)
                .collect(),
            Op::Exclude => {
                // This is going to be inefficient. Get all docs in the index (just the id, which is a ls on the documents dir)
                // Find all documents that _would_ match the tag:word
                // and take the difference all - matching
                let all_docs = self.filesystem.list_documents();
                let matching: HashMap<Document, Score> = self
                    .get_matching_docs_with_score(&selector, segment)
                    .collect();
                let result: HashMap<Document, Score> = all_docs
                    .iter()
                    .filter_map(|d| {
                        let doc = Document::new(d.to_string());
                        match matching.contains_key(&doc) {
                            true => Some((doc, 0.05)), // Arbitrary 0.05 score for exclusion results
                            false => None,
                        }
                    })
                    .collect();
                result
            }
            Op::Similar => {
                warn!("Varro does not support similarity selections yet, defaulting to no matches");
                HashMap::new()
            }
        }
    }

    pub fn search(&self, query: &str, segment: &Segment) -> HashMap<Document, Score> {
        let engine = Engine::new();
        let ast = engine.execute(query);
        debug!("Ast from query: {:#?}", ast);
        self._search(ast, segment)
    }

    fn _search(&self, ast: Node, segment: &Segment) -> HashMap<Document, Score> {
        match ast {
            Node::Selector(token) => match token {
                Token::Selector(op, _, word) => {
                    let s = Selector {
                        op: op.into(),
                        tag: None,
                        word,
                    };
                    self.search_for_selector(s, segment)
                }
                _ => panic!("Something went wrong in selector"),
            },
            Node::BinaryOp(node, token, node1) => {
                let left = self._search(*node, segment);
                let right = self._search(*node1, segment);
                match token {
                    Token::And => and(left, right),
                    Token::Or => or(left, right),
                    _ => panic!("Something went wrong in binary op"),
                }
            }
        }
    }

    fn get_doc_by_id(&self, id: String) -> Option<Document> {
        let file = self.filesystem.read_from_documents(Path::new(&id.clone()));
        match file {
            Ok(f) => {
                let config = config::standard();
                let (decoded, _): (Document, usize) =
                    bincode::decode_from_slice(&f[..], config).unwrap();
                Some(decoded)
            }
            Err(_) => None,
        }
    }
}
