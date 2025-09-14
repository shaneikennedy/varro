# PyVarro: python bindings for the Varro search index library

## Developing
`cargo build` will produce a dylib file in target/debug. If you're on a mac, copy/move/symlink this file to varro.so in the root of the pyvarro directory. And then you can run `uv run main.py` to as a test.
