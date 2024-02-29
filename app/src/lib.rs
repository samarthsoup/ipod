use std::{error::Error, fs::File, sync::mpsc::Sender};
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::{io::{self, BufReader, Write}, thread, sync::mpsc};
use std::time::Duration;
use std::io::BufRead;

pub enum InterruptMessage {
    Play(String),
    Queue(String),
    Stop,
    Pause,
    Resume,
    Next,
    Previous,
    LoadPlaylist(String)
}

fn get_base_directory() -> Result<String, Box<dyn Error>> {
    let env_name = "env.txt";
    let env = File::open(env_name)?;

    let reader = BufReader::new(env);

    for line in reader.lines() {
        if let Some((key,value)) = line?.split_once('='){
            if key.trim() == "BASE_DIR" {
                return Ok(value.to_string());
            }
        }      
    };

    return Err("value not found".into());
}

fn get_songs_directory() -> Result<String, Box<dyn Error>> {
    let env_name = "env.txt";
    let env = File::open(env_name)?;

    let reader = BufReader::new(env);

    for line in reader.lines() {
        if let Some((key,value)) = line?.split_once('='){
            if key.trim() == "SONGS_DIR" {
                return Ok(value.to_string());
            } 
        }      
    };

    return Err("value not found".into());
}

fn load_playlist(playlist_name: String, queue: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;

    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str(); 

    let playlist_file = File::open(playlist_file_name)?;

    let reader = BufReader::new(playlist_file);

    for line in reader.lines() {
        queue.push(line?);
    }

    Ok(())
}

fn code_to_song_title(code: String) -> Result<String, Box<dyn Error>> {
    let base_directory = get_base_directory()?;

    let hash_table_name = base_directory + r"\hash-table\hash-table.txt"; 

    let hash_table = File::open(hash_table_name)?;

    let reader = BufReader::new(hash_table);

    for line in reader.lines() {
        if let Some((key,value)) = line?.split_once(":"){
            if value.trim() == code {
                return Ok(key.to_string());
            }
        }      
    };

    return Err("value not found".into());
}

fn play_track(sink: &mut Option<Sink>, track_name: String, stream_handle: &OutputStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    let songs_directory = get_songs_directory()?;

    let track_path = songs_directory + track_name.as_str();
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

pub fn play_mp3(rx: mpsc::Receiver<InterruptMessage>) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let mut queue: Vec<String> = Vec::new();
    let mut current_track = 0;

    let mut sink: Option<Sink> = None;

    for msg in rx {
        match msg {
            InterruptMessage::Play(song_code) => {
                if let Some(ref s) = sink {
                    s.stop();
                }

                queue.clear();
                current_track = 0;
                queue.push(song_code);

                let file_path = code_to_song_title(queue[current_track].clone())?;

                if let Some(ref s) = sink {
                    s.stop();
                }
                match play_track(&mut sink, file_path, &stream_handle){
                    Ok(_) => {},
                    Err(_) => continue
                };
            },
            InterruptMessage::Queue(song_code) => {
                queue.push(song_code);
            },
            InterruptMessage::Stop => {
                if let Some(ref s) = sink {
                    s.stop();
                }
                sink = None;
            },
            InterruptMessage::Pause => {
                if let Some(ref s) = sink {
                    s.pause();
                }
            },
            InterruptMessage::Resume => {
                if let Some(ref s) = sink {
                    s.play();
                }
            },
            InterruptMessage::Next => {
                if !queue.is_empty() {
                    current_track = (current_track + 1) % queue.len();
                    let file_path = code_to_song_title(queue[current_track].clone())?;

                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    match play_track(&mut sink, file_path, &stream_handle){
                        Ok(_) => {},
                        Err(_) => continue
                    };
                }
            },
            InterruptMessage::Previous => {
                if !queue.is_empty() {
                    if current_track == 0 {
                        current_track = queue.len() - 1;
                    } else {
                        current_track -= 1;
                    }

                    let file_path = code_to_song_title(queue[current_track].clone())?;

                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    match play_track(&mut sink, file_path, &stream_handle){
                        Ok(_) => {},
                        Err(_) => continue
                    };
                }
            },
            InterruptMessage::LoadPlaylist(playlist_name) => {
                load_playlist(playlist_name, &mut queue)?;
            }
        }
        if let Some(ref s) = &sink {
            if s.empty() && !queue.is_empty() {
                current_track = (current_track + 1) % queue.len();
                let file_path = code_to_song_title(queue[current_track].clone())?;

                match play_track(&mut sink, file_path, &stream_handle){
                    Ok(_) => {},
                    Err(_) => continue
                };
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

pub fn execute(audiotx: Sender<InterruptMessage>) {
    loop{
        print!(": ");
        io::stdout().flush().unwrap();


        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("read error");
            continue; 
        }
        let input = input.trim();

        match input {
            command if command.starts_with("p ") => {
                let song_code = command[2..].to_string();
                audiotx.send(InterruptMessage::Play(song_code)).unwrap();
            },
            command if command.starts_with("q ") => {
                let song_code = command[2..].to_string();
                audiotx.send(InterruptMessage::Queue(song_code)).unwrap();
            },
            "pz" => {
                audiotx.send(InterruptMessage::Pause).unwrap();
            },
            "r" => {
                audiotx.send(InterruptMessage::Resume).unwrap();
            },
            "s" => {
                audiotx.send(InterruptMessage::Stop).unwrap();
            },
            "nx" => {
                audiotx.send(InterruptMessage::Next).unwrap();
            },
            "pr" => {
                audiotx.send(InterruptMessage::Previous).unwrap();
            },
            command if command.starts_with("lp") => {
                let playlist_name = command[3..].to_string();
                audiotx.send(InterruptMessage::LoadPlaylist(playlist_name)).unwrap();
            }
            "h" => {
                println!("walkman docs\nplay: p {{songname}}\nqueue: q {{songname}}\npz: pause\nr: resume\nstop: s\nexit: e\ndocs: h");
            }
            "e" => {
                break;
            },
            _ => println!("invalid command"),
        }
    }       
}