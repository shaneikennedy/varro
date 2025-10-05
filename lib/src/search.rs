use std::collections::HashMap;

use crate::{Document, Score};

pub fn or(
    existing: HashMap<Document, Score>,
    matches: Vec<(Document, Score)>,
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
        let matches = vec![(Document::new("a".into()), 1.0)];
        let res = or(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &2.0);
    }

    #[test]
    fn test_or_new_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 1.0);
        let matches = vec![(Document::new("b".into()), 1.0)];
        let res = or(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert!(res.contains_key(&Document::new("b".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &1.0);
        assert_eq!(res.get(&Document::new("b".into())).unwrap(), &1.0);
    }
}

pub fn and(
    existing: HashMap<Document, Score>,
    matches: Vec<(Document, Score)>,
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
        let matches = vec![(Document::new("a".into()), 1.0)];
        let res = and(existing, matches);
        assert!(res.contains_key(&Document::new("a".into())));
        assert_eq!(res.get(&Document::new("a".into())).unwrap(), &2.0);
    }

    #[test]
    fn test_or_new_values() {
        let mut existing = HashMap::new();
        existing.insert(Document::new("a".into()), 1.0);
        let matches = vec![(Document::new("b".into()), 1.0)];
        let res = and(existing, matches);
        assert!(!res.contains_key(&Document::new("a".into())));
        assert!(!res.contains_key(&Document::new("b".into())));
    }
}
