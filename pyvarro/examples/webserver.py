from fastapi import FastAPI
from pyvarro import PyVarro

app = FastAPI()
varro = PyVarro(".index")
print(f"Current index size: {varro.index_size()}")

@app.get("/search")
def search(q: str):
    results = varro.search(q)
    print(f"Found {len(results)} matching documents")
    response = {"results": []}
    for (doc, score) in results:
        response["results"].append({"document_id": doc.id(), "score": score})

    return response
