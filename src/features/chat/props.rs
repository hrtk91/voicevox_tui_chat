use crate::features::chat::state::{ChatMessage, InputMode};
use crate::features::chat::theme::ChatTheme;

#[derive(Debug)]
pub struct ChatScreenProps<'a> {
    pub messages: &'a [ChatMessage],
    pub theme: &'a ChatTheme,
    pub scroll_offset: usize,
    pub auto_scroll_enabled: bool,
}

#[derive(Debug)]
pub struct InputAreaProps<'a> {
    pub current_input: &'a str,
    pub cursor_position: usize,
    pub input_mode: InputMode,
    pub theme: &'a ChatTheme,
}
