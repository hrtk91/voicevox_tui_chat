use super::events::ChatEvent;
use crate::openai::ChatCompletion;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct ChatWorkerConfig {
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

pub struct ChatWorker {
    chat_completion: ChatCompletion,
    user_input_rx: mpsc::Receiver<String>,
    chat_event_tx: mpsc::Sender<ChatEvent>,
}

impl ChatWorker {
    pub fn new(
        config: ChatWorkerConfig,
        client: Arc<Client>,
        user_input_rx: mpsc::Receiver<String>,
        chat_event_tx: mpsc::Sender<ChatEvent>,
    ) -> Self {
        let mut chat_completion = ChatCompletion::new(config.api_key.clone(), client);
        chat_completion.push_system_message(&config.system_prompt);

        Self {
            chat_completion,
            user_input_rx,
            chat_event_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(user_input) = self.user_input_rx.recv().await {
            // ユーザー入力をChatCompletionに追加
            self.chat_completion.push_user_message(&user_input);

            // ストリーミングレスポンス開始を通知
            let message_id = Uuid::new_v4().to_string();
            if let Err(e) = self
                .chat_event_tx
                .send(ChatEvent::StreamingStart(message_id.clone()))
                .await
            {
                eprintln!("Failed to send StreamingStart event: {}", e);
                break;
            }

            // OpenAI APIからストリーミングレスポンスを取得
            let event_tx = self.chat_event_tx.clone();
            let msg_id = message_id.clone();

            let result = self
                .chat_completion
                .completion_stream(move |chunk| {
                    let tx = event_tx.clone();
                    let id = msg_id.clone();
                    let chunk_str = chunk.to_string();

                    // ブロッキング送信を使用してSendエラーを回避
                    let _ = tx.try_send(ChatEvent::StreamingChunk(id, chunk_str));
                })
                .await;

            match result {
                Ok(full_response) => {
                    // レスポンス完了を通知
                    if let Err(e) = self
                        .chat_event_tx
                        .send(ChatEvent::StreamingComplete(message_id))
                        .await
                    {
                        eprintln!("Failed to send StreamingComplete event: {}", e);
                        break;
                    }

                    // ChatCompletionの履歴にレスポンスを追加
                    self.chat_completion.push_assistant_message(&full_response);
                }
                Err(e) => {
                    // エラーを通知
                    let error_msg = e.to_string();
                    if let Err(send_err) =
                        self.chat_event_tx.send(ChatEvent::Error(error_msg)).await
                    {
                        eprintln!("Failed to send Error event: {}", send_err);
                        break;
                    }
                }
            }
        }
    }
}

pub fn create_chat_worker(
    config: ChatWorkerConfig,
    client: Arc<Client>,
) -> (mpsc::Sender<String>, mpsc::Receiver<ChatEvent>) {
    let (user_input_tx, user_input_rx) = mpsc::channel::<String>(32);
    let (chat_event_tx, chat_event_rx) = mpsc::channel::<ChatEvent>(32);

    let worker = ChatWorker::new(config, client, user_input_rx, chat_event_tx);

    tokio::spawn(async move {
        worker.run().await;
    });

    (user_input_tx, chat_event_rx)
}
