use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::features::chat::state::{AppState, InputMode};

pub fn handle_settings_mode(key: KeyEvent, state: &mut AppState) -> bool {
    let max_items = state.current_settings.len();

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.input_mode = InputMode::Normal;
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_settings_selection_up();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_settings_selection_down(max_items);
            false
        }
        _ => false,
    }
}
