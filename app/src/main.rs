use std::{collections::HashMap, error::Error, fs::File, io::Read, process};
use dotenv::dotenv;
use std::env;
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::{fs::File, io::{self, BufReader, Write}, thread, sync::mpsc};
use std::time::Duration;

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

fn play_track(sink: &mut Option<Sink>, track_path: String, stream_handle: &OutputStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    
    let file = match File::open(track_path){
        Ok(f) => f,
        Err(e) => {
            eprintln!("io error: {}", e);
            print!(": ");
            io::stdout().flush().unwrap();
            return Err(Box::new(e));
        }
    };
    let source = match Decoder::new(BufReader::new(file)){
        Ok(s) => s,
        Err(e) => {
            eprintln!("decoder error: {}", e);
            print!(": ");
            io::stdout().flush().unwrap();
            return Err(Box::new(e)); 
        }
    };

    if let Some(ref mut s) = sink {
        s.stop();
        s.append(source);
    } else {
        let new_sink = Sink::try_new(&stream_handle)?;
        new_sink.append(source);
        *sink = Some(new_sink);
    }
    
    Ok(())
}

fn main() {
    

    
}
