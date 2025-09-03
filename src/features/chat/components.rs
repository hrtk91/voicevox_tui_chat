use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::state::{AppState, InputMode};

/// 文字列の表示幅を計算する（日本語文字対応）
fn calculate_display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

/// 単一段落を指定幅で折り返し、複数行に分割する
fn wrap_paragraph(paragraph: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![paragraph.to_string()];
    }
    
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    
    for ch in paragraph.chars() {
        let char_width = if ch.is_ascii() { 1 } else { 2 }; // 日本語文字は幅2として計算
        
        if current_width + char_width > width && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line.clear();
            current_width = 0;
        }
        
        current_line.push(ch);
        current_width += char_width;
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

/// テキストを改行コードで分割し、各段落を指定幅で折り返し、複数行に分割する
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut result = Vec::new();
    
    // まず改行文字で分割
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            // 空行も保持
            result.push(String::new());
        } else {
            // 各段落を幅で折り返し
            let wrapped_lines = wrap_paragraph(paragraph, width);
            result.extend(wrapped_lines);
        }
    }
    
    if result.is_empty() {
        result.push(String::new());
    }
    
    result
}

pub fn render_ui(frame: &mut Frame, state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    render_message_list(frame, state, main_layout[0]);
    render_input_area(frame, state, main_layout[1]);
}

fn render_message_list(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let max_prefix_width = state.max_prefix_width();
    
    // ボーダーを考慮した実際の表示幅を計算
    let content_width = area.width.saturating_sub(2) as usize; // 左右のボーダー分
    let text_width = content_width.saturating_sub(max_prefix_width + 2); // プレフィックスとスペース分
    
    let mut all_lines: Vec<ListItem> = Vec::new();
    
    for msg in &state.messages {
        let style = state.theme.get_message_style(&msg.role);
        let prefix = msg.role.formatted_prefix(max_prefix_width);
        
        // メッセージ内容を指定幅で折り返し
        let wrapped_lines = wrap_text(&msg.content, text_width);
        
        for (i, line_content) in wrapped_lines.iter().enumerate() {
            let line_text = if i == 0 {
                // 最初の行にはプレフィックスを付ける
                format!("{}{}", prefix, line_content)
            } else {
                // 2行目以降は適切なインデントを追加
                format!("{}{}", " ".repeat(max_prefix_width + 2), line_content)
            };
            
            all_lines.push(ListItem::new(Line::from(Span::styled(line_text, style))));
        }
    }

    let mut list_state = ListState::default();
    let total_lines = all_lines.len();

    let messages_list = List::new(all_lines)
        .block(Block::default().borders(Borders::ALL).title("Chat History"))
        .highlight_style(state.theme.get_highlight_style());
    
    if total_lines > 0 {
        // スクロールオフセットを行単位で適用
        let selected_index = state.scroll_offset.min(total_lines.saturating_sub(1));
        list_state.select(Some(selected_index));
    }

    frame.render_stateful_widget(messages_list, area, &mut list_state);
}

fn render_input_area(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let (mode_text, help_text) = match state.input_mode {
        InputMode::Normal => (
            "-- NORMAL --",
            "i:Insert q:Quit j/k:Scroll g/G:Top/Bottom",
        ),
        InputMode::Insert => (
            "-- INSERT --",
            "Esc:Normal Enter:Send",
        ),
    };

    let border_color = state.theme.get_border_color(state.input_mode == InputMode::Insert);
    let title = format!("{} ({})", mode_text, help_text);

    let input_paragraph = Paragraph::new(state.current_input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );

    frame.render_widget(input_paragraph, area);

    // Insertモード時にカーソル位置を設定
    if state.input_mode == InputMode::Insert {
        let text_width = calculate_display_width(&state.current_input);
        let cursor_x = area.x + 1 + text_width as u16; // ボーダー + テキスト幅
        let cursor_y = area.y + 1; // ボーダー内の1行目
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}