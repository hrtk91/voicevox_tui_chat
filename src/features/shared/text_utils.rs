/// 文字列の表示幅を計算する（日本語文字対応）
pub fn calculate_display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

/// 複数行テキストでのカーソル位置を計算する
pub fn calculate_multiline_cursor_position(
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
pub fn wrap_paragraph(paragraph: &str, width: usize) -> Vec<String> {
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
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
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
