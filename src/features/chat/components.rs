use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::props::{ChatScreenProps, InputAreaProps};
use super::state::{AppState, InputMode, MessageRole};
use crate::features::shared::text_utils::{
    calculate_input_height, calculate_multiline_cursor_position, wrap_text,
};

pub fn render_ui(frame: &mut Frame, state: &AppState) {
    match state.input_mode {
        InputMode::ModelSelect => {
            crate::features::model_select::component::render_model_select_screen(
                frame,
                &crate::features::model_select::props::ModelSelectProps {
                    available_models: &state.available_models,
                    current_model: &state.current_model,
                    selected_index: state.model_select_index,
                    theme: &state.theme,
                },
            );
        }
        InputMode::Settings => {
            crate::features::settings::component::render_settings_screen(
                frame,
                &crate::features::settings::props::SettingsScreenProps {
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
