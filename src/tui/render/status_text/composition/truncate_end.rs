pub(crate) fn fit_text_end(text: &str, width: usize) -> String {
    let width = width.max(1);
    if text_display_width(text) <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }

    let mut suffix = String::new();
    let mut used_columns = 1;
    for character in text.chars().rev() {
        let character_width = character_display_width(character, used_columns);
        if character_width > 0 && used_columns + character_width > width {
            break;
        }
        suffix.insert(0, character);
        used_columns += character_width;
    }
    format!("…{suffix}")
}
