# PyVarro: python bindings for the Varro search index library

A Python wrapper around the Rust `varro` crate for efficient document indexing and search.

## Description

`pyvarro` provides Python bindings to the `varro` Rust library. varro was inspired by Apache Lucene and that makes pyvarro a possible alternative to the pylucene or similar lucene python wrapper packages, but without the need to have the JVM installed.

This wrapper is built using [PyO3](https://pyo3.rs/), ensuring seamless integration with Python while maintaining Rust's performance.

## Features

- Fast indexing and searching of Documents.
- Highly configurable and tunable to your needs.
- Local filesystem or s3 for durability.
- Compatible with Python 3.8+.

## Installation

Install via pip:

```bash
pip install pyvarro
```

No additional dependencies are required beyond the standard library.

## Usage

### Basic Example

```python
from pyvarro import PyVarro, Document

# Create a PyVarro instance with the default options
search_index = PyVarro(None, None, None)

# Add a document to the index
contents = "searching from pyvarro"
doc = Document(str(uuid.uuid4()))
doc.add_field("name", "document", False)
doc.add_field("content", contents, False)
varro.index(doc)

# Flush to make the document searchable
varro.flush()

# Search
results = varro.search("pyvarro", None)
for res in results:
	print(res)

```

Expect the Rust wrapper to be significantly faster for large inputs.

## Contributing

Contributions are welcome! Please:

1. Fork the repository.
2. Create a feature branch.
3. Submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
