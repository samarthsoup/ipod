import os

def create_structure(base_directory):
    subdirectories = ["playlists", "songs", "hash-table"]

    if not os.path.exists(base_directory):
        os.makedirs(base_directory)

    for directory in subdirectories:
        os.makedirs(os.path.join(base_directory, directory), exist_ok=True)