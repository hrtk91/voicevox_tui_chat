use ratatui::{
    style::Style,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::props::SettingsScreenProps;

pub fn render_settings_screen(frame: &mut Frame, props: &SettingsScreenProps) {
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
        Block::default().borders(Borders::ALL).title("Settings"),
        settings_area,
    );

    let inner_area = ratatui::layout::Rect {
        x: settings_area.x + 1,
        y: settings_area.y + 1,
        width: settings_area.width - 2,
        height: settings_area.height - 2,
    };

    // Convert HashMap to sorted vector for consistent display
    let mut settings_items: Vec<(String, String)> = props
        .settings
        .iter()
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
