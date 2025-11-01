import os
import uuid
from pyvarro import PyVarro, Document

def main():
    varro = PyVarro(".index")
    print(f"Index size before: {varro.index_size()}")
    directory_path = "../lib/documents"
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
                        doc = Document(str(uuid.uuid4()))
                        doc.add_field("name", filename)
                        doc.add_field("content", contents)
                        varro.index(doc)
                except Exception as e:
                    print(f"Error reading file {filename}: {str(e)}")
    except Exception as e:
        print(f"Error accessing directory {directory_path}: {str(e)}")
        return {}

    # Flush to make them searchable
    varro.flush()

    print(f"Index size after: {varro.index_size()}")


if __name__ == "__main__":
    main()
