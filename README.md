# Varro

Marcus Terentius Varro was a Roman polymath, scholar, and writer, often considered one of the most learned men of ancient Rome. He was renowned for his vast knowledge and ability to compile, organize, and synthesize information from various sources, including text

## What
Varro is a text-based search engine inspired by Apache Lucene, attempting to offer a familiar API and general concepts, but with no attempt to be a drop in replacement for Lucene.

## TODOs
- [ ] Document updates by id
- [ ] Delete document by id (and remove indexed data)
- [ ] Field based indexing (default for a field is true but allow the user to choose to turn off a field)
- [ ] "Highlighting" i.e store where the token is for the document
- [ ] A query language to express more than just boolean query types
- [x] TFIDF scoring on search
- [x] BM25 scoring on search
- [x] Configurable boolean querying for multi-term queries (default OR)
- [x] Segment compaction and cleanup
- [ ] Compile to wasm and have full in-browser search engine
- [x] PyO3 packaging for python
- [ ] Agent with codebase indexed and varro tool call for codesearch instead of grep/ripgrep
- [x] S3-compatible backend option for real durability
