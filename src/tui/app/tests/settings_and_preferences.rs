use super::*;
use crate::tui::input::*;

#[test]
fn editor_config_path_prefers_xdg_config_home() {
    let temp = TempArea::new("config-path-xdg");
    let xdg = temp.path("xdg-config");
    let home = temp.path("home");

    let path =
        editor_config_path(Some(xdg.as_path()), Some(home.as_path())).expect("resolve config path");

    assert_eq!(path, xdg.join("kfnotepad").join("config.toml"));
}

#[test]
fn editor_config_path_falls_back_to_home_config() {
    let temp = TempArea::new("config-path-home");
    let home = temp.path("home");

    let path = editor_config_path(None, Some(home.as_path())).expect("resolve config path");

    assert_eq!(
        path,
        home.join(".config").join("kfnotepad").join("config.toml")
    );
}

#[test]
fn editor_config_path_requires_a_base_directory() {
    assert!(editor_config_path(None, None).is_none());
}

#[test]
fn load_editor_settings_uses_defaults_for_missing_config() {
    let temp = TempArea::new("config-missing");
    let path = temp.path("missing").join("config.toml");

    let settings = load_editor_settings(&path).expect("missing config should use defaults");

    assert_eq!(settings, EditorSettings::default());
}

#[test]
fn parse_editor_settings_config_reads_known_keys_and_ignores_bad_values() {
    let settings = parse_editor_settings_config(
        r#"
theme = "terror"
syntax_theme = "abyss"
line_numbers = false
wrap = true
search_case_sensitive = true
gui_restore_last_workspace = true
gui_reader_mode_enabled = true
gui_reader_lines_per_minute = 180
gui_font_family = "fira-code"
gui_font_size = 20
gui_ui_font_size = 13
unknown = "ignored"
"#,
    );

    assert_eq!(
        settings,
        EditorSettings {
            show_line_numbers: false,
            theme_id: EditorThemeId::Terror,
            syntax_theme_id: EditorThemeId::Abyss,
            wrap_lines: true,
            search_case_sensitive: true,
            gui_restore_last_workspace: true,
            gui_reader_mode_enabled: true,
            gui_reader_lines_per_minute: 180,
            gui_font_family: GuiFontFamily::FiraCode,
            gui_font_size: 20,
            gui_ui_font_size: 13,
        }
    );

    let fallback = parse_editor_settings_config(
        r#"
theme = "not-a-theme"
line_numbers = maybe
wrap = "true"
gui_restore_last_workspace = "true"
gui_font_family = "papyrus"
gui_font_size = 500
gui_ui_font_size = 500
"#,
    );

    assert_eq!(fallback, EditorSettings::default());
}

#[test]
fn save_editor_settings_writes_atomic_private_config() {
    let temp = TempArea::new("config-save");
    let path = temp.path("xdg").join("kfnotepad").join("config.toml");
    let settings = EditorSettings {
        show_line_numbers: false,
        theme_id: EditorThemeId::Abyss,
        wrap_lines: true,
        gui_restore_last_workspace: true,
        gui_font_family: GuiFontFamily::JetBrainsMono,
        gui_font_size: 18,
        gui_ui_font_size: 15,
        ..EditorSettings::default()
    };

    save_editor_settings(&path, settings).expect("save editor config");

    assert_eq!(
            fs::read_to_string(&path).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"jetbrains-mono\"\ngui_font_size = 18\ngui_ui_font_size = 15\n"
        );
    assert_no_temp_files(path.parent().expect("config parent"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let file_mode = fs::metadata(&path)
            .expect("config metadata")
            .permissions()
            .mode()
            & 0o777;
        let dir_mode = fs::metadata(path.parent().expect("config parent"))
            .expect("config dir metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600);
        assert_eq!(dir_mode, 0o700);
    }
}

#[test]
fn preference_controls_persist_runtime_settings_when_configured() {
    let temp = TempArea::new("config-runtime");
    let config_path = temp.path("config").join("kfnotepad").join("config.toml");
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        config_path: Some(config_path.clone()),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)
    ));
    assert_eq!(runtime.settings.theme_id, EditorThemeId::Aurora);
    assert_eq!(runtime.status, "Theme: aurora");
    assert_eq!(
            fs::read_to_string(&config_path).expect("read config"),
            "theme = \"aurora\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
    ));
    assert!(!runtime.settings.show_line_numbers);
    assert!(fs::read_to_string(&config_path)
        .expect("read updated config")
        .contains("line_numbers = false"));

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.settings.wrap_lines);
    assert!(fs::read_to_string(&config_path)
        .expect("read updated config")
        .contains("wrap = true"));
}
