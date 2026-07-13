use super::*;

#[test]
fn consecutive_typed_inserts_coalesce_as_one_undo_step() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert second");
    buffer.insert_char(0, 2, 'c').expect("insert third");

    assert_eq!(buffer.to_text(), "abc");
    assert_eq!(buffer.undo_history.len(), 1);
    assert!(matches!(
        buffer.undo_history.back(),
        Some(HistoryEntry::InsertText { text, .. }) if text == "abc"
    ));

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "abc");
}

#[test]
fn consecutive_unicode_typed_inserts_coalesce_and_redo_exactly() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, '\u{754c}').expect("insert CJK");
    buffer.insert_char(0, 1, '\u{1f642}').expect("insert emoji");
    buffer
        .insert_char(0, 2, '\u{301}')
        .expect("insert combining mark");

    assert_eq!(buffer.to_text(), "\u{754c}\u{1f642}\u{301}");
    assert_eq!(buffer.undo_history.len(), 1);
    assert!(buffer.undo_bytes < 256);
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "\u{754c}\u{1f642}\u{301}");
}

#[test]
fn insert_newline_breaks_typing_undo_group() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert second");
    buffer.insert_newline(0, 2).expect("insert newline");
    buffer.insert_char(1, 0, 'c').expect("insert after newline");

    assert_eq!(buffer.to_text(), "ab\nc");
    assert_eq!(buffer.undo_history.len(), 3);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "ab\n");
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "ab");
}

#[test]
fn cursor_jumps_break_typed_insert_undo_coalescing() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert contiguous");
    buffer.insert_char(0, 0, 'c').expect("cursor jump insert");

    assert_eq!(buffer.to_text(), "cab");
    assert_eq!(buffer.undo_history.len(), 2);
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "ab");
}

#[test]
fn explicit_undo_boundary_breaks_typed_insert_coalescing() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert second");
    assert_eq!(buffer.undo_history.len(), 1);

    buffer.break_undo_group();

    buffer
        .insert_char(0, 2, 'c')
        .expect("insert after boundary");
    assert_eq!(buffer.undo_history.len(), 2);
    buffer
        .insert_char(0, 3, 'd')
        .expect("insert after boundary contiguous");
    assert_eq!(buffer.undo_history.len(), 2);
}

#[test]
fn coalescing_timeout_breaks_typed_insert_undo_group() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert contiguous");
    std::thread::sleep(TYPING_UNDO_COALESCE_WINDOW + Duration::from_millis(25));
    buffer.insert_char(0, 2, 'c').expect("insert after timeout");

    assert_eq!(buffer.to_text(), "abc");
    assert_eq!(buffer.undo_history.len(), 2);
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "ab");
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "");
}

#[test]
fn compound_edit_records_one_snapshot_for_multiple_edit_kinds() {
    let mut document = TextDocument {
        path: PathBuf::from("compound.txt"),
        buffer: TextBuffer::from_text("hello world"),
    };

    document.with_compound_edit(|document| {
        document
            .buffer
            .delete_range(Cursor { row: 0, column: 5 }, Cursor { row: 0, column: 11 })
            .expect("delete selection");
        document
            .buffer
            .insert_newline(0, 5)
            .expect("insert newline");
        document
            .buffer
            .insert_text(Cursor { row: 1, column: 0 }, "x")
            .expect("insert bulk text");
        document
            .buffer
            .insert_char(1, 1, 'y')
            .expect("insert typed character");
    });

    assert_eq!(document.buffer.to_text(), "hello\nxy");
    assert_eq!(document.buffer.undo_history.len(), 1);
    assert!(matches!(
        document.buffer.undo_history.back(),
        Some(HistoryEntry::Snapshot(_))
    ));
    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.to_text(), "hello world");
    assert!(!document.buffer.undo_last_edit());
}

#[test]
fn compound_edit_without_buffer_changes_does_not_create_undo_history() {
    let mut document = TextDocument {
        path: PathBuf::from("unchanged.txt"),
        buffer: TextBuffer::from_text("unchanged"),
    };

    document.with_compound_edit(|_| {});

    assert!(document.buffer.undo_history.is_empty());
    assert_eq!(document.buffer.undo_bytes, 0);
    assert!(!document.buffer.is_dirty());
}

#[test]
fn bulk_insert_text_updates_lines_cursor_revision_and_undo_once() {
    let mut buffer = TextBuffer::from_text("hello world");
    let initial_revision = buffer.edit_revision();

    let cursor = buffer
        .insert_text(Cursor { row: 0, column: 5 }, " there\nfriend\n")
        .expect("insert multiline text");

    assert_eq!(buffer.to_text(), "hello there\nfriend\n world");
    assert_eq!(cursor, Cursor { row: 2, column: 0 });
    assert_eq!(buffer.edit_revision(), initial_revision.wrapping_add(1));
    assert_eq!(buffer.undo_history.len(), 1);
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "hello world");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "hello there\nfriend\n world");
}

#[test]
fn bulk_insert_text_advances_to_combining_grapheme_end() {
    let mut buffer = TextBuffer::from_text("e");

    let cursor = buffer
        .insert_text(Cursor { row: 0, column: 1 }, "\u{301}")
        .expect("insert combining mark");

    assert_eq!(buffer.to_text(), "e\u{301}");
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "e");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "e\u{301}");
}

#[test]
fn bulk_ascii_insert_before_combining_mark_still_normalizes_cursor() {
    let mut buffer = TextBuffer::from_text("\u{301}x");

    let cursor = buffer
        .insert_text(Cursor { row: 0, column: 0 }, "e")
        .expect("insert base before combining mark");

    assert_eq!(buffer.to_text(), "e\u{301}x");
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "\u{301}x");
}

#[test]
fn bulk_insert_delta_undo_preserves_adjacent_graphemes_and_trailing_newline() {
    let mut buffer = TextBuffer::from_text("eX\n");

    buffer
        .insert_text(Cursor { row: 0, column: 1 }, "\u{301}\nfriend")
        .expect("insert text ending beside existing content");
    assert_eq!(buffer.to_text(), "e\u{301}\nfriendX\n");
    assert!(buffer.has_trailing_newline());

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "eX\n");
    assert!(buffer.has_trailing_newline());

    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "e\u{301}\nfriendX\n");
    assert!(buffer.has_trailing_newline());
}

#[test]
fn bulk_insert_delta_history_scales_with_insert_instead_of_document() {
    let base = "a".repeat(128 * 1024);
    let inserted = "paste".repeat(4 * 1024);
    let mut buffer = TextBuffer::from_text(&base);

    buffer
        .insert_text(
            Cursor {
                row: 0,
                column: base.len(),
            },
            &inserted,
        )
        .expect("insert 20 KiB paste");

    assert_eq!(buffer.undo_history.len(), 1);
    assert!(buffer.undo_bytes >= inserted.len());
    assert!(buffer.undo_bytes < base.len());
    assert!(matches!(
        buffer.undo_history.back(),
        Some(HistoryEntry::InsertText { .. })
    ));
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), base);
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), format!("{base}{inserted}"));
}

#[test]
fn new_edit_clears_bulk_insert_delta_redo_history() {
    let mut buffer = TextBuffer::from_text("base");

    buffer
        .insert_text(Cursor { row: 0, column: 4 }, " paste")
        .expect("insert paste");
    assert!(buffer.undo_last_edit());
    buffer.insert_char(0, 4, '!').expect("insert new edit");

    assert!(!buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "base!");
}

#[test]
fn multiline_unicode_delete_delta_undoes_and_redoes_exactly() {
    let mut buffer = TextBuffer::from_text("a\u{1f1fa}\u{1f1f8}\nmid\ne\u{301}z\n");

    buffer
        .delete_range(Cursor { row: 0, column: 1 }, Cursor { row: 2, column: 1 })
        .expect("delete multiline Unicode range");

    assert_eq!(buffer.to_text(), "az\n");
    assert!(matches!(
        buffer.undo_history.back(),
        Some(HistoryEntry::DeleteText { text, .. })
            if text == "\u{1f1fa}\u{1f1f8}\nmid\ne\u{301}"
    ));
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "a\u{1f1fa}\u{1f1f8}\nmid\ne\u{301}z\n");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "az\n");
}

#[test]
fn newline_join_delete_delta_restores_line_boundary() {
    let mut buffer = TextBuffer::from_text("abc\ndef");

    buffer.delete_char(0, 3).expect("delete line boundary");
    assert_eq!(buffer.to_text(), "abcdef");
    assert!(matches!(
        buffer.undo_history.back(),
        Some(HistoryEntry::DeleteText { text, .. }) if text == "\n"
    ));
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "abc\ndef");
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "abcdef");
}

#[cfg(feature = "gui")]
#[test]
fn replacement_delete_delta_restores_trailing_newline_policy() {
    let mut buffer = TextBuffer::from_text("abc\n");

    buffer
        .delete_replacement_range(Cursor { row: 0, column: 1 }, Cursor { row: 0, column: 2 })
        .expect("delete replacement selection");
    assert_eq!(buffer.to_text(), "ac");
    assert!(!buffer.has_trailing_newline());

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "abc\n");
    assert!(buffer.has_trailing_newline());
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text(), "ac");
    assert!(!buffer.has_trailing_newline());
}

#[test]
fn delete_delta_history_scales_with_removed_text_instead_of_document() {
    let base = "a".repeat(1024 * 1024);
    let removed_bytes = 20 * 1024;
    let mut buffer = TextBuffer::from_text(&base);

    buffer
        .delete_range(
            Cursor {
                row: 0,
                column: base.len() - removed_bytes,
            },
            Cursor {
                row: 0,
                column: base.len(),
            },
        )
        .expect("delete 20 KiB range");

    assert_eq!(buffer.undo_history.len(), 1);
    assert!(buffer.undo_bytes >= removed_bytes);
    assert!(buffer.undo_bytes < base.len());
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), base);
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.byte_len(), base.len() - removed_bytes);
}

#[test]
fn bulk_insert_empty_text_is_unchanged() {
    let mut buffer = TextBuffer::from_text("unchanged");

    let cursor = buffer
        .insert_text(Cursor { row: 0, column: 4 }, "")
        .expect("accept empty insert");

    assert_eq!(cursor, Cursor { row: 0, column: 4 });
    assert_eq!(buffer.to_text(), "unchanged");
    assert!(!buffer.is_dirty());
    assert!(buffer.undo_history.is_empty());
}

#[test]
fn insert_operations_reject_growth_beyond_text_limit_without_mutation() {
    let limit = usize::try_from(MAX_TEXT_FILE_BYTES).expect("text limit fits usize");
    let original = "x".repeat(limit);
    let mut buffer = TextBuffer::from_text(&original);

    assert_eq!(
        buffer.insert_char(0, limit, 'y'),
        Err(BufferError::TooLarge {
            bytes: limit + 1,
            limit,
        })
    );
    assert_eq!(
        buffer.insert_newline(0, limit),
        Err(BufferError::TooLarge {
            bytes: limit + 1,
            limit,
        })
    );
    assert_eq!(
        buffer.insert_text(
            Cursor {
                row: 0,
                column: limit
            },
            "yz"
        ),
        Err(BufferError::TooLarge {
            bytes: limit + 2,
            limit,
        })
    );

    assert_eq!(buffer.to_text(), original);
    assert!(!buffer.is_dirty());
    assert!(buffer.undo_history.is_empty());
}

#[test]
fn overwrite_at_text_limit_allows_equal_bytes_and_rejects_growth() {
    let limit = usize::try_from(MAX_TEXT_FILE_BYTES).expect("text limit fits usize");
    let mut buffer = TextBuffer::from_text(&"x".repeat(limit));

    buffer
        .replace_char(0, 0, 'y')
        .expect("equal-byte overwrite remains within limit");
    assert_eq!(buffer.byte_len(), limit);
    assert_eq!(
        buffer.replace_char(0, 1, '界'),
        Err(BufferError::TooLarge {
            bytes: limit + 2,
            limit,
        })
    );
    assert_eq!(
        buffer.line(0).and_then(|line| line.chars().nth(1)),
        Some('x')
    );
}

#[test]
fn large_file_undo_history_uses_byte_budget_and_remains_responsive() {
    let base_size = usize::try_from(MAX_TEXT_FILE_BYTES.saturating_sub(1024))
        .expect("max text byte limit fits in usize");
    let initial_text = "a".repeat(base_size);
    let mut buffer = TextBuffer::from_text(&initial_text);
    assert_eq!(buffer.to_text().len(), base_size);
    assert_eq!(buffer.to_text().chars().count(), base_size);

    buffer.mark_clean();
    assert!(!buffer.is_dirty());

    for _ in 0..300 {
        buffer
            .insert_char(0, 0, 'x')
            .expect("insert near large document");
    }

    assert!(buffer.is_dirty());

    let undo_budget = buffer
        .undo_history
        .iter()
        .map(HistoryEntry::byte_size)
        .sum::<usize>();
    assert_eq!(buffer.undo_bytes, undo_budget);
    assert!(
        undo_budget <= MAX_UNDO_BYTES,
        "undo budget {undo_budget} exceeds hard cap {MAX_UNDO_BYTES}"
    );

    assert!(buffer.undo_last_edit());
    let after_edit_len = buffer.to_text().len();
    assert!(after_edit_len > base_size);
    assert!(after_edit_len <= base_size + 300);

    let mut undo_steps = 0usize;
    while buffer.undo_last_edit() {
        undo_steps += 1;
    }

    assert_eq!(undo_steps, MAX_UNDO_HISTORY - 1);
    assert_eq!(buffer.to_text().len(), initial_text.len() + 44);
}

#[test]
fn undo_history_is_bounded_and_redo_still_restores_latest_edit() {
    let mut buffer = TextBuffer::from_text("");

    for _ in 0..(MAX_UNDO_HISTORY + 10) {
        buffer.insert_char(0, 0, 'x').expect("insert");
    }

    assert_eq!(buffer.undo_history.len(), MAX_UNDO_HISTORY);
    assert!(buffer.undo_last_edit());
    let after_undo = buffer.to_text();
    assert_eq!(after_undo.len(), MAX_UNDO_HISTORY + 9);
    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.to_text().len(), MAX_UNDO_HISTORY + 10);
}

#[test]
fn history_push_prefers_latest_entries_and_tracks_byte_budget() {
    let snapshots = [
        BufferSnapshot {
            lines: vec!["a".to_string()],
            trailing_newline: false,
            byte_size: 60,
        },
        BufferSnapshot {
            lines: vec!["b".to_string()],
            trailing_newline: false,
            byte_size: 60,
        },
        BufferSnapshot {
            lines: vec!["c".to_string()],
            trailing_newline: false,
            byte_size: 60,
        },
        BufferSnapshot {
            lines: vec!["d".to_string()],
            trailing_newline: false,
            byte_size: 60,
        },
    ];
    let mut history = VecDeque::new();
    let mut used_bytes = 0;
    for snapshot in snapshots {
        push_history_snapshot(&mut history, &mut used_bytes, snapshot, 4, 120);
    }

    assert_eq!(history.len(), 2);
    assert_eq!(used_bytes, 120);
    assert!(matches!(
        &history[0],
        HistoryEntry::Snapshot(snapshot) if snapshot.lines[0] == "c"
    ));
    assert!(matches!(
        &history[1],
        HistoryEntry::Snapshot(snapshot) if snapshot.lines[0] == "d"
    ));
}
