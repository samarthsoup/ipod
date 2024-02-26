import os
import sys

def create_structure(base_directory):
    subdirectories = ["playlists", "hash-table"]

    if not os.path.exists(base_directory):
        os.makedirs(base_directory)

    for directory in subdirectories:
        os.makedirs(os.path.join(base_directory, directory), exist_ok=True)

    file_path = base_directory + r"\hash-table\hash-table.txt"

    if not os.path.exists(file_path):
        with open(file_path, 'w') as file:
            file.write("")

if len(sys.argv) > 1:
    create_structure(sys.argv[1])