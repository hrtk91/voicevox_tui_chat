use std::{env, io::{self, Write}};
use colored::Colorize;
use voicevox_chat::sound;
use reqwest::Client;
use voicevox_chat::audio_processor::to_audio;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Client::new();
    let mut input = String::new();
    let sound = sound::Sound::new();
    let system_message = Message {
        role: "system".into(),
        content: env::var("PROMPT").unwrap_or(r"
                あなたはチャットAIです。ユーザーと楽しく会話をしてください。
                口語で話すときのように、一文を短く、会話形式での応答を心がけてください。
            ".into()
        ),
    };
    let mut logs: Vec<Message> = vec![];

    loop {
        print!("{}", "you>".blue().bold());
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin().read_line(&mut input).expect("Failed to read line");
        let trimmed_input = input.trim().to_string();

        // チャット履歴は30件まで
        if logs.len() > 30 {
            logs.remove(0);
        }

        // ユーザーの入力をログに追加
        logs.push(Message {
            role: "user".into(),
            content: trimmed_input,
        });

        // システムメッセージ、過去ログを結合
        let messages = vec![system_message.clone()].into_iter().chain(logs.iter().cloned()).collect::<Vec<Message>>();

        // リクエストボディを作成
        let body = json!({
            "model": "gpt-4o-mini",
            "messages":  messages
        });

        // OpenAI APIへのリクエストを送信
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            Ok(response) => response,
            Err(e) => {
                eprintln!("{}", format!("Error sending request: {}", e).red());
                continue;
            }
        };

        // レスポンスのステータスコードを確認
        if !resp.status().is_success() {
            eprintln!("{}", format!("リクエストが失敗しました。ステータスコード: {}", resp.status()).red());
            continue;
        }

        // レスポンスボディをJSONとして解析
        let json: serde_json::Value = match resp.json().await {
            Ok(json) => json,
            Err(e) => {
                eprintln!("{}", format!("Error parsing response JSON: {}", e).red());
                continue;
            }
        };
        // AIからの応答を取得して表示
        if let Some(reply) = json["choices"][0]["message"]["content"].as_str() {
            print!("{}", "AI >".to_string().green().bold());
            println!("{}", reply);

            // AIの応答をログに追加
            logs.push(Message {
                role: "assistant".into(),
                content: reply.to_string(),
            });
            // AIの応答を音声に変換して再生
            match to_audio(&client, reply, voicevox_chat::audio_processor::Speakers::Metan).await {
                Ok(bytes) => {
                    sound.play(bytes);
                }
                Err(e) => {
                    eprintln!("{}", format!("Error processing audio: {}", e).red());
                }
            }
        } else {
            eprintln!("{}", "AIからの応答を取得できませんでした。".red());
        }
        
        input.clear();
    }
}
