use super::*;

fn replacement_key_event(
    key: Key,
    modifiers: keyboard::Modifiers,
    text: Option<&str>,
) -> keyboard::Event {
    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: text.map(Into::into),
        repeat: false,
    }
}

#[test]
fn gui_editor_replacement_keyboard_bridge_maps_text_and_navigation() {
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("a".into()),
            keyboard::Modifiers::NONE,
            Some("a"),
        )),
        vec![GuiEditorReplacementInput::InsertChar('a')]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("A".into()),
            keyboard::Modifiers::SHIFT,
            Some("A"),
        )),
        vec![GuiEditorReplacementInput::InsertChar('A')]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("x".into()),
            keyboard::Modifiers::CTRL,
            Some("x"),
        )),
        Vec::<GuiEditorReplacementInput>::new()
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("a".into()),
            keyboard::Modifiers::CTRL,
            Some("a"),
        )),
        vec![GuiEditorReplacementInput::SelectAll]
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("c".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Copy)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("x".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Cut)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("v".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Paste)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("z".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Undo)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("z".into()),
            keyboard::Modifiers::CTRL.union(keyboard::Modifiers::SHIFT),
            None,
        )),
        Some(GuiMenuCommand::Redo)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("y".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Redo)
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Enter),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::InsertNewline]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Backspace),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteBackward]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Delete),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteForward]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Escape),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::ClearSelection]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Home),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::MoveLineStart]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::End),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::MoveLineEnd]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowDown),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down)]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::PageDown),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::ScrollViewportLines(
            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowLeft),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(
            kfnotepad::CursorMove::WordLeft
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowRight),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(
            kfnotepad::CursorMove::WordRight
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Backspace),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeletePreviousWord]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Delete),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteNextWord]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("k".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteToLineEnd]
    );
}

#[test]
fn gui_editor_replacement_ime_commit_maps_to_text_insertion() {
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
            "かな".to_string()
        )),
        vec![
            GuiEditorReplacementInput::InsertChar('か'),
            GuiEditorReplacementInput::InsertChar('な'),
        ]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Preedit(
            "か".to_string(),
            Some(0..3),
        )),
        Vec::<GuiEditorReplacementInput>::new()
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Closed),
        Vec::<GuiEditorReplacementInput>::new()
    );
}

#[test]
fn gui_editor_ime_preedit_renders_at_cursor_without_mutating_line() {
    let line = GuiEditorViewportLine {
        number: 1,
        text: "start end".to_string(),
        cursor_column: Some(6),
        selection: None,
        syntax_segments: Some(vec![GuiEditorSyntaxSegment {
            text: "start end".to_string(),
            color: Color::WHITE,
        }]),
    };
    let preedit = GuiImePreedit {
        tile_id: GuiTileId(1),
        content: "かな".to_string(),
        selection: Some(0..3),
    };

    let rendered = gui_editor_viewport_line_with_ime_preedit(line.clone(), Some(&preedit));

    assert_eq!(line.text, "start end");
    assert_eq!(rendered.text, "start かなend");
    assert_eq!(rendered.cursor_column, None);
    assert_eq!(
        rendered.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 6,
            end_column: 7,
        })
    );
    assert_eq!(rendered.syntax_segments, None);
}

#[test]
fn gui_editor_ime_preedit_selection_expands_to_grapheme_boundaries() {
    let preedit = GuiImePreedit {
        tile_id: GuiTileId(1),
        content: "e\u{301}x".to_string(),
        selection: Some(1..3),
    };

    assert_eq!(gui_ime_preedit_selection_columns(&preedit), Some((0, 2)));

    let line = GuiEditorViewportLine {
        number: 1,
        text: "ab".to_string(),
        cursor_column: Some(1),
        selection: None,
        syntax_segments: None,
    };
    let rendered = gui_editor_viewport_line_with_ime_preedit(line, Some(&preedit));

    assert_eq!(rendered.text, "ae\u{301}xb");
    assert_eq!(
        rendered.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 1,
            end_column: 3,
        })
    );
}

#[test]
fn gui_editor_ime_request_cursor_rect_tracks_visual_row_and_gutter() {
    let request = GuiImeInputMethodRequest {
        visual_row: 3,
        cursor_column: 5,
        gutter_width: 42.0,
        character_width: 9.0,
        row_height: 18.0,
        preedit: Some(input_method::Preedit {
            content: "かな".to_string(),
            selection: Some(0..3),
            text_size: Some(Pixels(16.0)),
        }),
    };

    assert_eq!(
        request.cursor_rect(Rectangle::new(
            iced::Point::new(10.0, 20.0),
            Size::new(500.0, 300.0)
        )),
        Rectangle::new(iced::Point::new(97.0, 74.0), Size::new(1.0, 18.0))
    );
    assert_eq!(
        request
            .preedit
            .as_ref()
            .map(|preedit| preedit.content.as_str()),
        Some("かな")
    );
}

#[test]
fn gui_editor_replacement_keyboard_bridge_applies_when_explicitly_routed() {
    let temp = TempArea::new("gui-replacement-bridge");
    let file = temp.path("bridge.txt");
    fs::write(&file, "one\ntwo").expect("write bridge");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::InsertChar('X'),
        GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
        GuiEditorReplacementInput::InsertNewline,
    ]);

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "Xo\nne\ntwo"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.active_document_text(), "Xo\nne\ntwo");
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.status_message, "replacement edit");
}

#[test]
fn gui_editor_replacement_home_end_move_within_current_line() {
    let temp = TempArea::new("gui-replacement-home-end");
    let file = temp.path("home-end.txt");
    fs::write(&file, "abc\ndefgh\n").expect("write home end");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 1, column: 2 });
    state.sync_active_editor_to_document();

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::MoveLineEnd,
    ]);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 5 }
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::MoveLineStart,
    ]);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "abc\ndefgh\n"
    );
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
}

#[test]
fn gui_editor_replacement_ime_commit_applies_when_explicitly_routed() {
    let temp = TempArea::new("gui-replacement-ime");
    let file = temp.path("ime.txt");
    fs::write(&file, "start").expect("write ime");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
            "かな".to_string(),
        )),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "かなstart"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 2 }
    );
    assert_eq!(state.active_document_text(), "かなstart");
}

#[test]
fn gui_editor_replacement_ime_preedit_is_transient_until_commit() {
    let temp = TempArea::new("gui-replacement-ime-preedit");
    let file = temp.path("ime-preedit.txt");
    fs::write(&file, "start").expect("write ime preedit");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;

    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Opened),
    );
    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Preedit("かな".to_string(), Some(0..3))),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "start"
    );
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.replacement_ime_preedit,
        Some(GuiImePreedit {
            tile_id,
            content: "かな".to_string(),
            selection: Some(0..3),
        })
    );

    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Commit("かな".to_string())),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "かなstart"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(state.replacement_ime_preedit, None);
}

#[test]
fn gui_editor_overwrite_ime_commit_uses_bulk_edit_and_preserves_filtering() {
    let temp = TempArea::new("gui-replacement-overwrite-ime");
    let file = temp.path("overwrite-ime.txt");
    fs::write(&file, "alpha").expect("write overwrite ime");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    state.replacement_overwrite_mode = true;
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 1 });
    state.sync_pane_cursor_to_document(state.active_pane);

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Commit("か\nな".to_string())),
    );

    assert_eq!(state.active_document_text(), "aかなha");
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 3 }
    );
    assert_eq!(kfnotepad::to_text_call_count(), 1);
    assert_eq!(kfnotepad::from_text_call_count(), 0);

    state.undo_active_edit();
    assert_eq!(state.active_document_text(), "alpha");
}

#[test]
fn gui_editor_replacement_selection_persists_until_next_active_tile_edit() {
    let temp = TempArea::new("gui-replacement-selection");
    let file = temp.path("selection.txt");
    fs::write(&file, "alpha beta").expect("write selection");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 6 },
            focus: DocumentCursor { row: 0, column: 10 },
        },
    ]);

    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .and_then(|pane| pane.editor.replacement_selection),
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 0, column: 6 },
            focus: DocumentCursor { row: 0, column: 10 },
        })
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 10 }
    );
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "alpha beta"
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::InsertChar('X'),
    ]);

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "alpha X"
    );
    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .and_then(|pane| pane.editor.replacement_selection),
        None
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 7 }
    );
    assert_eq!(state.active_document_text(), "alpha X");
}

#[test]
fn gui_editor_replacement_message_edits_active_tile_when_renderer_is_live() {
    let temp = TempArea::new("gui-replacement-live");
    let file = temp.path("live.txt");
    fs::write(&file, "unchanged").expect("write live");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "Xunchanged"
    );
    assert_eq!(state.active_document_text(), "Xunchanged");
    assert_eq!(state.status_message, "replacement edit");
}

#[test]
fn gui_restore_last_workspace_toggle_persists_config() {
    let temp = TempArea::new("gui-restore-toggle");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
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

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

    assert!(state.settings.gui_restore_last_workspace);
    assert_eq!(state.status_message, "restore last workspace: on");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(false));

    assert!(!state.settings.gui_restore_last_workspace);
    assert_eq!(state.status_message, "restore last workspace: off");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
fn gui_restore_last_workspace_toggle_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-restore-toggle-failure");
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

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

    assert!(!state.settings.gui_restore_last_workspace);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}
