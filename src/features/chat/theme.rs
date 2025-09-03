use ratatui::style::{Color, Style};

use super::state::MessageRole;

#[derive(Debug, Clone)]
pub struct ChatTheme {
    pub user_color: Color,
    pub assistant_color: Color,
    pub system_color: Color,
    pub highlight_bg: Color,
    pub normal_border: Color,
    pub insert_border: Color,
}

#[derive(Debug, Clone)]
pub enum ThemePreset {
    Default,
    Dark,
    Light,
}

impl ChatTheme {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Default => Self {
                user_color: Color::Cyan,
                assistant_color: Color::Green,
                system_color: Color::Yellow,
                highlight_bg: Color::Rgb(40, 40, 40),
                normal_border: Color::Blue,
                insert_border: Color::Green,
            },
            ThemePreset::Dark => Self {
                user_color: Color::LightBlue,
                assistant_color: Color::LightGreen,
                system_color: Color::LightYellow,
                highlight_bg: Color::Rgb(60, 60, 60),
                normal_border: Color::Blue,
                insert_border: Color::Green,
            },
            ThemePreset::Light => Self {
                user_color: Color::Blue,
                assistant_color: Color::Rgb(0, 150, 0),
                system_color: Color::Red,
                highlight_bg: Color::Rgb(240, 240, 220),
                normal_border: Color::Rgb(0, 0, 150),
                insert_border: Color::Rgb(0, 120, 0),
            },
        }
    }

    pub fn get_message_style(&self, role: &MessageRole) -> Style {
        let fg_color = match role {
            MessageRole::User => self.user_color,
            MessageRole::Assistant => self.assistant_color,
            MessageRole::System => self.system_color,
        };
        Style::default().fg(fg_color)
    }

    pub fn get_highlight_style(&self) -> Style {
        Style::default().bg(self.highlight_bg)
    }

    pub fn get_border_color(&self, is_insert_mode: bool) -> Color {
        if is_insert_mode {
            self.insert_border
        } else {
            self.normal_border
        }
    }
}