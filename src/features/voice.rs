use super::chat::events::ChatEvent;
use super::chat::state::{AppState, MessageRole};
use crate::audio;
use log::{debug, error, info, warn};
use reqwest::Client;
use std::sync::Arc;

pub async fn speak_text(
    client: Arc<Client>,
    text: &str,
    audio_tx: &std::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), String> {
    debug!("Starting voice synthesis for text: {}", text);

    let wav_data = match audio::generate_wav(client, text, audio::Speakers::Zundamon).await {
        Ok(data) => {
            info!("Successfully generated WAV data ({} bytes)", data.len());
            data
        }
        Err(e) => {
            error!("Failed to generate WAV data: {}", e);
            return Err(e);
        }
    };

    debug!("Sending audio data to playback system");
    match audio_tx.send(wav_data) {
        Ok(_) => {
            info!("Audio data sent successfully");
            Ok(())
        }
        Err(_) => {
            error!("Failed to send audio data - channel closed");
            Err("Audio channel closed".to_string())
        }
    }
}

pub fn handle_voice_event(
    chat_event: &ChatEvent,
    app_state: &AppState,
    client: Arc<Client>,
    audio_tx: std::sync::mpsc::Sender<Vec<u8>>,
) {
    if let ChatEvent::StreamingComplete(_) = chat_event {
        if let Some(last_message) = app_state.messages.last() {
            if last_message.role == MessageRole::Assistant {
                let text = last_message.content.clone();
                info!("Triggering voice synthesis for assistant message");
                debug!("Message content (length: {}): {}", text.len(), text);

                tokio::spawn(async move {
                    match speak_text(client, &text, &audio_tx).await {
                        Ok(_) => {
                            info!("Voice synthesis completed successfully");
                        }
                        Err(e) => {
                            error!("Voice synthesis failed: {}", e);
                        }
                    }
                });
            } else {
                debug!("Skipping voice synthesis for non-assistant message");
            }
        } else {
            warn!("No messages found when trying to synthesize voice");
        }
    }
}
