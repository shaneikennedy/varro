use std::collections::HashMap;

use bincode::{Decode, Encode};

use crate::{document, tokens};

/// A Segment is just a map of terms to TFDFs for a given "flush".
#[derive(Encode, Decode, Debug)]
pub(crate) struct Segment {
    pub(crate) term_index: HashMap<String, Tfdf>,
}

impl Segment {
    pub fn new() -> Self {
        Self {
            term_index: HashMap::new(),
        }
    }

    // For a segment, update the existing term_index with all
    // the terms and corresponding TFs from DocumentSegment
    // if the term doesn't already exist in the term index, insert a new one
    pub fn add_docucment_segment(&mut self, seg: &DocumentSegment) {
        for (term, _) in seg.terms.iter() {
            if self.term_index.contains_key(term) {
                self.term_index
                    .entry(term.to_string())
                    .and_modify(|t| t.add_for_doc(seg));
            } else {
                let mut tfdf = Tfdf::new(term);
                tfdf.add_for_doc(seg);
                self.term_index.insert(term.to_string(), tfdf);
            }
        }
    }

    // Used during segment compaction
    pub fn add_segment(&mut self, seg: Segment) {
        for (term, tfdf) in seg.term_index {
            self.term_index
                .entry(term)
                .and_modify(|t| {
                    t.doc_freq += tfdf.doc_freq;
                    t.term_freq.extend(tfdf.term_freq.clone());
                })
                .or_insert(tfdf);
        }
    }
}

/// A TFDF is holds the info for which documents (id, the String in the term_freq map) have a given term and it's count (the i32 in the term_freq map)
/// and the total number of documents that the term appears in
#[derive(Encode, Decode, Debug)]
pub(crate) struct Tfdf {
    pub term: String,

    // Each tuple is a document_id, and the normalized fresquency
    // for the term in this doc, that is, # occurances / total words in the doc
    pub term_freq: Vec<(String, f64)>,
    pub doc_freq: i32,
}

impl Tfdf {
    pub fn new(term: &str) -> Self {
        Self {
            term: term.into(),
            term_freq: Vec::new(),
            doc_freq: 0,
        }
    }

    pub fn add_for_doc(&mut self, doc_seg: &DocumentSegment) {
        self.term_freq.push((
            doc_seg.document_id(),
            *doc_seg.terms.get(&self.term).unwrap() as f64 / doc_seg.document_length as f64, // Normalize the TF by the document length
        ));
        self.doc_freq += 1;
    }
}

#[derive(Debug)]
pub(crate) struct DocumentSegment {
    document_id: String,
    // Total number of words in the doc
    document_length: i32,
    terms: HashMap<String, i32>,
}

impl DocumentSegment {
    pub fn new(doc: &document::Document) -> Self {
        let mut doc_seg = DocumentSegment {
            document_id: doc.id(),
            document_length: 0,
            terms: HashMap::new(),
        };
        let mut word_count = 0;
        for field in doc.fields() {
            let content = field.contents();
            let content = tokens::tokenize(&content);
            content.for_each(|w| {
                word_count += 1;
                doc_seg.terms.entry(w).and_modify(|v| *v += 1).or_insert(1);
            });
        }
        doc_seg.document_length = word_count;
        doc_seg
    }

    pub fn document_id(&self) -> String {
        self.document_id.clone()
    }
}

#[cfg(test)]
mod document_segment_tests {
    use super::*;

    #[test]
    fn test_document_segment() {
        let mut doc = document::Document::default();
        doc.add_field("name".into(), "wow such nice test".into(), true);
        doc.add_field("body".into(), "wow such nice test again".into(), true);
        let doc_seg = DocumentSegment::new(&doc);
        assert_eq!(doc.id(), doc_seg.document_id);
        assert_eq!(doc_seg.terms.get("wow"), Some(&2));
        assert_eq!(doc_seg.terms.get("such"), Some(&2));
        assert_eq!(doc_seg.terms.get("nice"), Some(&2));
        assert_eq!(doc_seg.terms.get("test"), Some(&2));
        assert_eq!(doc_seg.terms.get("again"), Some(&1));
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;

    #[test]
    fn test_add_document_segment() {
        let mut segment = Segment::new();
        let mut doc1 = document::Document::default();
        doc1.add_field(
            "content".into(),
            "mes deux chats chili och arnie".into(),
            false,
        );
        let doc_seg_1 = DocumentSegment::new(&doc1);
        segment.add_docucment_segment(&doc_seg_1);

        assert!(segment.term_index.contains_key("deux"));
        let tfdf = segment.term_index.get("deux").unwrap();
        assert_eq!(tfdf.term, "deux");
        assert_eq!(tfdf.term_freq.len(), 1);
        assert!(
            tfdf.term_freq
                .iter()
                .any(|(doc_id, _)| doc_id == &doc1.id())
        );
    }

    #[test]
    fn test_add_segment() {
        let mut segment = Segment::new();
        let mut tfdf = Tfdf::new("deux");
        tfdf.term_freq.push(("doc1".into(), 0.43));
        segment.term_index.insert("deux".into(), tfdf);

        let mut segment_to_merge = Segment::new();
        let mut tfdf = Tfdf::new("deux");
        tfdf.term_freq.push(("doc2".into(), 0.43));
        segment_to_merge.term_index.insert("deux".into(), tfdf);
        let mut tfdf = Tfdf::new("coucou");
        tfdf.term_freq.push(("doc2".into(), 0.43));
        segment_to_merge.term_index.insert("coucou".into(), tfdf);
        segment.add_segment(segment_to_merge);

        assert!(segment.term_index.contains_key("deux"));
        assert!(segment.term_index.contains_key("coucou"));
        assert_eq!(segment.term_index.get("deux").unwrap().term_freq.len(), 2);
    }
}
