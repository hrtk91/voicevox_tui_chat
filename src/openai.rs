use std::sync::Arc;

use colored::Colorize;
use futures::StreamExt;
use serde::Serialize;

pub struct ChatCompletion {
    api_key: String,
    client: Arc<reqwest::Client>,
    model: String,
    log_size: u32,
    system_messages: Vec<Message>,
    chat_messages: Vec<Message>,
}

#[derive(Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl ChatCompletion {
    pub fn new(api_key: String, client: Arc<reqwest::Client>) -> Self {
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5-nano".to_string());

        Self {
            api_key,
            client,
            model: model.clone(),
            log_size: 30,
            system_messages: vec![],
            chat_messages: vec![],
        }
    }

    pub fn push_system_message(&mut self, prompt: &str) {
        let system_message = Message {
            role: "system".into(),
            content: prompt.into(),
        };

        self.system_messages.push(system_message);
    }

    fn push_chat_message(&mut self, role: &str, input: &str) {
        if self.chat_messages.len() > self.log_size as usize {
            self.chat_messages.remove(0);
        }

        self.chat_messages.push(Message {
            role: role.into(),
            content: input.into(),
        });
    }

    pub fn push_user_message(&mut self, input: &str) {
        self.push_chat_message("user", input);
    }

    pub fn push_assistant_message(&mut self, input: &str) {
        self.push_chat_message("assistant", input);
    }

    pub fn messages(&self) -> Vec<Message> {
        self.system_messages
            .iter()
            .cloned()
            .chain(self.chat_messages.clone())
            .collect()
    }

    pub async fn completion(&self) -> Result<String, reqwest::Error> {
        let body = serde_json::json!({
          "model": self.model,
          "messages": self.messages(),
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            // レスポンスのステータスコードを確認
            Ok(response) => match response.error_for_status() {
                Ok(valid_response) => valid_response,
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!(
                            "Request body: {}",
                            serde_json::to_string_pretty(&body).unwrap_or_default()
                        )
                        .yellow()
                    );
                    eprintln!("{}", format!("Error in response status: {}", e).red());
                    return Err(e);
                }
            },
            Err(e) => {
                eprintln!(
                    "{}",
                    format!(
                        "Request body: {}",
                        serde_json::to_string_pretty(&body).unwrap_or_default()
                    )
                    .yellow()
                );
                eprintln!("{}", format!("Error sending request: {}", e).red());
                return Err(e);
            }
        };

        let resp_json: serde_json::Value = resp.json().await?;
        let choices = resp_json["choices"]
            .as_array()
            .expect("choices is not an array");
        let text = choices[0]["message"]["content"]
            .as_str()
            .expect("content is not a string");

        Ok(text.to_string())
    }

    pub async fn completion_stream<F>(
        &self,
        mut callback: F,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(&str),
    {
        let body = serde_json::json!({
          "model": self.model,
          "messages": self.messages(),
          "stream": true,
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            Ok(response) => match response.error_for_status() {
                Ok(valid_response) => valid_response,
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!(
                            "Request body: {}",
                            serde_json::to_string_pretty(&body).unwrap_or_default()
                        )
                        .yellow()
                    );
                    eprintln!("{}", format!("Error in response status: {}", e).red());
                    return Err(Box::new(e));
                }
            },
            Err(e) => {
                eprintln!(
                    "{}",
                    format!(
                        "Request body: {}",
                        serde_json::to_string_pretty(&body).unwrap_or_default()
                    )
                    .yellow()
                );
                eprintln!("{}", format!("Error sending request: {}", e).red());
                return Err(Box::new(e));
            }
        };

        let mut full_content = String::new();
        let mut bytes_stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut byte_buffer = Vec::new(); // UTF-8バイトバッファ

        while let Some(chunk) = bytes_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // バイトバッファに追加
                    byte_buffer.extend_from_slice(&bytes);

                    // UTF-8文字列として変換を試行
                    let chunk_str = match std::str::from_utf8(&byte_buffer) {
                        Ok(valid_str) => {
                            // 完全な文字列なので、バッファをクリアして使用
                            let result = valid_str.to_string();
                            byte_buffer.clear();
                            result
                        }
                        Err(utf8_error) => {
                            // 不完全なUTF-8バイト列の場合
                            let valid_up_to = utf8_error.valid_up_to();
                            if valid_up_to == 0 {
                                // 先頭から無効な場合、次のチャンクを待つ
                                continue;
                            }

                            // 有効な部分を取得
                            let valid_str = std::str::from_utf8(&byte_buffer[..valid_up_to])
                                .unwrap()
                                .to_string();

                            // 不完全な部分を残す
                            let remaining_bytes = byte_buffer[valid_up_to..].to_vec();
                            byte_buffer = remaining_bytes;

                            valid_str
                        }
                    };

                    buffer.push_str(&chunk_str);

                    // Process complete lines
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer.drain(..=newline_pos).collect::<String>();
                        let line = line.trim();

                        if line.starts_with("data: ") {
                            let data = &line[6..]; // Remove "data: " prefix

                            if data == "[DONE]" {
                                break;
                            }

                            // Parse JSON chunk
                            match serde_json::from_str::<serde_json::Value>(data) {
                                Ok(json) => {
                                    if let Some(choices) =
                                        json.get("choices").and_then(|c| c.as_array())
                                    {
                                        if let Some(choice) = choices.first() {
                                            if let Some(delta) =
                                                choice.get("delta").and_then(|d| d.as_object())
                                            {
                                                if let Some(content) =
                                                    delta.get("content").and_then(|c| c.as_str())
                                                {
                                                    if !content.is_empty() {
                                                        callback(content);
                                                        full_content.push_str(content);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "{}",
                                        format!("JSON parse error: {} for data: {}", e, data).red()
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading stream: {}", e).red());
                    return Err(Box::new(e));
                }
            }
        }

        Ok(full_content)
    }
}

impl ChatCompletion {
    pub fn api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key = api_key.to_string();
        self
    }

    pub fn model(&mut self, model_name: &str) -> &mut Self {
        self.model = model_name.to_string();
        self
    }

    pub fn client(&mut self, client: Arc<reqwest::Client>) -> &mut Self {
        self.client = client;
        self
    }

    pub fn log_size(&mut self, size: u32) -> &mut Self {
        self.log_size = size;
        self
    }
}
