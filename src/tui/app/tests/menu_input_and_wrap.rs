#[test]
fn horizontal_viewport_follows_cursor_left_and_right() {
    let settings = EditorSettings::default();
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n界xyz\n"),
    };
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 5 }, settings, 4, 10, 0),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 2 }, settings, 4, 10, 4),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 3 }, settings, 4, 10, 2),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 1, column: 4 }, settings, 4, 10, 0),
        2
    );
}

#[test]
fn ctrl_w_toggles_wrap_mode() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!runtime.settings.wrap_lines);
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap on");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
    ));
    assert!(!runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap off");
}

#[test]
fn paste_text_inserts_multiple_characters() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "ab\ncd");

    assert_eq!(document.buffer.lines(), vec!["ab", "cdhello"]);
    assert_eq!(cursor, Cursor { row: 1, column: 2 });
    assert_eq!(runtime.status, "Modified");
    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.to_text(), "hello");
    assert!(!document.buffer.undo_last_edit());
}

#[test]
fn paste_text_normalizes_crlf_sequences() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "x\r\ny");

    assert_eq!(document.buffer.lines(), vec!["x", "yhello"]);
    assert_eq!(cursor, Cursor { row: 1, column: 1 });
}

#[test]
fn paste_text_advances_cursor_to_combining_grapheme_end() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("e"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "\u{301}");

    assert_eq!(document.buffer.to_text(), "e\u{301}");
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
}

#[test]
fn paste_adds_to_search_query_when_search_is_active() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        search_active: true,
        ..EditorRuntime::default()
    };

    insert_paste(&mut document, &mut cursor, &mut runtime, "term");

    assert_eq!(runtime.search_query, "term");
    assert_eq!(runtime.status, "Search: term");
}

#[test]
fn f10_file_menu_can_save() {
    let temp = TempArea::new("file-menu-save");
    let path = temp.path("note.txt");
    fs::write(&path, "hello\n").expect("write fixture");
    let mut document = TextDocument {
        path: path.clone(),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)
    ));
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));

    assert_eq!(runtime.menu, None);
    assert_eq!(runtime.status, "Saved");
    assert!(!runtime.quit_confirmation_pending);
    assert!(!document.buffer.is_dirty());
    assert_eq!(
        fs::read_to_string(path).expect("read saved file"),
        "!hello\n"
    );
}

#[test]
fn f10_file_menu_quit_confirms_dirty_buffer() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [KeyCode::Down, KeyCode::Down, KeyCode::Down, KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert!(runtime.quit_confirmation_pending);
    assert!(runtime.status.contains("Unsaved changes"));

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [KeyCode::Down, KeyCode::Down, KeyCode::Down] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }
    assert!(handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
}

#[test]
fn f10_menu_can_toggle_wrap() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    assert_eq!(runtime.menu, Some(MenuState::default()));

    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::View,
            selected: 6,
        })
    );

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(runtime.menu, None);
    assert!(runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap on");
}

#[test]
fn f10_menu_tabs_between_groups() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::File,
            selected: 1,
        })
    );

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Edit,
            selected: 0,
        })
    );
    assert_eq!(runtime.status, "Menu: Edit");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::File,
            selected: 0,
        })
    );
    assert_eq!(runtime.status, "Menu: File");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Help,
            selected: 0,
        })
    );
    assert_eq!(runtime.status, "Menu: Help");
}

#[test]
fn f10_menu_home_and_end_select_first_and_last_items() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)
    ));
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::End, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Edit,
            selected: MenuGroup::Edit.items().len() - 1,
        })
    );

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)
    ));
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Edit,
            selected: 0,
        })
    );
}

#[test]
fn f10_menu_can_toggle_lines_and_theme() {
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
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [KeyCode::Right, KeyCode::Right, KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert!(!runtime.settings.show_line_numbers);
    assert!(!runtime.quit_confirmation_pending);
    assert_eq!(runtime.status, "Line numbers off");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(runtime.settings.theme_id, EditorThemeId::Aurora);
    assert_eq!(runtime.status, "Theme: aurora");
}

#[test]
fn f10_menu_can_redo() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document
        .buffer
        .insert_char(0, 5, '!')
        .expect("insert char for setup");
    assert!(document.buffer.undo_last_edit());
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(document.buffer.lines(), &["hello!".to_string()]);
    assert_eq!(runtime.status, "Redone");
}

#[test]
fn f10_menu_can_find_next() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f10_menu_can_find_previous() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 1, column: 10 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f10_menu_can_delete_words() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 11 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(document.buffer.line(0), Some("alpha gamma"));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");

    runtime.menu = Some(MenuState {
        group: MenuGroup::Edit,
        selected: 5,
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
fn f10_menu_can_start_go_to_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert!(runtime.goto_line_active);
    assert_eq!(runtime.status, "Go to line: ");
}

#[test]
fn f10_menu_can_go_to_top_and_bottom() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 1, column: 1 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }
    assert_eq!(cursor, Cursor { row: 2, column: 5 });
    assert_eq!(runtime.status, "Bottom");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
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
    assert_eq!(runtime.status, "Top");
}

#[test]
fn f10_menu_can_move_by_word() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 16 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(cursor, Cursor { row: 0, column: 11 });
    assert_eq!(runtime.status, "Previous word");

    cursor = Cursor { row: 0, column: 0 };
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE)
    ));
    for key in [
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.menu, None);
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Next word");
}
