use std::env;

use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct AudioQuery {
    text: String,
    speaker: u32,
}

#[derive(Serialize)]
pub enum Speakers {
    Metan = 2,
    Zundamon = 3,
}

impl From<Speakers> for u32 {
    fn from(val: Speakers) -> Self {
        val as u32
    }
}

pub async fn to_audio(client: &Client, input: &str, speaker: Speakers) -> Result<Vec<u8>, String> {
    let speaker: u32 = speaker.into();
    let query = AudioQuery {
        text: input.to_string(),
        speaker,
    };

    let origin = env::var("VOICEVOX_ENGINE_URL").unwrap_or("http://localhost:50021".to_string());

    let res = client.post(format!("{}/audio_query", origin))
        .query(&query)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let bytes = res.bytes().await.map_err(|e| format!("Failed to get response bytes: {}", e))?;

    let query = String::from_utf8(bytes.to_vec()).map_err(|e| format!("Failed to convert bytes to string: {}", e))?;

    let res = client.post(
            format!("{}/synthesis?speaker={}", origin, speaker)
        )
        .body(query)
        .send()
        .await
        .map_err(|e| format!("Failed to send synthesis request: {}", e))?;

    if res.status().is_success() {
        let bytes = res.bytes().await.map_err(|e| format!("Failed to get synthesis response bytes: {}", e))?;
        Ok(bytes.to_vec())
    } else {
        Err(format!("Failed to play audio: {}", res.status()))
    }
}
