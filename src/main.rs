use std::{env, io::{self, Write}};
use agent_demo::sound;
use reqwest::Client;
use agent_demo::audio_processor::to_audio;
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
        content: "あなたはチャットAIです。ユーザーと楽しく会話をしてください。時には生意気な皮肉やユーモアを交えて会話を盛り上げてください。口語で話すときのように、一文を短く、会話形式での応答を心がけてください。".into(),
    };
    let mut logs: Vec<Message> = vec![];

    loop {
        print!("you>");
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
                eprintln!("Error sending request: {}", e);
                continue;
            }
        };

        // レスポンスのステータスコードを確認
        if !resp.status().is_success() {
            eprintln!("リクエストが失敗しました。ステータスコード: {}", resp.status());
            continue;
        }

        // レスポンスボディをJSONとして解析
        let json: serde_json::Value = match resp.json().await {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error parsing response JSON: {}", e);
                continue;
            }
        };
        // AIからの応答を取得して表示
        if let Some(reply) = json["choices"][0]["message"]["content"].as_str() {
            println!("AI >{}", reply);
            // AIの応答をログに追加
            logs.push(Message {
                role: "assistant".into(),
                content: reply.to_string(),
            });
            // AIの応答を音声に変換して再生
            match to_audio(&client, &reply, agent_demo::audio_processor::Speakers::Metan).await {
                Ok(bytes) => {
                    sound.play(bytes);
                }
                Err(e) => {
                    eprintln!("Error processing audio: {}", e);
                }
            }
        } else {
            println!("AIからの応答を取得できませんでした。");
        }
        
        input.clear();
    }
}
