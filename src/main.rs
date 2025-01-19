use colored::Colorize;
use reqwest::Client;
use std::{
    env,
    io::{self, Write},
    sync::Arc,
};
use voicevox_chat::audio::generate_wav;
use voicevox_chat::{openai::ChatCompletion, sound};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = Arc::new(Client::new());
    let mut input = String::new();
    let sound = sound::Player::new();
    let mut chat_completion = ChatCompletion::new(api_key, client.clone());
    chat_completion.push_system_message(
        env::var("PROMPT")
            .unwrap_or(
                r"
            あなたはチャットAIです。ユーザーと楽しく会話をしてください。
            口語で話すときのように、一文を短く、会話形式での応答を心がけてください。
            "
                .into(),
            )
            .as_str(),
    );

    loop {
        print!("{}", "you>".blue().bold());
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed_input = input.trim().to_string();

        // ユーザーの入力をログに追加
        chat_completion.push_user_message(&trimmed_input);

        // OpenAI APIへのリクエストを送信
        let reply = match chat_completion.completion().await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("{}", format!("Error sending request: {}", e).red());
                continue;
            }
        };

        // AIからの応答を取得して表示
        print!("{}", "AI >".to_string().green().bold());
        println!("{}", reply);

        // AIの応答をログに追加
        chat_completion.push_assistant_message(&reply);

        // AIの応答を音声に変換して再生
        let wav = generate_wav(
            client.clone(),
            reply.as_str(),
            voicevox_chat::audio::Speakers::Metan,
        )
        .await;

        match wav {
            Ok(bytes) => {
                sound.play(bytes);
            }
            Err(e) => {
                eprintln!("{}", format!("Error processing audio: {}", e).red());
            }
        }

        input.clear();
    }
}
