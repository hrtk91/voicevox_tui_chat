use uuid::Uuid;

use super::theme::{ChatTheme, ThemePreset};

pub type MessageId = String;
pub type Content = String;
pub type ErrorMessage = String;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
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

#[derive(Debug, Clone)]
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
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub input_mode: InputMode,
    pub theme: ChatTheme,
    pub auto_scroll_enabled: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_input: String::new(),
            should_quit: false,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            theme: ChatTheme::from_preset(ThemePreset::Default),
            auto_scroll_enabled: true,
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
}
