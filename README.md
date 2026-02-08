# Varro

Marcus Terentius Varro was a Roman polymath, scholar, and writer, often considered one of the most learned men of ancient Rome. He was renowned for his vast knowledge and ability to compile, organize, and synthesize information from various sources, including text

## What
Varro is a text-based search engine inspired by Apache Lucene, attempting to offer a familiar API and general concepts, but with no attempt to be a drop in replacement for Lucene.

## Getting started
#### lib
This is where the rust library code lives, it's the actual implementation of Varro.

```
cd lib
cargo build
cargo fmt
cargo clippy
cargo test
```

Theres a bunch of example programs to run in lib/examples, they all rely on a "documents" folder, which for now is a collection of my own blog posts, in markdown.
```
cargo run --example ingest # this will ingest the /documents folder into a search index on your local machine (.index/ in this directory)
cargo run --example webserver # a basic webserver where you can issue queries via curl
curl http://127.0.0.1:8080\?q\=\~%27first%20taste%20of%20emacs%27 | jq # this will query ~'first taste of emacs' which will do a vector search on that query string
curl http://127.0.0.1:8080\?q\=\emacs%26-gorilla | jq # this will query emacs & -gorilla, so documents with the term emacs but not gorilla. See lib/src/vql.rs for more query info
```
#### pyvarro
Python bindings for the Varro library. This is a wip and not actively maintained. The gist is that the pyvarro/bindings rust project essentially wraps the Varro library and exports python types. Then on build it generates a .so file. Copy that .so file into the pyvarro python source code (uv project) to run pyvarro with the latest Varro version.

pyvarro/examples has a uv project that builds the pyvarro library and runs a basic index and search program.

#### varrojs
Completely unready.

### VQL
Custom query language for querying the Varro search index. It's basic, but growing. Today it is essentially boolean expressions on "selectors" where a selector is of the form `operation?field?:?query-string`.

Valid operations +,-,~ which are include, exclude and similar, respectively.
- **\+** BM25 search on the index
- **\-** BM25 negation, essentially run BM25 on the index and exclude these documents from the results
- **~** vector search

Some valid queries:
- `title:cats & cats | -body:dog & (title:dog | ~body:hound)`
- `cats`
- `~body:'hello from the other siiiiiiiide' & -artist:adelle`


### Semantic Search
As you might have guessed from the query language, Varro supports mixing BM25 search with Vector Search to give you the best of both when it suits you. The vector search is backed by a sqlite database.


### Storage
The Varro library lets you choose from using the local filesystem (default) or S3-compatible object storage for where the index is stored. The FileSystem trait is what defines how Varro reads and writes results to the index, so in theory you could write your own "Filesystem" backend and plug it in. Right now it's not possible to plug in a non-supported filesystem however, maybe something for the future.
