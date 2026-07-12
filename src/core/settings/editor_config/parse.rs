//! Tolerant editor configuration parsing with bounded numeric values.

use super::*;

pub fn parse_editor_settings_config(text: &str) -> EditorSettings {
    let mut settings = EditorSettings::default();

    for line in text.lines() {
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value
            .split_once('#')
            .map_or(raw_value, |(value, _)| value)
            .trim();

        match key {
            "theme" => {
                if let Some(theme_id) =
                    parse_config_string(value).and_then(EditorThemeId::from_label)
                {
                    settings.theme_id = theme_id;
                }
            }
            "syntax_theme" => {
                if let Some(theme_id) =
                    parse_config_string(value).and_then(EditorThemeId::from_label)
                {
                    settings.syntax_theme_id = theme_id;
                }
            }
            "line_numbers" => {
                if let Some(show_line_numbers) = parse_config_bool(value) {
                    settings.show_line_numbers = show_line_numbers;
                }
            }
            "wrap" => {
                if let Some(wrap_lines) = parse_config_bool(value) {
                    settings.wrap_lines = wrap_lines;
                }
            }
            "search_case_sensitive" => {
                if let Some(search_case_sensitive) = parse_config_bool(value) {
                    settings.search_case_sensitive = search_case_sensitive;
                }
            }
            "gui_restore_last_workspace" => {
                if let Some(gui_restore_last_workspace) = parse_config_bool(value) {
                    settings.gui_restore_last_workspace = gui_restore_last_workspace;
                }
            }
            "gui_reader_mode_enabled" => {
                if let Some(gui_reader_mode_enabled) = parse_config_bool(value) {
                    settings.gui_reader_mode_enabled = gui_reader_mode_enabled;
                }
            }
            "gui_reader_lines_per_minute" => {
                if let Ok(lines_per_minute) = value.parse::<u16>() {
                    if (MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE)
                        .contains(&lines_per_minute)
                    {
                        settings.gui_reader_lines_per_minute = lines_per_minute;
                    }
                }
            }
            "gui_font_family" => {
                if let Some(gui_font_family) =
                    parse_config_string(value).and_then(GuiFontFamily::from_label)
                {
                    settings.gui_font_family = gui_font_family;
                }
            }
            "gui_font_size" => {
                if let Ok(gui_font_size) = value.parse::<u16>() {
                    if (MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&gui_font_size) {
                        settings.gui_font_size = gui_font_size;
                    }
                }
            }
            "gui_ui_font_size" => {
                if let Ok(gui_ui_font_size) = value.parse::<u16>() {
                    if (MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&gui_ui_font_size) {
                        settings.gui_ui_font_size = gui_ui_font_size;
                    }
                }
            }
            _ => {}
        }
    }

    settings
}
