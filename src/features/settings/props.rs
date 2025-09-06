use crate::features::chat::theme::ChatTheme;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SettingsScreenProps<'a> {
    pub settings: &'a HashMap<String, String>,
    pub selected_index: usize,
    pub theme: &'a ChatTheme,
}
