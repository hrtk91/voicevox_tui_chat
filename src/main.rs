use env_logger::{Builder, Target};
use std::fs::OpenOptions;
use voicevox_chat::features::terminal::app::run_chat_terminal;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Setup logging to file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .expect("Failed to create log file");

    Builder::from_default_env()
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    color_eyre::install().expect("Failed to install color_eyre");

    // チャットターミナルUIを起動
    if let Err(e) = run_chat_terminal().await {
        eprintln!("Error running terminal UI: {}", e);
    }
}
