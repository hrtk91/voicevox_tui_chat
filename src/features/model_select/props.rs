use crate::features::chat::theme::ChatTheme;

#[derive(Debug)]
pub struct ModelSelectProps<'a> {
    pub available_models: &'a [String],
    pub current_model: &'a str,
    pub selected_index: usize,
    pub theme: &'a ChatTheme,
}
