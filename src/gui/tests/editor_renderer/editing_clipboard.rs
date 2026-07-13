use super::*;

#[test]
fn gui_editor_replacement_input_model_edits_and_moves_shared_document() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement.txt"),
        buffer: TextBuffer::from_text("ab\ncd"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertChar('X'),
    );
    assert_eq!(document.buffer.to_text(), "aXb\ncd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
    );
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertNewline,
    );
    assert_eq!(document.buffer.to_text(), "aXb\n\ncd");
    assert_eq!(cursor, DocumentCursor { row: 1, column: 0 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );
    assert_eq!(document.buffer.to_text(), "aXb\ncd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteForward,
    );
    assert_eq!(document.buffer.to_text(), "aXbcd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });
    assert_eq!(viewport.first_line, 1);
}

#[test]
fn gui_editor_replacement_overwrite_replaces_whole_grapheme_cluster() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement-overwrite-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸e\u{301}!"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input_with_mode(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        true,
        GuiEditorReplacementInput::InsertChar('x'),
    );
    assert_eq!(document.buffer.to_text(), "axe\u{301}!");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });

    apply_gui_editor_replacement_input_with_mode(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        true,
        GuiEditorReplacementInput::InsertChar('y'),
    );
    assert_eq!(document.buffer.to_text(), "axy!");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });
}

#[test]
fn gui_editor_replacement_input_model_keeps_viewport_and_gutter_synced() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement-long.txt"),
        buffer: TextBuffer::from_text(&numbered_lines(100)),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::ScrollViewportLines(2),
    );

    let slice = gui_editor_viewport_slice(
        &document.buffer.to_text(),
        document.buffer.line_count(),
        viewport,
        cursor,
        None,
    );
    let model = gui_editor_read_only_render_model(&slice);

    assert_eq!(cursor, DocumentCursor { row: 2, column: 0 });
    assert_eq!(slice.first_line, 3);
    assert_eq!(
        model.gutter_text,
        gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES)
    );
    assert!(model.body_text.starts_with("3\n4\n5\n"));
}

#[test]
fn gui_editor_replacement_selection_extracts_same_line_and_multiline_text() {
    let document = TextDocument {
        path: PathBuf::from("selection.txt"),
        buffer: TextBuffer::from_text("abécd\nsecond\nthird"),
    };

    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 1 },
                focus: DocumentCursor { row: 0, column: 4 },
            },
        )
        .as_deref(),
        Some("béc")
    );
    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 1, column: 3 },
                focus: DocumentCursor { row: 0, column: 2 },
            },
        )
        .as_deref(),
        Some("écd\nsec")
    );
}

#[test]
fn gui_editor_replacement_selection_expands_to_grapheme_boundaries() {
    let flag = "🇺🇸";
    let mut document = TextDocument {
        path: PathBuf::from("selection-grapheme.txt"),
        buffer: TextBuffer::from_text(&format!("{flag}x\n")),
    };

    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 1 },
                focus: DocumentCursor { row: 0, column: 2 },
            },
        )
        .as_deref(),
        Some(flag)
    );

    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 1 },
        focus: DocumentCursor { row: 0, column: 2 },
    });
    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );

    assert_eq!(document.buffer.to_text(), "x");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 0 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_multiline_selection_expands_mixed_grapheme_boundaries() {
    let mut document = TextDocument {
        path: PathBuf::from("selection-mixed-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸b\nc👩‍💻d e\u{301}f"),
    };
    let reversed_selection = GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 1, column: 7 },
        focus: DocumentCursor { row: 0, column: 2 },
    };

    assert_eq!(
        gui_editor_replacement_selected_text(&document, reversed_selection).as_deref(),
        Some("🇺🇸b\nc👩‍💻d e\u{301}")
    );

    let mut cursor = DocumentCursor { row: 1, column: 7 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(reversed_selection);
    assert_eq!(
        gui_editor_replacement_cut_selection(
            &mut document,
            &mut cursor,
            &mut viewport,
            &mut selection,
        )
        .as_deref(),
        Some("🇺🇸b\nc👩‍💻d e\u{301}")
    );

    assert_eq!(document.buffer.to_text(), "af");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_selection_replaces_and_deletes_selected_ranges() {
    let mut document = TextDocument {
        path: PathBuf::from("selection-edit.txt"),
        buffer: TextBuffer::from_text("abc\ndef"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 1 },
            focus: DocumentCursor { row: 1, column: 2 },
        },
    );
    assert_eq!(cursor, DocumentCursor { row: 1, column: 2 });
    assert!(selection.is_some());

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertChar('X'),
    );
    assert_eq!(document.buffer.to_text(), "aXf");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 1 },
            focus: DocumentCursor { row: 0, column: 2 },
        },
    );
    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );
    assert_eq!(document.buffer.to_text(), "af");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_select_all_deletes_entire_document() {
    let mut document = TextDocument {
        path: PathBuf::from("select-all.txt"),
        buffer: TextBuffer::from_text("one\ntwo"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectAll,
    );
    assert_eq!(cursor, DocumentCursor { row: 1, column: 3 });
    assert_eq!(
        gui_editor_replacement_selected_text(&document, selection.expect("selection")).as_deref(),
        Some("one\ntwo")
    );

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteForward,
    );
    assert_eq!(document.buffer.to_text(), "");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 0 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_clipboard_copy_reads_selection_without_mutation() {
    let document = TextDocument {
        path: PathBuf::from("copy.txt"),
        buffer: TextBuffer::from_text("one\ntwo\nthree"),
    };
    let selection = GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 2, column: 2 },
        focus: DocumentCursor { row: 0, column: 1 },
    };

    assert_eq!(
        gui_editor_replacement_copy_selection(&document, Some(selection)).as_deref(),
        Some("ne\ntwo\nth")
    );
    assert_eq!(document.buffer.to_text(), "one\ntwo\nthree");
    assert_eq!(gui_editor_replacement_copy_selection(&document, None), None);
}

#[test]
fn gui_editor_replacement_clipboard_cut_returns_text_and_deletes_selection() {
    let mut document = TextDocument {
        path: PathBuf::from("cut.txt"),
        buffer: TextBuffer::from_text("alpha\nbeta\ngamma"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 2 },
        focus: DocumentCursor { row: 1, column: 2 },
    });

    assert_eq!(
        gui_editor_replacement_cut_selection(
            &mut document,
            &mut cursor,
            &mut viewport,
            &mut selection,
        )
        .as_deref(),
        Some("pha\nbe")
    );
    assert_eq!(document.buffer.to_text(), "alta\ngamma");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_clipboard_paste_replaces_selection_and_handles_newlines() {
    let mut document = TextDocument {
        path: PathBuf::from("paste.txt"),
        buffer: TextBuffer::from_text("hello world"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 11 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 6 },
        focus: DocumentCursor { row: 0, column: 11 },
    });

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "there\nfriend",
    );

    assert_eq!(document.buffer.to_text(), "hello there\nfriend");
    assert_eq!(cursor, DocumentCursor { row: 1, column: 6 });
    assert_eq!(selection, None);
    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.to_text(), "hello world");
    assert!(!document.buffer.undo_last_edit());
}

#[test]
fn gui_editor_replacement_paste_expands_partial_grapheme_selection() {
    let mut document = TextDocument {
        path: PathBuf::from("paste-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸b e\u{301}x"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 2 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 2 },
        focus: DocumentCursor { row: 0, column: 3 },
    });

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "Z",
    );

    assert_eq!(document.buffer.to_text(), "aZb e\u{301}x");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);

    selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 5 },
        focus: DocumentCursor { row: 0, column: 6 },
    });
    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "Y",
    );

    assert_eq!(document.buffer.to_text(), "aZb Yx");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 5 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_paste_advances_cursor_to_combining_grapheme_end() {
    let mut document = TextDocument {
        path: PathBuf::from("paste-combining-cursor.txt"),
        buffer: TextBuffer::from_text("e"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "\u{301}",
    );

    assert_eq!(document.buffer.to_text(), "e\u{301}");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_paste_rejects_oversized_result_atomically() {
    let limit = usize::try_from(kfnotepad::MAX_TEXT_FILE_BYTES).expect("text limit fits usize");
    let original = format!("x\n{}", "x".repeat(limit - 2));
    let mut document = TextDocument {
        path: PathBuf::from("oversized-paste.txt"),
        buffer: TextBuffer::from_text(&original),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 0 },
        focus: DocumentCursor { row: 0, column: 1 },
    });

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "three bytes",
    );

    assert_eq!(document.buffer.to_text(), original);
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert!(selection.is_some());
    assert!(!document.buffer.is_dirty());
}
#[test]
fn gui_editor_replacement_typed_insert_rejects_text_limit_without_cursor_move() {
    let limit = usize::try_from(kfnotepad::MAX_TEXT_FILE_BYTES).expect("text limit fits usize");
    let original = format!("x\n{}", "x".repeat(limit - 2));
    let mut document = TextDocument {
        path: PathBuf::from("typed-limit.txt"),
        buffer: TextBuffer::from_text(&original),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertChar('y'),
    );

    assert_eq!(document.buffer.to_text(), original);
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert!(!document.buffer.is_dirty());
}
