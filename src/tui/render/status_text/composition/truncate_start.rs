fn fit_text_start(text: &str, width: usize) -> String {
    let width = width.max(1);
    if text_display_width(text) <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }

    let mut prefix = String::new();
    let mut used_columns = 1;
    for character in text.chars() {
        let character_width = character_display_width(character, used_columns);
        if character_width > 0 && used_columns + character_width > width {
            break;
        }
        prefix.push(character);
        used_columns += character_width;
    }
    format!("{prefix}…")
}
