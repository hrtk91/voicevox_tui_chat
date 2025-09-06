use uuid::Uuid;

use super::theme::{ChatTheme, ThemePreset};
use std::collections::HashMap;

pub type MessageId = String;
pub type Content = String;
pub type ErrorMessage = String;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    ModelSelect,
    Settings,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: Content,
    pub is_streaming: bool,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: Content) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            is_streaming: false,
        }
    }

    pub fn new_streaming(role: MessageRole, initial_content: Content) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: initial_content,
            is_streaming: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            MessageRole::User => "You",
            MessageRole::Assistant => "AI",
            MessageRole::System => "System",
        }
    }

    pub fn formatted_prefix(&self, max_width: usize) -> String {
        format!("{:>width$}: ", self.display_name(), width = max_width)
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub messages: Vec<ChatMessage>,
    pub current_input: String,
    pub cursor_position: usize,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub input_mode: InputMode,
    pub theme: ChatTheme,
    pub auto_scroll_enabled: bool,
    pub current_model: String,
    pub available_models: Vec<String>,
    pub model_select_index: usize,
    pub current_settings: HashMap<String, String>,
    pub settings_scroll_index: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_input: String::new(),
            cursor_position: 0,
            should_quit: false,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            theme: ChatTheme::from_preset(ThemePreset::Default),
            auto_scroll_enabled: true,
            current_model: "gpt-5-nano".to_string(),
            available_models: vec![
                "gpt-5".to_string(),
                "gpt-5-mini".to_string(),
                "gpt-5-nano".to_string(),
            ],
            model_select_index: 2, // Default to gpt-5-nano
            current_settings: HashMap::new(),
            settings_scroll_index: 0,
        }
    }

    pub fn set_current_model(&mut self, model: String) {
        self.current_model = model.clone();
        // Update selected index to match current model
        if let Some(index) = self.available_models.iter().position(|m| m == &model) {
            self.model_select_index = index;
        }
    }

    pub fn move_model_selection_up(&mut self) {
        if self.model_select_index > 0 {
            self.model_select_index -= 1;
        }
    }

    pub fn move_model_selection_down(&mut self) {
        if self.model_select_index < self.available_models.len().saturating_sub(1) {
            self.model_select_index += 1;
        }
    }

    pub fn get_selected_model(&self) -> Option<&String> {
        self.available_models.get(self.model_select_index)
    }

    pub fn update_settings(&mut self, settings: HashMap<String, String>) {
        self.current_settings = settings;
    }

    pub fn move_settings_selection_up(&mut self) {
        if self.settings_scroll_index > 0 {
            self.settings_scroll_index -= 1;
        }
    }

    pub fn move_settings_selection_down(&mut self, max_items: usize) {
        if self.settings_scroll_index < max_items.saturating_sub(1) {
            self.settings_scroll_index += 1;
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: Content) -> MessageId {
        let message = ChatMessage::new(role, content);
        let id = message.id.clone();
        self.messages.push(message);
        // 自動スクロールは呼び出し側で適切なdisplay_widthと共に呼び出す
        id
    }

    pub fn start_streaming_message(
        &mut self,
        role: MessageRole,
        initial_content: Content,
    ) -> MessageId {
        let message = ChatMessage::new_streaming(role, initial_content);
        let id = message.id.clone();
        self.messages.push(message);
        // 自動スクロールは呼び出し側で適切なdisplay_widthと共に呼び出す
        id
    }

    /// 自動スクロール（最下部へ） - 有効時のみ実行
    pub fn auto_scroll_to_bottom(&mut self, display_width: usize) {
        if self.auto_scroll_enabled {
            let total_lines = self.get_total_lines(display_width);
            if total_lines > 0 {
                self.scroll_offset = total_lines - 1;
            }
        }
    }

    /// 手動スクロール実行 - 自動スクロールを無効化
    pub fn disable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = false;
    }

    /// 自動スクロールを再有効化
    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = true;
    }

    pub fn append_to_message(&mut self, message_id: &MessageId, content: &Content) -> bool {
        if let Some(message) = self.find_message_mut(message_id) {
            message.content.push_str(content);
            true
        } else {
            false
        }
    }

    pub fn finish_streaming_message(&mut self, message_id: &MessageId) -> bool {
        if let Some(message) = self.find_message_mut(message_id) {
            message.is_streaming = false;
            true
        } else {
            false
        }
    }

    pub fn find_message_mut(&mut self, id: &MessageId) -> Option<&mut ChatMessage> {
        self.messages.iter_mut().find(|msg| msg.id == *id)
    }

    pub fn scroll_up(&mut self, _display_width: usize) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, display_width: usize) {
        let total_lines = self.get_total_lines(display_width);
        if total_lines > 0 && self.scroll_offset + 1 < total_lines {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_top(&mut self, _display_width: usize) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self, display_width: usize) {
        let total_lines = self.get_total_lines(display_width);
        if total_lines > 0 {
            self.scroll_offset = total_lines - 1;
        }
    }

    pub fn get_visible_messages(&self, visible_height: usize) -> Vec<&ChatMessage> {
        let start = self.scroll_offset;
        let end = (start + visible_height).min(self.messages.len());
        self.messages[start..end].iter().collect()
    }

    pub fn max_prefix_width(&self) -> usize {
        // 全てのロール名の最大文字数を計算
        [
            MessageRole::User.display_name(),
            MessageRole::Assistant.display_name(),
            MessageRole::System.display_name(),
        ]
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
    }

    /// 単一段落の行数を計算する
    fn calculate_paragraph_lines(&self, paragraph: &str, width: usize) -> usize {
        if width == 0 {
            return 1;
        }

        let mut lines = 0;
        let mut current_width = 0;

        for ch in paragraph.chars() {
            let char_width = if ch.is_ascii() { 1 } else { 2 };

            if current_width + char_width > width && current_width > 0 {
                lines += 1;
                current_width = char_width;
            } else {
                current_width += char_width;
            }
        }

        if current_width > 0 {
            lines += 1;
        }

        lines.max(1)
    }

    /// テキストを改行コードで分割し、折り返し後の総行数を計算する
    fn calculate_wrapped_lines(&self, text: &str, width: usize) -> usize {
        let mut total_lines = 0;

        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                // 空行も1行としてカウント
                total_lines += 1;
            } else {
                total_lines += self.calculate_paragraph_lines(paragraph, width);
            }
        }

        total_lines.max(1)
    }

    /// 指定された表示幅での総行数を計算する
    pub fn get_total_lines(&self, display_width: usize) -> usize {
        let max_prefix_width = self.max_prefix_width();
        let text_width = display_width.saturating_sub(max_prefix_width + 2);

        self.messages
            .iter()
            .map(|msg| self.calculate_wrapped_lines(&msg.content, text_width))
            .sum()
    }

    /// 入力内容とカーソル位置をクリア
    pub fn clear_input(&mut self) {
        self.current_input.clear();
        self.cursor_position = 0;
    }

    /// カーソル位置を左に移動
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            // 文字境界を考慮してカーソルを移動
            let mut chars: Vec<(usize, char)> = self.current_input.char_indices().collect();
            chars.reverse();

            for (idx, _) in chars {
                if idx < self.cursor_position {
                    self.cursor_position = idx;
                    break;
                }
            }
        }
    }

    /// カーソル位置を右に移動
    pub fn move_cursor_right(&mut self) {
        let chars: Vec<(usize, char)> = self.current_input.char_indices().collect();

        for (idx, _) in chars {
            if idx > self.cursor_position {
                self.cursor_position = idx;
                return;
            }
        }

        // 最後の文字より後ろに移動する場合
        self.cursor_position = self.current_input.len();
    }

    /// カーソル位置に文字を挿入
    pub fn insert_char_at_cursor(&mut self, ch: char) {
        self.current_input.insert(self.cursor_position, ch);
        self.cursor_position += ch.len_utf8();
    }

    /// カーソル位置の前の文字を削除
    pub fn backspace_at_cursor(&mut self) {
        if self.cursor_position > 0 {
            let chars: Vec<(usize, char)> = self.current_input.char_indices().collect();

            // カーソル位置より前の文字を見つける
            for (idx, _ch) in chars.iter().rev() {
                if *idx < self.cursor_position {
                    self.current_input.remove(*idx);
                    self.cursor_position = *idx;
                    break;
                }
            }
        }
    }
}
