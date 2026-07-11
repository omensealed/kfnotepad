#[test]
fn page_up_and_down_move_by_visible_page() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\nfive\n"),
    };
    let mut cursor = Cursor { row: 1, column: 3 };
    let mut runtime = EditorRuntime {
        page_rows: 2,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 3, column: 3 });
    assert_eq!(runtime.status, "Page down");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 3 });
    assert_eq!(runtime.status, "Page up");

    cursor = Cursor { row: 3, column: 4 };
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 4, column: 4 });
}

#[test]
fn syntax_highlighter_detects_rust_and_falls_back_to_plain_text() {
    let highlighter = SyntaxHighlighter::default();
    let rust_document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
    };
    let text_document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("plain note\n"),
    };

    assert_eq!(highlighter.syntax_name_for_document(&rust_document), "Rust");
    assert_eq!(
        highlighter.syntax_name_for_document(&text_document),
        "Plain Text"
    );
    assert!(highlighter
        .highlight_line(&rust_document, "fn main() {}")
        .is_some());
    assert!(highlighter
        .highlight_line(&text_document, "plain note")
        .is_none());
}

#[test]
fn syntax_highlighter_keeps_state_before_viewport() {
    let highlighter = SyntaxHighlighter::default();
    let document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("/* start\ninside\n*/\nfn main() {}\n"),
    };

    let stateful = highlighter.highlight_visible_lines(&document, 1, 1);
    let reset = highlighter
        .highlight_line(&document, "inside")
        .expect("standalone Rust line highlights");
    let stateful_line = stateful
        .first()
        .and_then(Option::as_ref)
        .expect("stateful Rust line highlights");

    assert_ne!(stateful_line[0].0.foreground, reset[0].0.foreground);
    assert_eq!(stateful_line[0].1, "inside");
}

#[test]
fn viewport_follows_cursor_down_and_up() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\n"),
    };

    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 3, column: 0 },
            0,
            2,
            EditorSettings::default(),
            2,
            80
        ),
        2
    );
    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 0, column: 0 },
            2,
            2,
            EditorSettings::default(),
            2,
            80
        ),
        0
    );
}

#[test]
fn wrapped_viewport_can_scroll_to_last_source_line() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one two three four five\nlast\n"),
    };
    let settings = EditorSettings {
        wrap_lines: true,
        ..EditorSettings::default()
    };

    assert_eq!(
            clamp_passive_viewport(&document, 99, 10, settings),
            1,
            "wrapped mode must allow the final source line to become visible even when source line count is smaller than visible rows"
        );
    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 1, column: 0 },
            0,
            3,
            settings,
            2,
            10
        ),
        1,
        "cursor-following clamp must account for visual rows in wrapped mode"
    );
}

#[test]
fn dirty_quit_requires_second_ctrl_q() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();
    let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        quit
    ));
    assert!(runtime.quit_confirmation_pending);
    assert!(runtime.status.contains("Unsaved changes"));

    assert!(handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        quit
    ));
}

#[test]
fn backspace_key_updates_buffer_and_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 5 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
    ));

    assert_eq!(document.buffer.line(0), Some("hell"));
    assert_eq!(cursor, Cursor { row: 0, column: 4 });
    assert!(document.buffer.is_dirty());
}

#[test]
fn ctrl_backspace_and_ctrl_delete_update_buffer_by_word() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 11 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.line(0), Some("alpha gamma"));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Delete, KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn ctrl_k_and_edit_menu_delete_to_line_end() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\nnext\n"),
    };
    let mut cursor = Cursor { row: 0, column: 6 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)
    ));
    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
    assert!(!runtime.quit_confirmation_pending);

    assert!(document.buffer.undo_last_edit());
    cursor = Cursor { row: 0, column: 6 };
    runtime.menu = Some(MenuState {
        group: MenuGroup::Edit,
        selected: 6,
    });
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));

    assert_eq!(runtime.menu, None);
    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn home_and_end_keys_move_within_current_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
    };
    let mut cursor = Cursor { row: 0, column: 2 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::End, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}

#[test]
fn insert_key_toggles_overwrite_mode_for_typed_characters() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcd\n"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
    ));
    assert!(runtime.overwrite_mode);
    assert_eq!(runtime.status, "Overwrite on");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("aXcd"));
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
    assert_eq!(runtime.status, "Modified overwrite");

    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.line(0), Some("abcd"));
    assert!(!document.buffer.undo_last_edit());

    cursor.column = 4;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("abcd!"));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
    ));
    assert!(!runtime.overwrite_mode);
    assert_eq!(runtime.status, "Insert mode");
    cursor.column = 1;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("aYbcd!"));
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn overwrite_replaces_whole_grapheme_cluster_and_advances_to_boundary() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("a🇺🇸e\u{301}!"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime {
        overwrite_mode: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("axe\u{301}!"));
    assert_eq!(cursor, Cursor { row: 0, column: 2 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("axy!"));
    assert_eq!(cursor, Cursor { row: 0, column: 3 });
}

#[test]
fn ctrl_r_and_view_menu_toggle_reader_mode() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.settings.gui_reader_mode_enabled);
    assert_eq!(
        runtime.status,
        format!(
            "Reader mode on: {} lines/min",
            DEFAULT_GUI_READER_LINES_PER_MINUTE
        )
    );
    assert!(!runtime.quit_confirmation_pending);

    runtime.menu = Some(MenuState {
        group: MenuGroup::View,
        selected: 3,
    });
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert!(!runtime.settings.gui_reader_mode_enabled);
    assert_eq!(runtime.status, "Reader mode off");
}

#[test]
fn view_menu_adjusts_reader_speed_with_bounds() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        settings: EditorSettings {
            gui_reader_lines_per_minute: MIN_GUI_READER_LINES_PER_MINUTE,
            ..EditorSettings::default()
        },
        reader_scroll_milli_lines: 900,
        ..EditorRuntime::default()
    };

    runtime.menu = Some(MenuState {
        group: MenuGroup::View,
        selected: 4,
    });
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.settings.gui_reader_lines_per_minute,
        MIN_GUI_READER_LINES_PER_MINUTE
    );
    assert_eq!(runtime.reader_scroll_milli_lines, 0);
    assert_eq!(
        runtime.status,
        format!(
            "Reader speed: {} lines/min",
            MIN_GUI_READER_LINES_PER_MINUTE
        )
    );

    runtime.menu = Some(MenuState {
        group: MenuGroup::View,
        selected: 5,
    });
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.settings.gui_reader_lines_per_minute,
        MIN_GUI_READER_LINES_PER_MINUTE + 10
    );
    assert_eq!(
        runtime.status,
        format!(
            "Reader speed: {} lines/min",
            MIN_GUI_READER_LINES_PER_MINUTE + 10
        )
    );
}

#[test]
fn reader_tick_scrolls_viewport_without_moving_cursor_and_stops_at_end() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("1\n2\n3\n4\n5\n"),
    };
    let mut state = EditorTabState {
        cursor: Cursor { row: 0, column: 0 },
        viewport_start: 0,
        horizontal_offset: 0,
    };
    let mut runtime = EditorRuntime {
        settings: EditorSettings {
            gui_reader_mode_enabled: true,
            gui_reader_lines_per_minute: 240,
            ..EditorSettings::default()
        },
        ..EditorRuntime::default()
    };

    assert!(apply_reader_tick(&document, &mut state, &mut runtime, 2));
    assert_eq!(state.cursor, Cursor { row: 0, column: 0 });
    assert_eq!(state.viewport_start, 1);
    assert!(runtime.settings.gui_reader_mode_enabled);
    assert_eq!(runtime.status, "Reader mode: 240 lines/min");

    assert!(apply_reader_tick(&document, &mut state, &mut runtime, 2));
    assert!(apply_reader_tick(&document, &mut state, &mut runtime, 2));
    assert_eq!(state.viewport_start, 3);
    assert!(runtime.settings.gui_reader_mode_enabled);

    assert!(apply_reader_tick(&document, &mut state, &mut runtime, 2));
    assert_eq!(state.viewport_start, 3);
    assert!(!runtime.settings.gui_reader_mode_enabled);
    assert_eq!(runtime.status, "Reader mode stopped at document end");
}

#[test]
fn reader_viewport_clamp_does_not_snap_back_to_cursor() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("1\n2\n3\n4\n5\n6\n7\n8\n"),
    };
    let cursor = Cursor { row: 0, column: 0 };

    assert_eq!(
        clamp_viewport(&document, cursor, 4, 3, EditorSettings::default(), 2, 80),
        0
    );
    assert_eq!(
        clamp_passive_viewport(&document, 4, 3, EditorSettings::default()),
        4
    );
    assert_eq!(
        clamp_passive_viewport(&document, 99, 3, EditorSettings::default()),
        5
    );
}

#[test]
fn reader_tick_accumulates_fractional_speed_and_edit_stops_mode() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("1\n2\n3\n4\n5\n"),
    };
    let mut state = EditorTabState::default();
    let mut runtime = EditorRuntime {
        settings: EditorSettings {
            gui_reader_mode_enabled: true,
            gui_reader_lines_per_minute: 60,
            ..EditorSettings::default()
        },
        ..EditorRuntime::default()
    };

    assert!(!apply_reader_tick(&document, &mut state, &mut runtime, 2));
    assert_eq!(state.viewport_start, 0);
    assert_eq!(runtime.reader_scroll_milli_lines, 250);
    for _ in 0..3 {
        let _ = apply_reader_tick(&document, &mut state, &mut runtime, 2);
    }
    assert_eq!(state.viewport_start, 1);

    let mut cursor = Cursor { row: 0, column: 0 };
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
    ));
    assert!(!runtime.settings.gui_reader_mode_enabled);
    assert_eq!(runtime.reader_scroll_milli_lines, 0);
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn ctrl_a_and_ctrl_e_move_within_current_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
    };
    let mut cursor = Cursor { row: 0, column: 2 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });
    assert!(!runtime.quit_confirmation_pending);

    runtime.quit_confirmation_pending = true;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert!(!runtime.quit_confirmation_pending);
}

#[test]
fn ctrl_home_and_end_move_to_document_edges() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 1, column: 2 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 2, column: 5 });
    assert_eq!(runtime.status, "Bottom");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "Top");
}

#[test]
fn ctrl_left_and_right_move_by_words() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha, beta\n  gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 7 });
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 2 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 7 });
}

#[test]
fn ctrl_l_toggles_line_numbers() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
    ));
    assert!(!runtime.settings.show_line_numbers);
    assert!(!runtime.quit_confirmation_pending);
    assert_eq!(runtime.status, "Line numbers off");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.settings.show_line_numbers);
    assert_eq!(runtime.status, "Line numbers on");
}

#[test]
fn ctrl_t_cycles_builtin_themes() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert_eq!(runtime.settings.theme_id, EditorThemeId::Nocturne);
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)
    ));
    assert_eq!(runtime.settings.theme_id, EditorThemeId::Aurora);
    assert!(!runtime.quit_confirmation_pending);
    assert_eq!(runtime.status, "Theme: aurora");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)
    ));
    assert_eq!(runtime.settings.theme_id, EditorThemeId::Paper);
    assert_eq!(runtime.status, "Theme: pastel");

    for (theme_id, status) in [
        (EditorThemeId::Terminal, "Theme: terminal"),
        (EditorThemeId::Abyss, "Theme: abyss"),
        (EditorThemeId::Terror, "Theme: terror"),
        (EditorThemeId::Nocturne, "Theme: nocturne"),
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)
        ));
        assert_eq!(runtime.settings.theme_id, theme_id);
        assert_eq!(runtime.status, status);
    }
}

#[test]
fn ctrl_shift_t_cycles_syntax_themes() {
    let mut document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert_eq!(runtime.settings.syntax_theme_id, EditorThemeId::Nocturne);
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )
    ));
    assert_eq!(runtime.settings.syntax_theme_id, EditorThemeId::Aurora);
    assert!(!runtime.quit_confirmation_pending);
    assert_eq!(runtime.status, "Syntax theme: aurora");
}

#[test]
fn tab_and_backtab_indent_and_unindent_current_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("    alpha"));
    assert_eq!(cursor.column, 4);
    assert_eq!(runtime.status, "Indented");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("alpha"));
    assert_eq!(cursor.column, 0);
    assert_eq!(runtime.status, "Unindented");
}

#[test]
fn view_menu_can_cycle_syntax_theme() {
    let mut document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        menu: Some(MenuState {
            group: MenuGroup::View,
            selected: 2,
        }),
        ..EditorRuntime::default()
    };

    assert!(!handle_menu_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(runtime.settings.syntax_theme_id, EditorThemeId::Aurora);
    assert_eq!(runtime.status, "Syntax theme: aurora");
}

#[test]
fn requested_theme_palettes_are_available() {
    let terminal = EditorTheme::for_id(EditorThemeId::Terminal);
    assert_eq!(
        terminal.status_bg,
        Color::Rgb {
            r: 72,
            g: 255,
            b: 112
        }
    );
    assert_eq!(terminal.header_bg, Color::Rgb { r: 0, g: 36, b: 12 });

    let abyss = EditorTheme::for_id(EditorThemeId::Abyss);
    assert_eq!(abyss.help_bg, Color::Rgb { r: 3, g: 7, b: 18 });
    assert_eq!(
        abyss.dirty_fg,
        Color::Rgb {
            r: 255,
            g: 64,
            b: 96
        }
    );

    let terror = EditorTheme::for_id(EditorThemeId::Terror);
    assert_eq!(terror.header_bg, Color::Rgb { r: 45, g: 0, b: 58 });
    assert_eq!(
        terror.header_fg,
        Color::Rgb {
            r: 255,
            g: 42,
            b: 160
        }
    );
}

#[test]
fn terminal_syntax_themes_map_source_colors_to_distinct_palettes() {
    let sample = syntect::highlighting::Color {
        r: 120,
        g: 140,
        b: 230,
        a: 255,
    };

    assert_eq!(
        syntect_color_to_terminal(sample, EditorThemeId::Nocturne),
        Color::Rgb {
            r: 132,
            g: 172,
            b: 255,
        }
    );
    assert_eq!(
        syntect_color_to_terminal(sample, EditorThemeId::Terror),
        Color::Rgb {
            r: 136,
            g: 172,
            b: 255,
        }
    );
    assert_ne!(
        syntect_color_to_terminal(sample, EditorThemeId::Nocturne),
        syntect_color_to_terminal(sample, EditorThemeId::Paper)
    );
}

#[test]
fn ctrl_z_undo_restores_buffer_and_clamps_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
    };
    document
        .buffer
        .delete_char(0, 5)
        .expect("join lines for setup");
    let mut cursor = Cursor { row: 1, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL)
    ));

    assert_eq!(
        document.buffer.lines(),
        &["hello".to_string(), "world".to_string()]
    );
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Undone");
}

#[test]
fn ctrl_y_redo_restores_buffer_and_clamps_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
    };
    document
        .buffer
        .delete_char(0, 5)
        .expect("join lines for setup");
    assert!(document.buffer.undo_last_edit());
    let mut cursor = Cursor { row: 1, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.lines(), &["helloworld".to_string()]);
    assert_eq!(cursor, Cursor { row: 0, column: 10 });
    assert_eq!(runtime.status, "Redone");
}

#[test]
fn f3_repeats_last_search() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
    ));
    for key in [
        KeyCode::Char('a'),
        KeyCode::Char('l'),
        KeyCode::Char('p'),
        KeyCode::Char('h'),
        KeyCode::Char('a'),
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.last_search_query, "alpha");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f3_repeats_last_search_case_insensitive_by_default() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("Alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 1, column: 4 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f3_without_search_reports_missing_query() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "No previous search");
}

#[test]
fn shift_f3_repeats_last_search_backwards() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\ngamma alpha\n"),
    };
    let mut cursor = Cursor { row: 2, column: 7 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn shift_f3_without_search_reports_missing_query() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "No previous search");
}

#[test]
fn ctrl_g_goes_to_line_and_clamps_column() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 0, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.goto_line_active);
    assert_eq!(runtime.status, "Go to line: ");

    for key in [KeyCode::Char('3'), KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert!(!runtime.goto_line_active);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });
    assert_eq!(runtime.status, "Line 3");
}

#[test]
fn go_to_line_rejects_out_of_range_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
    };
    let mut cursor = Cursor { row: 1, column: 1 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
    ));
    for key in [KeyCode::Char('9'), KeyCode::Char('9'), KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert!(!runtime.goto_line_active);
    assert_eq!(cursor, Cursor { row: 1, column: 1 });
    assert_eq!(runtime.status, "Line out of range: 99");
}

#[test]
fn ctrl_f_search_moves_cursor_to_match() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.search_active);

    for value in ['b', 'e', 't', 'a'] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE)
        ));
    }

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 0 });
    assert!(!runtime.search_active);
    assert_eq!(runtime.status, "Found: beta");
}

#[test]
fn ctrl_f_search_remembers_history_and_recalls_with_arrows() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\ngamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    for query in ["alpha", "beta", "alpha"] {
        start_search(&mut runtime);
        runtime.search_query = query.to_string();
        handle_search_key_event(
            &document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
    }

    assert_eq!(
        runtime.search_history,
        vec![String::from("alpha"), String::from("beta")]
    );

    start_search(&mut runtime);
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "alpha");
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "beta");
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "alpha");
}

#[test]
fn search_case_toggle_persists_and_exact_case_changes_results() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("Alpha\nalpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    toggle_search_case(&mut runtime);
    assert!(runtime.settings.search_case_sensitive);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 0 });
}

#[test]
fn save_failure_status_does_not_include_buffer_contents() {
    let secret = "SUPER_SECRET_TOKEN";
    let mut document = TextDocument {
        path: PathBuf::from("."),
        buffer: kfnotepad::TextBuffer::from_text(secret),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
    ));

    assert!(runtime.status.starts_with("Save failed:"));
    assert!(!runtime.status.contains(secret));
}

#[test]
fn tui_save_refuses_external_modification_since_open() {
    let temp = TempArea::new("tui-save-conflict");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut document = open_text_file(&path).expect("open document");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    fs::write(&path, "external\n").expect("external edit");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
    ));

    assert_eq!(fs::read_to_string(&path).expect("read file"), "external\n");
    assert!(document.buffer.is_dirty());
    assert!(runtime
        .status
        .contains("file changed on disk since open or last save"));
}
