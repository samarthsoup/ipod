import os
import sys

env_file_path = r'C:\Users\thesa\codes\ipod\app\.env'

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

env_vars = {}

if len(sys.argv) > 1:
    create_structure(sys.argv[1])
    with open(env_file_path, 'r') as file:
        for line in file:
            if '=' in line:
                key, value = line.strip().split('=', 1)
                env_vars[key] = value
    
    env_vars['BASE_DIR'] = sys.argv[1]

    with open(env_file_path, 'w') as file:
        for key, value in env_vars.items():
            file.write(f"{key}={value}\n")