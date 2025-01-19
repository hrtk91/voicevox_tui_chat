use std::{
    io,
    sync::{Arc, Mutex},
};

pub struct Player {
    sink: Arc<Mutex<rodio::Sink>>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        let (_stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to get default output stream");

        let sink = rodio::Sink::try_new(&stream_handle).expect("Failed to create sink");

        Player {
            sink: Arc::new(Mutex::new(sink)),
        }
    }

    pub fn play(&self, bytes: Vec<u8>) {
        let Ok(sink) = self.sink.lock() else {
            eprintln!("Failed to lock sink");
            return;
        };
        sink.stop();
        sink.clear();

        let cursor = io::Cursor::new(bytes);
        let source = rodio::Decoder::new(cursor).expect("Failed to create decoder");

        sink.append(source);
        sink.play();
    }
}
