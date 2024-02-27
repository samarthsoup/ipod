import sys

env_file_path = r'C:\Users\thesa\codes\ipod\app\.env'

env_vars = {}

if len(sys.argv) > 1:
    with open(env_file_path, 'r') as file:
        for line in file:
            if '=' in line:
                key, value = line.strip().split('=', 1)
                env_vars[key] = value

env_vars['SONGS_DIR'] = sys.argv[1]

with open(env_file_path, 'w') as file:
    for key, value in env_vars.items():
        file.write(f"{key}={value}\n")