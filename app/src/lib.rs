use std::{error::Error, fs::{self, File, OpenOptions}, sync::mpsc::Sender};
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::{io::{self, BufReader, Write}, thread, sync::mpsc};
use std::time::Duration;
use std::io::BufRead;
use std::path::Path;

pub enum InterruptMessage {
    Play,
    Queue(String),
    Stop,
    Pause,
    Resume,
    Next,
    Previous,
    LoadPlaylist(String),
    CreatePlaylist(String),
    DeletePlaylist(String),
    AddToPlaylist(String),
    RemoveFromPlaylist(String),
    QueueToPlaylist(String),
    DisplayQueue,
}

fn get_base_directory() -> Result<String, Box<dyn Error>> {
    let env_name = ".env";
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
    let env_name = ".env";
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

fn create_playlist(playlist_name: String) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;
    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str();
    File::create(playlist_file_name)?;
    Ok(())
}

fn delete_playlist(playlist_name: String) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;
    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str();
    fs::remove_file(playlist_file_name)?;
    Ok(())
}

fn add_to_playlist(playlist_name: String, code: String) -> Result<(), Box<dyn Error>> {
    let code_existence = match does_song_code_exist(&code) {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => return Err(e)
    };
    if !code_existence {return Err("code DNE".into());}
    let base_directory = get_base_directory()?;
    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str();

    let mut playlist_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(playlist_file_name)?;

    let code_with_newline = format!("\n{}", code);

    playlist_file.write_all(code_with_newline.as_bytes())?;

    Ok(())
}

fn remove_nth_line<P: AsRef<Path> + std::convert::AsRef<std::path::Path>>(path: P, n: usize) -> std::io::Result<()> {
    let file = File::open(&path)?;
    let buf_reader = BufReader::new(file);
    let lines: Vec<String> = buf_reader.lines().collect::<Result<_, _>>()?;

    if lines.len() > n {
        let mut file = File::create(&path)?;
        for (i, line) in lines.into_iter().enumerate() {
            if i != n {
                writeln!(file, "{}", line)?;
            }
        }
    }

    Ok(())
}

fn remove_from_playlist(playlist_name: String, n: usize) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;
    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str();

    remove_nth_line(playlist_file_name, n)?;
    Ok(())
}

fn queue_to_playlist(playlist_name: String, queue: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let base_directory = get_base_directory()?;
    let playlist_file_name = base_directory + r"\playlists\" + playlist_name.as_str();

    let mut file = File::create(playlist_file_name)?;
    for i in queue {
        writeln!(file, "{i}")?;
    }

    Ok(())
}

fn does_song_code_exist(code: &str) -> Result<bool, Box<dyn Error>> {
    let base_directory = get_base_directory()?;
    let hash_table_name = base_directory + r"\hash-table\hash-table.txt"; 
    let hash_table = File::open(hash_table_name)?;
    let reader = BufReader::new(hash_table);

    for line in reader.lines() {
        if let Some((_,value)) = line?.split_once(":"){
            if value.trim() == code {
                return Ok(true);
            }
        }      
    };

    return Ok(false);
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
    let file = File::open(track_path)?;

    let source = Decoder::new(BufReader::new(file))?;

    if let Some(ref mut s) = sink {
        s.stop(); 
    }

    let new_sink = Sink::try_new(stream_handle)?;
    new_sink.append(source);
    *sink = Some(new_sink);

    if let Some(ref mut s) = sink {
        s.play();
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
            InterruptMessage::Play => {
                let song_code = (&queue[current_track]).to_string();
                let file_path = code_to_song_title(song_code).unwrap();

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
                    let file_path = code_to_song_title((&queue[current_track]).to_string())?;
                    
                    match play_track(&mut sink, file_path, &stream_handle){
                        Ok(_) => {},
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
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
            }, 
            InterruptMessage::CreatePlaylist(playlist_name) => {
                create_playlist(playlist_name)?;
            },
            InterruptMessage::DeletePlaylist(playlist_name) => {
                delete_playlist(playlist_name)?;
            },
            InterruptMessage::AddToPlaylist(data) => {
                let data_vec:Vec<&str> = data.split_whitespace().collect();

                if data_vec.len() != 2 {
                    continue;
                }

                add_to_playlist(data_vec[0].to_string(), data_vec[1].to_string())?;

            },
            InterruptMessage::RemoveFromPlaylist(data) => {
                let data_vec:Vec<&str> = data.split_whitespace().collect();

                if data_vec.len() != 2 {
                    continue;
                }

                remove_from_playlist(data_vec[0].to_string(), data_vec[1].parse()?)?;
            },
            InterruptMessage::QueueToPlaylist(playlist_name) => {
                queue_to_playlist(playlist_name, &queue)?;
            },
            InterruptMessage::DisplayQueue => println!("{queue:?}")

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
            "p" => {
                audiotx.send(InterruptMessage::Play).unwrap();
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
            },
            command if command.starts_with("crpl") => {
                let playlist_name = command[5..].to_string();
                audiotx.send(InterruptMessage::CreatePlaylist(playlist_name)).unwrap();
            },
            command if command.starts_with("dlpl") => {
                let playlist_name = command[5..].to_string();
                audiotx.send(InterruptMessage::DeletePlaylist(playlist_name)).unwrap();
            },
            command if command.starts_with("+pl") => {
                let data = command[4..].to_string();
                audiotx.send(InterruptMessage::AddToPlaylist(data)).unwrap();
            },
            command if command.starts_with("-pl") => {
                let data = command[4..].to_string();
                audiotx.send(InterruptMessage::RemoveFromPlaylist(data)).unwrap();
            },
            command if command.starts_with("q2p") => {
                let playlist_name = command[4..].to_string();
                audiotx.send(InterruptMessage::QueueToPlaylist(playlist_name)).unwrap();
            },
            "kyu" => audiotx.send(InterruptMessage::DisplayQueue).unwrap(),
            "e" => {
                break;
            },
            _ => println!("invalid command"),
        }
    }       
}