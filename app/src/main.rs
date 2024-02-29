use std::{thread, sync::mpsc};

fn main() {
    let (audiotx, audiorx) = mpsc::channel();

    thread::spawn(move || {
        match app::play_mp3(audiorx){
            Ok(()) => {},
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    });

    app::execute(audiotx);
}
