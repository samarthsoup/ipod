use std::{collections::HashMap, error::Error, fs::File, io::Read, process};
use dotenv::dotenv;
use std::env;
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::{fs::File, io::{self, BufReader, Write}, thread, sync::mpsc};
use std::time::Duration;

enum InterruptMessage {
    Play(String),
    Queue(String),
    Stop,
    Pause,
    Resume,
    Next,
    Previous,
}

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

fn play_mp3(rx: mpsc::Receiver<InterruptMessage>) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let mut queue: Vec<String> = Vec::new();
    let mut current_track = 0;

    let mut sink: Option<Sink> = None;

    for msg in rx {
        match msg {
            InterruptMessage::Play(file_path) => {
                if let Some(ref s) = sink {
                    s.stop();
                }

                queue.clear();
                current_track = 0;
                queue.push(file_path);

                if let Some(ref s) = sink {
                    s.stop();
                }
                match play_track(&mut sink, &queue, current_track, &stream_handle){
                    Ok(_) => {},
                    Err(_) => continue
                };
            },
            InterruptMessage::Queue(file_path) => {
                queue.push(file_path);
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
                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    match play_track(&mut sink, &queue, current_track, &stream_handle){
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
                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    match play_track(&mut sink, &queue, current_track, &stream_handle){
                        Ok(_) => {},
                        Err(_) => continue
                    };
                }
            }
        }
        if let Some(ref s) = &sink {
            if s.empty() && !queue.is_empty() {
                current_track = (current_track + 1) % queue.len();
                match play_track(&mut sink, &queue, current_track, &stream_handle){
                    Ok(_) => {},
                    Err(_) => continue
                };
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn main() {
    let (audiotx, audiorx) = mpsc::channel();

    thread::spawn(move || {
        match play_mp3(audiorx){
            Ok(()) => {},
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    });

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
                let
                let file_path = format!(r"C:\Users\thesa\walkman\src\{}.mp3", command[2..].to_string());
                audiotx.send(InterruptMessage::Play(file_path)).unwrap();
            },
            command if command.starts_with("q ") => {
                let file_path = format!(r"C:\Users\thesa\walkman\src\{}.mp3", command[2..].to_string());
                audiotx.send(InterruptMessage::Queue(file_path)).unwrap();
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
