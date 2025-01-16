use std::io::{self, Cursor, Write};
use reqwest::Client;
use rodio::Source;
use serde::Serialize;

#[derive(Serialize)]
struct AudioQuery {
    text: String,
    speaker: u32,
}
#[derive(Serialize)]
enum Speakers {
    Metan = 2,
    Zundamon = 3,
}

impl Into<u32> for Speakers {
    fn into(self) -> u32 {
        self as u32
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();
    let mut input = String::new();
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .expect("Failed to get default output stream");

    loop {
        print!("$>");
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin().read_line(&mut input).expect("Failed to read line");
        let trimmed_input = input.trim().to_string();

        let query = AudioQuery {
            text: trimmed_input.clone(),
            speaker: Speakers::Metan.into(),
        };

        let res = client.post("http://localhost:50021/audio_query")
            .query(&query)
            .send()
            .await;

        let resp = match res {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                continue;
            }
        };

        let bytes = match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to get response bytes: {}", e);
                continue;
            }
        };

        let query = match String::from_utf8(bytes.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to convert bytes to string: {}", e);
                continue;
            }
        };

        let resp = client.post("http://localhost:50021/synthesis?speaker=3")
            .body(query)
            .send()
            .await;

        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to send synthesis request: {}", e);
                continue;
            }
        };

        if resp.status().is_success() {
            let bytes = match resp.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Failed to get synthesis response bytes: {}", e);
                    continue;
                }
            };
            // rodioで再生
            let cursor = Cursor::new(bytes);
            let source = match rodio::Decoder::new(cursor) {
                Ok(decoder) => decoder,
                Err(e) => {
                    eprintln!("Failed to create decoder: {}", e);
                    continue;
                }
            };
            match stream_handle.play_raw(source.convert_samples()) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to play audio: {}", e),
            };
        } else {
            eprintln!("Failed to play audio: {}", resp.status());
        }
        input.clear();
    }
}
