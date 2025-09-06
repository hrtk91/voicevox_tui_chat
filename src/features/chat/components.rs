use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::state::{AppState, ChatMessage, InputMode, MessageRole};
use super::theme::ChatTheme;
use std::collections::HashMap;

// Screen-specific props structures
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

#[derive(Debug)]
pub struct ModelSelectProps<'a> {
    pub available_models: &'a [String],
    pub current_model: &'a str,
    pub selected_index: usize,
    pub theme: &'a ChatTheme,
}

#[derive(Debug)]
pub struct SettingsScreenProps<'a> {
    pub settings: &'a HashMap<String, String>,
    pub selected_index: usize,
    pub theme: &'a ChatTheme,
}

/// 文字列の表示幅を計算する（日本語文字対応）
fn calculate_display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

/// 複数行テキストでのカーソル位置を計算する
fn calculate_multiline_cursor_position(
    text: &str,
    cursor_position: usize,
    area: ratatui::layout::Rect,
) -> (u16, u16) {
    // カーソル位置までのテキストを取得
    let cursor_text = if cursor_position <= text.len() {
        &text[..cursor_position]
    } else {
        text
    };

    let lines: Vec<&str> = cursor_text.split('\n').collect();
    let current_line = lines.last().map_or("", |line| line);
    let line_count = lines.len();

    let cursor_x = area.x + 1 + calculate_display_width(current_line) as u16; // ボーダー + 現在行のテキスト幅
    let cursor_y = area.y + line_count as u16; // ボーダー + 行数

    (cursor_x, cursor_y)
}

/// 入力テキストに必要な高さを計算する（ボーダー込み）
pub fn calculate_input_height(text: &str, width: usize) -> usize {
    let content_width = width.saturating_sub(2); // 左右ボーダー分を除外
    let mut total_lines = 0;

    for line in text.split('\n') {
        if line.is_empty() {
            total_lines += 1;
        } else {
            // 長い行は折り返しを考慮
            let wrapped_lines = wrap_paragraph(line, content_width);
            total_lines += wrapped_lines.len();
        }
    }

    // 最低3行（ボーダー + 内容1行 + ボーダー）、最大10行に制限

    (total_lines + 2).clamp(3, 10)
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
    match state.input_mode {
        InputMode::ModelSelect => {
            render_model_select_screen(
                frame,
                &ModelSelectProps {
                    available_models: &state.available_models,
                    current_model: &state.current_model,
                    selected_index: state.model_select_index,
                    theme: &state.theme,
                },
            );
        }
        InputMode::Settings => {
            render_settings_screen(
                frame,
                &SettingsScreenProps {
                    settings: &state.current_settings,
                    selected_index: state.settings_scroll_index,
                    theme: &state.theme,
                },
            );
        }
        _ => {
            // 入力内容に応じて動的に入力エリアの高さを計算
            let input_height =
                calculate_input_height(&state.current_input, frame.area().width as usize);

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(input_height as u16)])
                .split(frame.area());

            render_message_list(
                frame,
                &ChatScreenProps {
                    messages: &state.messages,
                    theme: &state.theme,
                    scroll_offset: state.scroll_offset,
                    auto_scroll_enabled: state.auto_scroll_enabled,
                },
                main_layout[0],
            );

            render_input_area(
                frame,
                &InputAreaProps {
                    current_input: &state.current_input,
                    cursor_position: state.cursor_position,
                    input_mode: state.input_mode,
                    theme: &state.theme,
                },
                main_layout[1],
            );
        }
    }
}

fn render_message_list(frame: &mut Frame, props: &ChatScreenProps, area: ratatui::layout::Rect) {
    // Calculate max prefix width from available message roles
    let max_prefix_width = [
        MessageRole::User.display_name(),
        MessageRole::Assistant.display_name(),
        MessageRole::System.display_name(),
    ]
    .iter()
    .map(|s| s.len())
    .max()
    .unwrap_or(0);

    // ボーダーを考慮した実際の表示幅を計算
    let content_width = area.width.saturating_sub(2) as usize; // 左右のボーダー分
    let text_width = content_width.saturating_sub(max_prefix_width + 2); // プレフィックスとスペース分

    let mut all_lines: Vec<ListItem> = Vec::new();

    for msg in props.messages {
        let style = props.theme.get_message_style(&msg.role);
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
        .highlight_style(props.theme.get_highlight_style());

    if total_lines > 0 {
        // スクロールオフセットを行単位で適用
        let selected_index = props.scroll_offset.min(total_lines.saturating_sub(1));
        list_state.select(Some(selected_index));
    }

    frame.render_stateful_widget(messages_list, area, &mut list_state);
}

fn render_input_area(frame: &mut Frame, props: &InputAreaProps, area: ratatui::layout::Rect) {
    let (mode_text, help_text) = match props.input_mode {
        InputMode::Normal => (
            "-- NORMAL --",
            "i:Insert m:Model s:Settings q:Quit j/k:Scroll g/G:Top/Bottom",
        ),
        InputMode::Insert => (
            "-- INSERT --",
            "Esc:Normal Enter:Send /model:ModelSelect Ctrl+N:NewLine",
        ),
        InputMode::ModelSelect => ("-- MODEL SELECT --", "j/k:Navigate Enter:Select Esc:Cancel"),
        InputMode::Settings => ("-- SETTINGS --", "j/k:Scroll Esc:Back q:Quit"),
    };

    let border_color = props
        .theme
        .get_border_color(props.input_mode == InputMode::Insert);
    let title = format!("{} ({})", mode_text, help_text);

    let input_paragraph = Paragraph::new(props.current_input)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        );

    frame.render_widget(input_paragraph, area);

    // Insertモード時にカーソル位置を設定
    if props.input_mode == InputMode::Insert {
        let (cursor_x, cursor_y) =
            calculate_multiline_cursor_position(props.current_input, props.cursor_position, area);
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_model_select_screen(frame: &mut Frame, props: &ModelSelectProps) {
    let area = frame.area();

    // Create a centered popup
    let popup_area = ratatui::layout::Rect {
        x: area.width / 4,
        y: area.height / 4,
        width: area.width / 2,
        height: area.height / 2,
    };

    // Clear the background
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Select Model"),
        popup_area,
    );

    let inner_area = ratatui::layout::Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let mut items = Vec::new();
    for (i, model) in props.available_models.iter().enumerate() {
        let mut line = model.clone();
        if model == props.current_model {
            line = format!("● {}", line);
        } else {
            line = format!("  {}", line);
        }

        let style = if i == props.selected_index {
            Style::default().bg(ratatui::style::Color::DarkGray)
        } else {
            Style::default()
        };

        items.push(ListItem::new(line).style(style));
    }

    let model_list = List::new(items).block(Block::default().borders(Borders::NONE));

    frame.render_widget(model_list, inner_area);
}

fn render_settings_screen(frame: &mut Frame, props: &SettingsScreenProps) {
    let area = frame.area();
    
    // Use most of the screen for settings
    let settings_area = ratatui::layout::Rect {
        x: area.width / 8,
        y: area.height / 8,
        width: area.width * 3 / 4,
        height: area.height * 3 / 4,
    };

    // Main settings container
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title("Settings"),
        settings_area,
    );

    let inner_area = ratatui::layout::Rect {
        x: settings_area.x + 1,
        y: settings_area.y + 1,
        width: settings_area.width - 2,
        height: settings_area.height - 2,
    };

    // Convert HashMap to sorted vector for consistent display
    let mut settings_items: Vec<(String, String)> = props.settings.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    settings_items.sort_by(|a, b| a.0.cmp(&b.0));

    let mut items = Vec::new();
    for (i, (key, value)) in settings_items.iter().enumerate() {
        let line = format!("{:.<30} {}", key, value);
        
        let style = if i == props.selected_index {
            Style::default().bg(ratatui::style::Color::DarkGray)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(line).style(style));
    }

    let settings_list = List::new(items).block(Block::default().borders(Borders::NONE));

    frame.render_widget(settings_list, inner_area);
}
