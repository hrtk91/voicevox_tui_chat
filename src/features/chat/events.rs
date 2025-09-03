use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

use super::state::{AppState, InputMode, MessageRole, MessageId, Content, ErrorMessage};

#[derive(Debug, Clone)]
pub enum ScrollAction {
    Up,
    Down,
    ToTop,
    ToBottom,
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    StreamingStart(MessageId),
    StreamingChunk(MessageId, Content),
    StreamingComplete(MessageId),
    Error(ErrorMessage),
}

pub fn handle_chat_event(app_state: &mut AppState, event: ChatEvent) {
    match event {
        ChatEvent::StreamingStart(_message_id) => {
            // 新しいストリーミングメッセージを開始
            let _actual_id = app_state.start_streaming_message(MessageRole::Assistant, String::new());
            // message_idとactual_idの対応を内部で管理する必要がある場合は追加実装
        }
        ChatEvent::StreamingChunk(_message_id, content) => {
            // 最後に追加されたストリーミングメッセージにcontentを追加
            if let Some(last_message) = app_state.messages.last_mut() {
                if last_message.is_streaming {
                    last_message.content.push_str(&content);
                }
            }
        }
        ChatEvent::StreamingComplete(_message_id) => {
            // 最後に追加されたストリーミングメッセージを完了状態にする
            if let Some(last_message) = app_state.messages.last_mut() {
                if last_message.is_streaming {
                    last_message.is_streaming = false;
                }
            }
        }
        ChatEvent::Error(error_msg) => {
            app_state.add_message(MessageRole::System, format!("Error: {}", error_msg));
        }
    }
}

pub fn handle_key_event(key: KeyEvent, state: &mut AppState, user_input_tx: Option<&mpsc::Sender<String>>) -> (bool, Option<ScrollAction>) {
    if key.kind != KeyEventKind::Press {
        return (false, None);
    }

    match state.input_mode {
        InputMode::Normal => handle_normal_mode(key, state),
        InputMode::Insert => handle_insert_mode(key, state, user_input_tx),
    }
}

fn handle_normal_mode(key: KeyEvent, state: &mut AppState) -> (bool, Option<ScrollAction>) {
    match key.code {
        KeyCode::Char('q') => {
            state.should_quit = true;
            (true, None)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            (false, Some(ScrollAction::Up))
        }
        KeyCode::Down | KeyCode::Char('j') => {
            (false, Some(ScrollAction::Down))
        }
        KeyCode::Char('g') => {
            (false, Some(ScrollAction::ToTop))
        }
        KeyCode::Char('G') => {
            (false, Some(ScrollAction::ToBottom))
        }
        KeyCode::Char('i') => {
            state.input_mode = InputMode::Insert;
            (false, None)
        }
        _ => (false, None),
    }
}

fn handle_insert_mode(key: KeyEvent, state: &mut AppState, user_input_tx: Option<&mpsc::Sender<String>>) -> (bool, Option<ScrollAction>) {
    match key.code {
        KeyCode::Esc => {
            state.input_mode = InputMode::Normal;
            (false, None)
        }
        KeyCode::Enter => {
            if !state.current_input.trim().is_empty() {
                // ユーザーの入力をメッセージとして追加
                let _user_id = state.add_message(MessageRole::User, state.current_input.clone());
                
                // 新しいメッセージ送信時に自動スクロールを再有効化
                state.enable_auto_scroll();
                
                // ChatWorkerに入力を送信
                if let Some(tx) = user_input_tx {
                    let input = state.current_input.clone();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx_clone.send(input).await {
                            eprintln!("Failed to send user input to ChatWorker: {}", e);
                        }
                    });
                }
                
                state.current_input.clear();
                // Normalモードに戻る
                state.input_mode = InputMode::Normal;
                // メッセージ送信後は最下部にスクロール
                (false, Some(ScrollAction::ToBottom))
            } else {
                (false, None)
            }
        }
        KeyCode::Char(c) => {
            state.current_input.push(c);
            (false, None)
        }
        KeyCode::Backspace => {
            state.current_input.pop();
            (false, None)
        }
        _ => (false, None),
    }
}