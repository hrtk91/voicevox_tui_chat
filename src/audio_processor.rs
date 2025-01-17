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

impl Into<u32> for Speakers {
    fn into(self) -> u32 {
        self as u32
    }
}

pub async fn to_audio(client: &Client, input: &str, speaker: Speakers) -> Result<Vec<u8>, String> {
    let speaker: u32 = speaker.into();
    let query = AudioQuery {
        text: input.to_string(),
        speaker: speaker.into(),
    };

    let res = client.post("http://localhost:50021/audio_query")
        .query(&query)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let bytes = res.bytes().await.map_err(|e| format!("Failed to get response bytes: {}", e))?;

    let query = String::from_utf8(bytes.to_vec()).map_err(|e| format!("Failed to convert bytes to string: {}", e))?;

    let res = client.post(
            format!("http://localhost:50021/synthesis?speaker={}", speaker)
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
