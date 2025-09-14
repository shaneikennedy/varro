# Varro

Marcus Terentius Varro was a Roman polymath, scholar, and writer, often considered one of the most learned men of ancient Rome. He was renowned for his vast knowledge and ability to compile, organize, and synthesize information from various sources, including text

## What
Varro is a text-based search engine inspired by Apache Lucene, attempting to offer a familiar API and general concepts, but with no attempt to be a drop in replacement for Lucene.

## TODOs
- [x] TFIDF scoring on search
- [x] BM25 scoring on search
- [x] Configurable boolean querying for multi-term queries (default OR)
- [x] Segment compaction and cleanup
- [ ] Indexing token positions in documents
- [ ] Compile to wasm and have full in-browser search engine
- [x] PyO3 packaging for python
- [ ] Agent with codebase indexed and varro tool call for codesearch instead of grep/ripgrep
- [ ] S3-compatible backend option for real durability
