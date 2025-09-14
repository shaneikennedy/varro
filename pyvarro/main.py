import os
import uuid
import pyvarro

def main():
    varro = pyvarro.PyVarro(None, None, None)
    directory_path = "../documents"
    try:
        # Check if directory exists
        if not os.path.isdir(directory_path):
            raise ValueError(f"Directory {directory_path} does not exist")

        # Iterate through all files in the directory
        for filename in os.listdir(directory_path):
            file_path = os.path.join(directory_path, filename)

            # Only the files
            if os.path.isfile(file_path):
                try:
                    # Add docs to index
                    with open(file_path, 'r', encoding='utf-8') as file:
                        contents = file.read()
                        doc = pyvarro.Document(str(uuid.uuid4()))
                        doc.add_field("name", "document", False)
                        doc.add_field("content", contents, False)
                        varro.index(doc)
                except Exception as e:
                    print(f"Error reading file {filename}: {str(e)}")
    except Exception as e:
        print(f"Error accessing directory {directory_path}: {str(e)}")
        return {}

    # Flush to make them searchable
    varro.flush()
    print(f"Hello from pyvarro! current index size: {varro.index_size()}")

    results = varro.search("git", None)
    for res in results:
        print(res)


if __name__ == "__main__":
    main()
