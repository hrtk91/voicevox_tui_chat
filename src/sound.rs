use std::io;

pub struct Player {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
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

        Player { _stream, sink }
    }

    pub fn play(&self, bytes: Vec<u8>) {
        self.sink.stop();
        self.sink.clear();

        let cursor = io::Cursor::new(bytes);
        let source = rodio::Decoder::new(cursor).expect("Failed to create decoder");

        self.sink.append(source);
        self.sink.play();
    }
}
