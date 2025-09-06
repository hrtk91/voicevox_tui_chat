use ratatui::{
    style::Style,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::props::ModelSelectProps;

pub fn render_model_select_screen(frame: &mut Frame, props: &ModelSelectProps) {
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
