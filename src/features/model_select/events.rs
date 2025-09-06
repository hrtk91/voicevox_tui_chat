use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::features::chat::state::{AppState, InputMode, MessageRole};

pub fn handle_model_select_mode(key: KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.input_mode = InputMode::Normal;
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_model_selection_up();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_model_selection_down();
            false
        }
        KeyCode::Enter => {
            if let Some(selected_model) = state.get_selected_model().cloned() {
                state.set_current_model(selected_model.clone());
                state.add_message(
                    MessageRole::System,
                    format!("Model changed to: {}", selected_model),
                );
                state.input_mode = InputMode::Normal;
                // TODO: Send model change event to worker
            }
            false
        }
        _ => false,
    }
}
