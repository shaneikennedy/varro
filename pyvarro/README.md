# PyVarro: python bindings for the Varro search index library

A Python wrapper around the Rust `varro` crate for efficient document indexing and search.

## Description

`pyvarro` provides Python bindings to the `varro` Rust library. varro was inspired by Apache Lucene and that makes pyvarro a possible alternative to the pylucene or similar lucene python wrapper packages, but without the need to have the JVM installed.

This wrapper is built using [PyO3](https://pyo3.rs/), ensuring seamless integration with Python while maintaining Rust's performance.

## Features

- Fast indexing and searching of Documents.
- Highly configurable and tunable to your needs.
- Local filesystem or s3 for durability.

## Installation

Install via pip:

```bash
pip install pyvarro
```

No additional dependencies are required beyond the standard library.

## Examples

Checkout the examples/ directory:
1. `uv run examples/ingest.py` to fill the index with the test documents that I've included in ../lib/documents for development purposes. The docs are posts from my blog in markdown
2. `uv run examples/search.py` to run a quick script that searches the index
3. `uv run uvicorn examples.webserver:app --reload` to run a fastapi backend that has a /search endpoint that takes a query parameter `q` and runs a search on the index. Use `curl http://127.0.0.1:8000/search\?q\=~name%3A%27docker%20tips%27 | jq` to test.
