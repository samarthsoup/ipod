import random
import string
import os

base_directory = None
songs_directory = None

env_file_path = r'C:\Users\thesa\codes\ipod\app\.env'
with open(env_file_path, 'r') as file:
    lines = file.readlines()
    for line in lines:
        if line.startswith('BASE_DIR='):
            base_directory = line.strip().split('=')[1]
        if line.startswith('SONGS_DIR='):
            songs_directory = line.strip().split('=')[1]

mp3_files = {}
codes_set = set()

def generate_code():
    return ''.join(random.choices(string.ascii_letters + string.digits, k=3))

def generate_unique_code():
    while True:
        code = generate_code()
        if code not in codes_set:
            codes_set.add(code)
            return code
        
hash_table_path = os.path.join(base_directory, 'hash-table', 'hash-table.txt')
if os.path.exists(hash_table_path):
    with open(hash_table_path, 'r') as file:
        for line in file:
            key, value = line.strip().split(': ')
            mp3_files[key] = value
            codes_set.add(value)

for root, dirs, files in os.walk(songs_directory):
    for file in files:
        if file.endswith('.mp3'):
            relative_path = os.path.relpath(os.path.join(root, file), start=songs_directory)
            if relative_path not in mp3_files:
                unique_code = generate_unique_code()
                mp3_files[relative_path] = unique_code

with open(base_directory + r'\hash-table\hash-table.txt', 'w') as file:
    for key, value in mp3_files.items():
        file.write(f"{key}: {value}\n")