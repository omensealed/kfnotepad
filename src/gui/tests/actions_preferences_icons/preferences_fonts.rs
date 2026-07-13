use super::*;

#[test]
fn gui_preferences_panel_toggles_line_numbers_and_wrap_in_config() {
    let temp = TempArea::new("gui-preferences-toggle");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

    assert!(!state.settings.show_line_numbers);
    assert_eq!(state.status_message, "line numbers: off");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::WrapLinesChanged(true));

    assert!(state.settings.wrap_lines);
    assert_eq!(state.status_message, "wrap text: on");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
fn gui_preferences_panel_cycles_font_family_and_size_in_config() {
    let temp = TempArea::new("gui-preferences-font");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleGuiFontFamily);

    assert_eq!(state.settings.gui_font_family, GuiFontFamily::SansSerif);
    assert_eq!(state.status_message, "font: Sans serif");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::GuiFontSizeChanged(20));

    assert_eq!(state.settings.gui_font_size, 20);
    assert_eq!(
        state.settings.gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(state.status_message, "editor font size: 20");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::GuiUiFontSizeChanged(18));

    assert_eq!(state.settings.gui_font_size, 20);
    assert_eq!(state.settings.gui_ui_font_size, 18);
    assert_eq!(state.status_message, "ui font size: 18");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 18\n"
        );
}

#[test]
fn gui_preferences_panel_rejects_out_of_range_font_size() {
    let temp = TempArea::new("gui-preferences-font-size-bounds");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(
        &mut state,
        Message::GuiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
    );

    assert_eq!(state.settings.gui_font_size, DEFAULT_GUI_FONT_SIZE);
    assert_eq!(state.status_message, "editor font size must be 10-32");
    assert!(!config.exists());

    let _ = update(
        &mut state,
        Message::GuiUiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
    );

    assert_eq!(
        state.settings.gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(state.status_message, "ui font size must be 10-32");
    assert!(!config.exists());
}

#[test]
fn gui_preferences_panel_toggle_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-preferences-toggle-failure");
    let blocked_parent = temp.path("blocked");
    fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(blocked_parent.join("config.toml"));
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

    assert!(state.settings.show_line_numbers);
    assert!(!state.settings.wrap_lines);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}

#[test]
fn gui_preferences_panel_font_change_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-preferences-font-failure");
    let blocked_parent = temp.path("blocked");
    fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(blocked_parent.join("config.toml"));
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleGuiFontFamily);

    assert_eq!(state.settings.gui_font_family, GuiFontFamily::Monospace);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}

#[test]
fn gui_editor_font_maps_font_presets() {
    assert_eq!(gui_editor_font(GuiFontFamily::Monospace), Font::MONOSPACE);
    assert_eq!(gui_editor_font(GuiFontFamily::SansSerif), Font::DEFAULT);
    assert_eq!(
        gui_editor_font(GuiFontFamily::Serif).family,
        iced::font::Family::Serif
    );
    assert_eq!(
        gui_editor_font(GuiFontFamily::JetBrainsMono),
        Font::with_name("JetBrains Mono")
    );
    assert_eq!(
        gui_editor_font(GuiFontFamily::FiraCode),
        Font::with_name("Fira Code")
    );
}

#[test]
fn gui_ui_font_size_helpers_use_separate_chrome_setting() {
    let settings = EditorSettings {
        gui_font_size: 30,
        gui_ui_font_size: 18,
        ..EditorSettings::default()
    };

    assert_eq!(
        EditorSettings::default().gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(gui_ui_text_size(settings), 18);
    assert_eq!(gui_ui_small_text_size(settings), 16);
    assert_eq!(gui_ui_heading_text_size(settings), 22);
    assert_eq!(gui_ui_icon_text_size(settings), 19);
    assert_eq!(gui_ui_tooltip_text_size(settings), 16);
}
