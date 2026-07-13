use super::*;

#[test]
fn gui_theme_palettes_cover_existing_presets() {
    let pastel = gui_theme_palette(EditorThemeId::Paper);
    assert_eq!(pastel.background, color(245, 226, 244));
    assert_eq!(pastel.text, color(34, 24, 48));

    let terminal = gui_theme_palette(EditorThemeId::Terminal);
    assert_eq!(terminal.background, color(0, 18, 7));
    assert_eq!(terminal.primary, color(72, 255, 112));

    let abyss = gui_theme_palette(EditorThemeId::Abyss);
    assert_eq!(abyss.background, color(3, 7, 18));
    assert_eq!(abyss.danger, color(255, 64, 96));

    let terror = gui_theme_palette(EditorThemeId::Terror);
    assert_eq!(terror.background, color(24, 0, 30));
    assert_eq!(terror.primary, color(255, 42, 160));

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        assert_eq!(gui_theme(theme_id).palette(), gui_theme_palette(theme_id));
    }
}

#[test]
fn gui_pastel_syntax_colors_are_darkened_for_readability() {
    let pale = SyntaxColor {
        r: 220,
        g: 226,
        b: 232,
        a: 255,
    };
    let normal = gui_color_from_syntax(pale, EditorThemeId::Nocturne);
    let pastel = gui_color_from_syntax(pale, EditorThemeId::Paper);

    assert_eq!(normal, Color::from_rgb8(213, 224, 246));
    assert!(pastel.r < normal.r);
    assert!(pastel.g < normal.g);
    assert!(pastel.b < normal.b);
    assert_eq!(pastel, Color::from_rgb8(80, 67, 91));
}

#[test]
fn gui_syntax_theme_colors_keep_readable_contrast() {
    let samples = [
        (220, 226, 232),
        (190, 90, 120),
        (90, 170, 130),
        (120, 140, 230),
    ];

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        let background = gui_color_to_rgb(gui_theme_palette(theme_id).background);
        for (red, green, blue) in samples {
            let foreground = gui_syntax_rgb_for_theme(red, green, blue, theme_id);
            assert!(
                gui_contrast_ratio(foreground, background) >= 4.5,
                "{theme_id:?} foreground {foreground:?} lacks readable contrast on {background:?}"
            );
        }
    }
}

#[test]
fn gui_syntax_theme_role_palettes_stay_varied() {
    let samples = [
        (220, 226, 232),
        (100, 110, 120),
        (220, 80, 120),
        (220, 130, 70),
        (210, 190, 70),
        (80, 190, 120),
        (70, 190, 210),
        (80, 120, 220),
        (170, 95, 230),
    ];

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        let colors = samples
            .into_iter()
            .map(|(red, green, blue)| gui_syntax_rgb_for_theme(red, green, blue, theme_id))
            .collect::<HashSet<_>>();

        assert!(
            colors.len() >= 7,
            "{theme_id:?} syntax palette collapsed into {colors:?}"
        );
    }
}

#[test]
#[cfg(feature = "syntax")]
fn gui_highlighter_uses_shared_syntax_tokens_and_preset_theme_mapping() {
    let temp = TempArea::new("gui-syntax-highlight");
    let rust_path = temp.path("main.rs");
    let text_path = temp.path("note.txt");
    fs::write(&rust_path, "fn main() {}\n").expect("write rust");
    fs::write(&text_path, "plain\n").expect("write text");
    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![rust_path.clone(), text_path.clone()],
    });
    let rust_tile = state
        .workspace
        .tiles
        .iter()
        .find(|tile| tile.document.path == rust_path)
        .expect("rust tile");
    let text_tile = state
        .workspace
        .tiles
        .iter()
        .find(|tile| tile.document.path == text_path)
        .expect("text tile");

    assert_eq!(
        state
            .syntax_highlighter
            .syntax_token_for_document(&rust_tile.document),
        "rs"
    );
    assert_eq!(
        state
            .syntax_highlighter
            .syntax_token_for_document(&text_tile.document),
        "txt"
    );
    assert_eq!(
        gui_highlighter_theme(EditorThemeId::Paper),
        highlighter::Theme::InspiredGitHub
    );
    assert_eq!(
        gui_highlighter_theme(EditorThemeId::Terror),
        highlighter::Theme::Base16Eighties
    );
}

#[test]
fn gui_theme_cycle_updates_status_and_persists_existing_config_format() {
    let temp = TempArea::new("gui-theme-cycle");
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

    let _ = update(&mut state, Message::CycleTheme);

    assert_eq!(state.settings.theme_id, EditorThemeId::Abyss);
    assert_eq!(state.status_message, "theme: abyss");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
#[cfg(feature = "syntax")]
fn gui_syntax_theme_cycles_separately_from_app_theme_and_persists() {
    let temp = TempArea::new("gui-syntax-theme-cycle");
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
        syntax_theme_id: EditorThemeId::Nocturne,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleSyntaxTheme);

    assert_eq!(state.settings.theme_id, EditorThemeId::Terminal);
    assert_eq!(state.settings.syntax_theme_id, EditorThemeId::Aurora);
    assert_eq!(state.status_message, "syntax theme: aurora");
    let saved = fs::read_to_string(&config).expect("read config");
    assert!(saved.contains("theme = \"terminal\"\n"));
    assert!(saved.contains("syntax_theme = \"aurora\"\n"));
}

#[test]
#[cfg(not(feature = "syntax"))]
fn gui_syntax_theme_action_reports_unavailable_in_lean_build() {
    let temp = TempArea::new("gui-syntax-unavailable");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let original_theme = state.settings.syntax_theme_id;

    let _ = update(&mut state, Message::CycleSyntaxTheme);

    assert_eq!(state.settings.syntax_theme_id, original_theme);
    assert_eq!(
        state.status_message,
        "syntax highlighting unavailable in this build"
    );
}
