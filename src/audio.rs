use std::{env, sync::Arc};

use log::{debug, error, info};
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

pub async fn generate_wav(
    client: Arc<Client>,
    input: &str,
    speaker: Speakers,
) -> Result<Vec<u8>, String> {
    let speaker: u32 = speaker.into();
    info!(
        "Starting WAV generation for speaker {} with text length: {}",
        speaker,
        input.len()
    );
    debug!("Text content: {}", input);

    let query = AudioQuery {
        text: input.to_string(),
        speaker,
    };

    let Ok(origin) = env::var("VOICEVOX_ENGINE_URL") else {
        error!("VOICEVOX_ENGINE_URL environment variable not set");
        return Err("VOICEVOX_ENGINE_URL not set".to_string());
    };

    info!("Using VOICEVOX Engine at: {}", origin);

    // Step 1: Generate audio query
    debug!("Sending audio_query request to {}/audio_query", origin);
    let res = client
        .post(format!("{}/audio_query", origin))
        .query(&query)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send audio_query request: {}", e);
            format!("Failed to send request: {}", e)
        })?;

    debug!("audio_query response status: {}", res.status());
    if !res.status().is_success() {
        let status = res.status();
        error!("audio_query request failed with status: {}", status);
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Unable to get error text".to_string());
        error!("Error response body: {}", error_text);
        return Err(format!(
            "audio_query failed with status {}: {}",
            status, error_text
        ));
    }

    let bytes = res.bytes().await.map_err(|e| {
        error!("Failed to get audio_query response bytes: {}", e);
        format!("Failed to get response bytes: {}", e)
    })?;

    let query = String::from_utf8(bytes.to_vec()).map_err(|e| {
        error!("Failed to convert audio_query response to string: {}", e);
        format!("Failed to convert bytes to string: {}", e)
    })?;

    debug!("audio_query response length: {} bytes", query.len());

    // Step 2: Synthesize audio
    debug!(
        "Sending synthesis request to {}/synthesis?speaker={}",
        origin, speaker
    );
    let res = client
        .post(format!("{}/synthesis?speaker={}", origin, speaker))
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send synthesis request: {}", e);
            format!("Failed to send synthesis request: {}", e)
        })?;

    debug!("synthesis response status: {}", res.status());
    if res.status().is_success() {
        let bytes = res.bytes().await.map_err(|e| {
            error!("Failed to get synthesis response bytes: {}", e);
            format!("Failed to get synthesis response bytes: {}", e)
        })?;

        info!("Successfully generated WAV data: {} bytes", bytes.len());
        Ok(bytes.to_vec())
    } else {
        let status = res.status();
        error!("synthesis request failed with status: {}", status);
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Unable to get error text".to_string());
        error!("Error response body: {}", error_text);
        Err(format!(
            "synthesis failed with status {}: {}",
            status, error_text
        ))
    }
}
