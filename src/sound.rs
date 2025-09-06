use std::{io, sync::mpsc as std_mpsc, thread};

use log::{debug, error, info};

pub struct Player {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
}

impl Default for Player {
    fn default() -> Self {
        Self::new().expect("Failed to create default Player")
    }
}

impl Player {
    pub fn new() -> Result<Self, String> {
        debug!("Initializing audio output device");

        let (_stream, stream_handle) = rodio::OutputStream::try_default().map_err(|e| {
            error!("Failed to get default output stream: {}", e);
            format!("Failed to get default output stream: {}", e)
        })?;

        info!("Successfully initialized audio output stream");

        let sink = rodio::Sink::try_new(&stream_handle).map_err(|e| {
            error!("Failed to create audio sink: {}", e);
            format!("Failed to create sink: {}", e)
        })?;

        info!("Successfully created audio sink");

        Ok(Player { _stream, sink })
    }

    pub fn play(&self, bytes: Vec<u8>) -> Result<(), String> {
        debug!(
            "Starting audio playback with {} bytes of WAV data",
            bytes.len()
        );

        let sink = &self.sink;

        debug!("Stopping and clearing previous audio");
        sink.stop();
        sink.clear();

        debug!("Creating audio decoder from WAV data");
        let cursor = io::Cursor::new(bytes);
        let source = rodio::Decoder::new(cursor).map_err(|e| {
            error!("Failed to create audio decoder: {}", e);
            format!("Failed to create decoder: {}", e)
        })?;

        debug!("Adding audio source to sink");
        sink.append(source);

        debug!("Starting audio playback");
        sink.play();

        info!("Audio playback started successfully");
        Ok(())
    }
}

pub fn start_audio_loop() -> std_mpsc::Sender<Vec<u8>> {
    let (tx, rx) = std_mpsc::channel::<Vec<u8>>();

    thread::spawn(move || {
        info!("Starting audio playback thread");

        let player = match Player::new() {
            Ok(p) => {
                info!("Audio player created successfully");
                p
            }
            Err(e) => {
                error!("Failed to create audio player: {}", e);
                return;
            }
        };

        while let Ok(wav_data) = rx.recv() {
            debug!("Received audio playback request ({} bytes)", wav_data.len());

            match player.play(wav_data) {
                Ok(_) => {
                    info!("Audio playback completed successfully");
                }
                Err(e) => {
                    error!("Audio playback failed: {}", e);
                }
            }
        }

        info!("Audio playback thread terminated");
    });

    tx
}
