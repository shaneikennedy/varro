import os
import uuid
from pyvarro import PyVarro, Document

def main():
    varro = PyVarro(".index")
    print(f"Hello from pyvarro! current index size: {varro.index_size()}")

    results = varro.search("emacs")
    for (doc, score) in results:
        print(f"Document id: {doc.id()}, score: {score}")


if __name__ == "__main__":
    main()
