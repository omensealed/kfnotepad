use super::*;

#[test]
fn shared_editor_config_paths_parse_and_persist_without_terminal_types() {
    let temp = TempArea::new("editor-config");
    let xdg = temp.path("xdg");
    let home = temp.path("home");

    assert_eq!(
        editor_config_path(Some(xdg.as_path()), Some(home.as_path())),
        Some(xdg.join("kfnotepad").join("config.toml"))
    );
    assert_eq!(
        editor_config_path(None, Some(home.as_path())),
        Some(home.join(".config").join("kfnotepad").join("config.toml"))
    );
    assert!(editor_config_path(None, None).is_none());

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
gui_restore_last_workspace = yep
gui_font_family = "papyrus"
gui_font_size = 500
gui_ui_font_size = 500
"#,
    );
    assert_eq!(fallback, EditorSettings::default());

    let path = temp.path("config").join("kfnotepad").join("config.toml");
    save_editor_settings(
        &path,
        EditorSettings {
            show_line_numbers: false,
            theme_id: EditorThemeId::Abyss,
            wrap_lines: true,
            gui_restore_last_workspace: true,
            gui_font_family: GuiFontFamily::JetBrainsMono,
            gui_font_size: 18,
            gui_ui_font_size: 15,
            ..EditorSettings::default()
        },
    )
    .expect("save editor config");
    assert_eq!(
            fs::read_to_string(&path).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"jetbrains-mono\"\ngui_font_size = 18\ngui_ui_font_size = 15\n"
        );
    assert_no_temp_files(path.parent().expect("config parent"));
    assert_eq!(
        load_editor_settings(&path).expect("load config"),
        EditorSettings {
            show_line_numbers: false,
            theme_id: EditorThemeId::Abyss,
            wrap_lines: true,
            gui_restore_last_workspace: true,
            gui_font_family: GuiFontFamily::JetBrainsMono,
            gui_font_size: 18,
            gui_ui_font_size: 15,
            ..EditorSettings::default()
        }
    );

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
fn gui_layout_path_parse_and_serialize_round_trip_without_sensitive_paths() {
    let temp = TempArea::new("gui-layout");
    let xdg = temp.path("xdg");
    let home = temp.path("home");

    assert_eq!(
        gui_layout_path(Some(xdg.as_path()), Some(home.as_path())),
        Some(xdg.join("kfnotepad").join("gui-layout.v1"))
    );
    assert_eq!(
        gui_layout_path(None, Some(home.as_path())),
        Some(home.join(".config").join("kfnotepad").join("gui-layout.v1"))
    );
    assert!(gui_layout_path(None, None).is_none());

    let layout = GuiLayout {
        browser_visible: false,
        browser_width_px: Some(260),
        root: GuiLayoutNode::Split {
            axis: GuiLayoutAxis::Vertical,
            ratio_per_mille: 625,
            first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
            second: Box::new(GuiLayoutNode::Split {
                axis: GuiLayoutAxis::Horizontal,
                ratio_per_mille: 400,
                first: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                second: Box::new(GuiLayoutNode::Leaf { ordinal: 2 }),
            }),
        },
        minimized_ordinals: vec![1],
    };

    let text = serialize_gui_layout(&layout);

    assert_eq!(parse_gui_layout(&text, 3), Some(layout));
    assert!(text.contains("browser_width_px = 260"));
    assert!(!text.contains("note.txt"));
    assert!(!text.contains("/home"));
    assert!(!text.contains("search"));
    assert!(!text.contains("cursor"));
}

#[test]
fn gui_layout_parser_falls_back_for_malformed_or_incompatible_input() {
    let valid = r#"
version = 1
browser_visible = true
root = 0
node.0 = split vertical 500 1 2
node.1 = leaf 0
node.2 = leaf 1
minimized =
"#;
    let old_without_width = r#"
version = 1
browser_visible = true
root = 0
node.0 = leaf 0
minimized =
"#;

    assert!(parse_gui_layout(valid, 2).is_some());
    assert_eq!(
        parse_gui_layout(old_without_width, 1)
            .expect("old layout without width should parse")
            .browser_width_px,
        None
    );
    assert!(parse_gui_layout("version = 2\nroot = 0\nnode.0 = leaf 0\n", 1).is_none());
    assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = leaf x\n", 1).is_none());
    assert!(parse_gui_layout(
        "version = 1\nbrowser_width_px = nope\nroot = 0\nnode.0 = leaf 0\n",
        1
    )
    .is_none());
    assert!(parse_gui_layout(
        "version = 1\nbrowser_width_px = 0\nroot = 0\nnode.0 = leaf 0\n",
        1
    )
    .is_none());
    assert!(parse_gui_layout(
        "version = 1\nroot = 0\nnode.0 = split vertical 0 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\n",
        2
    )
    .is_none());
    assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = leaf 0\n", 2).is_none());
    assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = split vertical 500 1 2\nnode.1 = leaf 0\nnode.2 = leaf 0\n", 2).is_none());
    assert!(parse_gui_layout(
        "version = 1\nroot = 0\nnode.0 = leaf 0\nminimized = 0,0\n",
        1
    )
    .is_none());
}

#[test]
fn save_gui_layout_writes_atomic_private_layout_file() {
    let temp = TempArea::new("gui-layout-save");
    let path = temp.path("xdg").join("kfnotepad").join("gui-layout.v1");
    let layout = GuiLayout {
        browser_visible: true,
        browser_width_px: Some(240),
        root: GuiLayoutNode::Leaf { ordinal: 0 },
        minimized_ordinals: Vec::new(),
    };

    save_gui_layout(&path, &layout).expect("save gui layout");

    let text = fs::read_to_string(&path).expect("read gui layout");
    assert_eq!(parse_gui_layout(&text, 1), Some(layout));
    assert_no_temp_files(path.parent().expect("layout parent"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let file_mode = fs::metadata(&path)
            .expect("layout metadata")
            .permissions()
            .mode()
            & 0o777;
        let dir_mode = fs::metadata(path.parent().expect("layout parent"))
            .expect("layout dir metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600);
        assert_eq!(dir_mode, 0o700);
    }
}

#[test]
fn gui_workspace_project_path_and_round_trip_store_files_and_layout() {
    let temp = TempArea::new("gui-workspace-project");
    let xdg = temp.path("xdg");
    let home = temp.path("home");

    assert_eq!(
        gui_workspace_projects_dir(Some(xdg.as_path()), Some(home.as_path())),
        Some(xdg.join("kfnotepad").join("workspaces"))
    );
    assert_eq!(
        gui_workspace_projects_dir(None, Some(home.as_path())),
        Some(home.join(".config").join("kfnotepad").join("workspaces"))
    );
    assert!(gui_workspace_projects_dir(None, None).is_none());

    let project = GuiWorkspaceProject {
        name: "docs workspace".to_string(),
        files: vec![temp.path("README.md"), temp.path("docs/17-GUI-CONTRACT.md")],
        active_ordinal: 1,
        layout: Some(GuiLayout {
            browser_visible: false,
            browser_width_px: Some(220),
            root: GuiLayoutNode::Split {
                axis: GuiLayoutAxis::Vertical,
                ratio_per_mille: 550,
                first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
            },
            minimized_ordinals: vec![0],
        }),
    };

    let text = serialize_gui_workspace_project(&project).expect("serialize project");

    assert_eq!(parse_gui_workspace_project(&text), Some(project));
    assert!(text.contains("version = 1"));
    assert!(text.contains("file.0 = "));
    assert!(text.contains("layout.version = 1"));
    assert!(!text.contains(temp.root.to_string_lossy().as_ref()));
}

#[test]
fn gui_left_panel_model_switches_between_files_workspaces_and_preferences() {
    let mut panel = GuiLeftPanelState::default();

    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Files);
    assert_eq!(panel.title(), "Files");

    panel.toggle_visibility();
    assert!(!panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Files);

    panel.show_workspaces();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Workspaces);
    assert_eq!(panel.title(), "Workspaces");

    panel.show_preferences();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);
    assert_eq!(panel.title(), "Preferences");

    panel.toggle_visibility();
    assert!(!panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);

    panel.show_files();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Files);

    panel.toggle_mode();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Workspaces);
    panel.toggle_mode();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);
    panel.toggle_mode();
    assert!(panel.visible);
    assert_eq!(panel.mode, GuiLeftPanelMode::Files);
}

#[test]
fn gui_workspace_project_parser_rejects_invalid_snapshots() {
    let temp = TempArea::new("gui-workspace-project-invalid");
    let path_hex = path_to_hex(&temp.path("README.md"));
    let second_hex = path_to_hex(&temp.path("LICENSE"));

    assert!(parse_gui_workspace_project("version = 2\n").is_none());
    assert!(parse_gui_workspace_project("version = 1\nname_hex = zz\n").is_none());
    assert!(
        parse_gui_workspace_project("version = 1\nname_hex = 646f6373\nactive = 0\n").is_none()
    );
    assert!(parse_gui_workspace_project(&format!(
        "version = 1\nname_hex = 646f6373\nactive = 2\nfile.0 = {path_hex}\n"
    ))
    .is_none());
    assert!(parse_gui_workspace_project(&format!(
        "version = 1\nname_hex = 646f6373\nactive = 0\nfile.1 = {path_hex}\n"
    ))
    .is_none());
    assert!(parse_gui_workspace_project(&format!(
            "version = 1\nname_hex = 646f6373\nactive = 0\nfile.0 = {path_hex}\nfile.1 = {second_hex}\nlayout.version = 1\nlayout.root = 0\nlayout.node.0 = leaf 0\n"
        ))
        .is_none());
}

#[test]
fn save_gui_workspace_project_writes_atomic_private_project_file() {
    let temp = TempArea::new("gui-workspace-project-save");
    let path = temp
        .path("xdg")
        .join("kfnotepad")
        .join("workspaces")
        .join("docs.v1");
    let project = GuiWorkspaceProject {
        name: "docs".to_string(),
        files: vec![temp.path("README.md")],
        active_ordinal: 0,
        layout: None,
    };

    save_gui_workspace_project(&path, &project).expect("save project");

    let text = fs::read_to_string(&path).expect("read project");
    assert_eq!(parse_gui_workspace_project(&text), Some(project));
    assert_no_temp_files(path.parent().expect("project parent"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let file_mode = fs::metadata(&path)
            .expect("project metadata")
            .permissions()
            .mode()
            & 0o777;
        let dir_mode = fs::metadata(path.parent().expect("project parent"))
            .expect("project dir metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600);
        assert_eq!(dir_mode, 0o700);
    }
}

#[test]
fn list_gui_workspace_projects_filters_and_sorts_valid_projects() {
    let temp = TempArea::new("gui-workspace-project-list");
    let projects_dir = temp.path("workspaces");
    fs::create_dir_all(&projects_dir).expect("create workspaces dir");
    let alpha = GuiWorkspaceProject {
        name: "alpha".to_string(),
        files: vec![temp.path("alpha.md")],
        active_ordinal: 0,
        layout: None,
    };
    let zeta = GuiWorkspaceProject {
        name: "zeta".to_string(),
        files: vec![temp.path("zeta.md")],
        active_ordinal: 0,
        layout: None,
    };

    save_gui_workspace_project(&projects_dir.join("zeta.v1"), &zeta).expect("save zeta");
    save_gui_workspace_project(&projects_dir.join("alpha.v1"), &alpha).expect("save alpha");
    fs::write(projects_dir.join("broken.v1"), "not a project").expect("write broken");
    fs::write(projects_dir.join("ignore.txt"), "ignored").expect("write ignored");
    fs::create_dir(projects_dir.join("folder.v1")).expect("create ignored dir");

    let projects = list_gui_workspace_projects(&projects_dir).expect("list projects");

    assert_eq!(
        projects
            .iter()
            .map(|entry| entry.project.name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
    assert_eq!(projects[0].project, alpha);
    assert_eq!(projects[1].project, zeta);
    assert_eq!(
        gui_workspace_project_path(&projects_dir, "Daily Notes"),
        Some(projects_dir.join("daily-notes.v1"))
    );
    assert_eq!(gui_workspace_project_path(&projects_dir, "../bad"), None);
}

#[test]
fn list_gui_workspace_projects_returns_empty_for_missing_directory() {
    let temp = TempArea::new("gui-workspace-project-list-missing");

    assert_eq!(
        list_gui_workspace_projects(&temp.path("missing")).expect("list missing"),
        Vec::new()
    );
}

#[test]
fn shared_theme_ids_cycle_and_parse_without_terminal_types() {
    let mut theme_id = EditorThemeId::Nocturne;

    for expected in [
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
        EditorThemeId::Nocturne,
    ] {
        theme_id = theme_id.next();
        assert_eq!(theme_id, expected);
        assert_eq!(EditorThemeId::from_label(theme_id.label()), Some(theme_id));
    }

    assert_eq!(
        EditorThemeId::from_label("paper"),
        Some(EditorThemeId::Paper)
    );
    assert_eq!(
        EditorThemeId::from_label("pastel"),
        Some(EditorThemeId::Paper)
    );
    assert_eq!(EditorThemeId::from_label("missing"), None);
}

#[test]
#[cfg(feature = "syntax")]
fn shared_syntax_highlighter_detects_and_keeps_state_without_terminal_types() {
    let highlighter = SyntaxHighlighter::default();
    let rust_document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: TextBuffer::from_text("/* start\ninside\n*/\nfn main() {}\n"),
    };
    let text_document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("plain note\n"),
    };

    assert_eq!(highlighter.syntax_name_for_document(&rust_document), "Rust");
    assert_eq!(highlighter.syntax_token_for_document(&rust_document), "rs");
    assert_eq!(
        highlighter.syntax_name_for_document(&text_document),
        "Plain Text"
    );
    assert_eq!(highlighter.syntax_token_for_document(&text_document), "txt");
    assert!(highlighter
        .highlight_line(&rust_document, "fn main() {}")
        .is_some());
    assert!(highlighter
        .highlight_line(&text_document, "plain note")
        .is_none());

    let stateful = highlighter.highlight_visible_lines(&rust_document, 1, 1);
    let reset = highlighter
        .highlight_line(&rust_document, "inside")
        .expect("standalone Rust line highlights");
    let stateful_line = stateful
        .first()
        .and_then(Option::as_ref)
        .expect("stateful Rust line highlights");

    assert_ne!(stateful_line[0].0.foreground, reset[0].0.foreground);
    assert_eq!(stateful_line[0].1, "inside");
}

#[test]
#[cfg(not(feature = "syntax"))]
fn shared_syntax_highlighter_falls_back_to_plain_text_without_feature() {
    let highlighter = SyntaxHighlighter::default();
    let document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: TextBuffer::from_text("fn main() {}\n"),
    };

    assert_eq!(
        highlighter.syntax_name_for_document(&document),
        "Plain Text"
    );
    assert_eq!(highlighter.syntax_token_for_document(&document), "txt");
    assert_eq!(highlighter.highlight_line(&document, "fn main() {}"), None);
    let (lines, state) = highlighter.highlight_lines_incremental(&document, 0, 1, None);
    assert_eq!(lines, vec![None]);
    assert!(state.is_none());
}
