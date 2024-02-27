use std::{collections::HashMap, error::Error, fs::File, io::Read, process};
use dotenv::dotenv;
use std::env;

fn get_base_directory() -> Result<String, Box<dyn Error>> {
    dotenv().ok();

    env::var("BASE_DIR")
}

fn get_songs_directory() -> Result<String, Box<dyn Error>> {
    dotenv().ok();

    env::var("SONGS_DIR")
}

fn load_playlist(playlist_name: String, queue: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;

    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str(); 

    let mut playlist_file = File::open(playlist_file_name)?;

    let reader = BufReader::new(playlist_file);

    for line in reader.lines() {
        queue.push(line)?;
    }

    Ok(())
}

fn code_to_song_title(code: String) -> Result<String, Box<dyn Error>> {
    let base_directory = get_base_directory()?;

    let hash_table_name = base_directory + r"\hash-table\hash-table.txt"; 

    let mut hash_table = File::open(hash_table_name)?;

    codes = HashMap::new();

    let reader = BufReader::new(hash_table);

    for line in reader.lines() {
        if let Some((key,value)) = line.split_once(":"){
            if value == code {
                return key;
            } else {
                return Box::new("delimiter not found");
            }
        } 
            
    }
}

fn main() {
    

    
}
