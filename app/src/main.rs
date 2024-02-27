use std::{error::Error, fs::File, io::Read, process};
use dotenv::dotenv;
use std::env;

fn get_base_directory() -> Result<String, Box<dyn Error>> {
    dotenv().ok();

    env::var("BASE_DIR")
}

fn load_playlist(playlist_name: String, queue: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;

    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str(); 

    let mut playlist_file = File::open(playlist_file_name)?;
    let mut contents = String::new();
    playlist_file.read_to_string(&mut contents);

    let reader = BufReader::new(playlist_file);

    for line in reader.lines() {
        queue.push(line)?;
    }

    Ok(())
}

fn main() {
    

    
}
