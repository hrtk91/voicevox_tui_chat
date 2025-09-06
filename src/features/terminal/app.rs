use ratatui::crossterm::event::{self, Event};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use crate::features::chat::{
    components::render_ui,
    events::{handle_chat_event, handle_key_event, ScrollAction},
    state::{AppState, MessageRole},
    worker::{create_chat_worker, ChatWorkerConfig},
};
use crate::features::voice;
use crate::sound;

pub async fn run_chat_terminal() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    let mut app_state = AppState::new();

    // 環境変数から設定を読み取り
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
    let system_prompt = std::env::var("SYSTEM_PROMPT").unwrap_or_else(|_| {
        r"あなたはチャットAIです。ユーザーと楽しく会話をしてください。
口語で話すときのように、一文を短く、会話形式での応答を心がけてください。"
            .to_string()
    });

    // ChatWorkerを起動
    let client = Arc::new(Client::new());
    let config = ChatWorkerConfig {
        api_key,
        model,
        system_prompt,
    };

    let (user_input_tx, mut chat_event_rx) = create_chat_worker(config, client.clone());

    // Audio loopを開始
    let audio_tx = sound::start_audio_loop();

    // 初期メッセージを追加
    let _system_id = app_state.add_message(
        MessageRole::System,
        "Chat Terminal Started. Type 'i' to enter insert mode and start chatting!".to_string(),
    );

    loop {
        // UI描画とレイアウト情報の取得
        let mut display_width = 80; // デフォルト値
        terminal.draw(|frame| {
            render_ui(frame, &app_state);
            display_width = frame.area().width.saturating_sub(2) as usize;
        })?;

        // イベント処理（ノンブロッキング）
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let (should_quit, scroll_action) =
                    handle_key_event(key, &mut app_state, Some(&user_input_tx));

                // スクロールアクションの処理
                if let Some(action) = scroll_action {
                    match action {
                        ScrollAction::Up => {
                            app_state.disable_auto_scroll();
                            app_state.scroll_up(display_width);
                        }
                        ScrollAction::Down => {
                            app_state.disable_auto_scroll();
                            app_state.scroll_down(display_width);
                        }
                        ScrollAction::ToTop => {
                            app_state.disable_auto_scroll();
                            app_state.scroll_to_top(display_width);
                        }
                        ScrollAction::ToBottom => {
                            // 明示的に最下部スクロールした場合は自動スクロール再有効化
                            app_state.enable_auto_scroll();
                            app_state.scroll_to_bottom(display_width);
                        }
                    }
                }

                if should_quit || app_state.should_quit {
                    break;
                }
            }
        }

        // ChatEventの処理（ノンブロッキング）
        while let Ok(chat_event) = chat_event_rx.try_recv() {
            voice::handle_voice_event(&chat_event, &app_state, client.clone(), audio_tx.clone());
            handle_chat_event(&mut app_state, chat_event);
            // ストリーミング中は自動的に最下部にスクロール
            app_state.auto_scroll_to_bottom(display_width);
        }
    }

    ratatui::restore();
    Ok(())
}
