use super::*;

#[test]
fn buffer_starts_clean_and_preserves_lines() {
    let buffer = TextBuffer::from_text("alpha\nbeta\n");

    assert_eq!(buffer.lines(), &["alpha".to_string(), "beta".to_string()]);
    assert_eq!(buffer.line_count(), 2);
    assert!(buffer.has_trailing_newline());
    assert!(!buffer.is_dirty());
}

#[test]
fn buffer_inserts_unicode_by_character_column() {
    let mut buffer = TextBuffer::from_text("aé\n");

    buffer.insert_char(0, 2, '!').expect("insert char");

    assert_eq!(buffer.lines(), &["aé!".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn buffer_inserts_tab_and_moves_cursor_by_character_column() {
    let mut buffer = TextBuffer::from_text("ab\n");

    buffer.insert_char(0, 2, '\t').expect("insert tab");

    assert_eq!(buffer.to_text(), "ab\t\n");
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 2 }, CursorMove::Right),
        Ok(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn buffer_backspace_removes_combining_mark_cluster() {
    let mut buffer = TextBuffer::from_text("e\u{301}x\n");

    let cursor = buffer
        .delete_before_cursor(Cursor { row: 0, column: 2 })
        .expect("delete combined character");
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(buffer.to_text(), "x\n");
}

#[test]
fn buffer_backspace_removes_zwj_emoji_cluster() {
    let family = "👨‍👩‍👧‍👦";
    let mut buffer = TextBuffer::from_text(&format!("{family}\n"));
    let mut cursor = Cursor {
        row: 0,
        column: family.chars().count(),
    };

    cursor = buffer
        .delete_before_cursor(cursor)
        .expect("delete emoji cluster");

    assert_eq!(cursor.row, 0);
    assert_eq!(cursor.column, 0);
    assert_eq!(buffer.to_text(), "\n");
}

#[test]
fn buffer_cursor_moves_by_grapheme_clusters() {
    let mut buffer = TextBuffer::from_text("e\u{301}x\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Right),
        Ok(Cursor { row: 0, column: 2 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 2 }, CursorMove::Left),
        Ok(Cursor { row: 0, column: 0 })
    );

    let family = "👨‍👩‍👧‍👦";
    buffer.replace_text(&format!("{family}!\n"));
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Right),
        Ok(Cursor {
            row: 0,
            column: family.chars().count(),
        })
    );
}

#[test]
fn buffer_delete_and_replace_use_grapheme_clusters() {
    let flag = "🇺🇸";
    let mut buffer = TextBuffer::from_text(&format!("a{flag}b\n"));

    buffer.delete_char(0, 1).expect("delete flag cluster");
    assert_eq!(buffer.to_text(), "ab\n");

    let mut buffer = TextBuffer::from_text("e\u{301}x\n");
    buffer
        .replace_char(0, 0, 'E')
        .expect("replace combined character");
    assert_eq!(buffer.to_text(), "Ex\n");
}

#[test]
fn buffer_grapheme_boundary_column_snaps_inside_clusters() {
    let flag = "🇺🇸";
    let buffer = TextBuffer::from_text(&format!("{flag}x\n"));

    assert_eq!(buffer.grapheme_boundary_column(0, 0), Ok(0));
    assert_eq!(buffer.grapheme_boundary_column(0, 1), Ok(2));
    assert_eq!(buffer.grapheme_boundary_column(0, 2), Ok(2));
}

#[test]
fn buffer_grapheme_range_boundary_columns_expand_inside_clusters() {
    let flag = "🇺🇸";
    let buffer = TextBuffer::from_text(&format!("{flag}x\n"));

    assert_eq!(buffer.grapheme_range_boundary_columns(0, 1, 2), Ok((0, 2)));
    assert_eq!(buffer.grapheme_range_boundary_columns(0, 2, 3), Ok((2, 3)));
}

#[test]
fn buffer_splits_line_without_losing_text() {
    let mut buffer = TextBuffer::from_text("abcdef");

    buffer.insert_newline(0, 3).expect("insert newline");

    assert_eq!(buffer.lines(), &["abc".to_string(), "def".to_string()]);
    assert!(!buffer.has_trailing_newline());
    assert!(buffer.is_dirty());
}

#[test]
fn buffer_replace_text_marks_dirty_only_when_changed() {
    let mut buffer = TextBuffer::from_text("alpha\n");

    buffer.replace_text("alpha\n");
    assert_eq!(buffer.to_text(), "alpha\n");
    assert!(!buffer.is_dirty());

    buffer.replace_text("beta");
    assert_eq!(buffer.lines(), &["beta".to_string()]);
    assert_eq!(buffer.to_text(), "beta");
    assert!(buffer.is_dirty());

    buffer.mark_clean();
    buffer.replace_text("beta\n");
    assert_eq!(buffer.to_text(), "beta\n");
    assert!(buffer.has_trailing_newline());
    assert!(buffer.is_dirty());
}

#[test]
fn buffer_rejects_out_of_range_position() {
    let mut buffer = TextBuffer::from_text("abc");

    assert_eq!(
        buffer.insert_char(0, 4, '!'),
        Err(BufferError::ColumnOutOfBounds {
            column: 4,
            columns: 3
        })
    );
}

#[test]
fn cursor_moves_horizontally_by_character_column() {
    let buffer = TextBuffer::from_text("aé\nz");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 1 }, CursorMove::Right),
        Ok(Cursor { row: 0, column: 2 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 2 }, CursorMove::Right),
        Ok(Cursor { row: 1, column: 0 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 1, column: 0 }, CursorMove::Left),
        Ok(Cursor { row: 0, column: 2 })
    );
}

#[test]
fn cursor_moves_right_by_word_boundaries() {
    let buffer = TextBuffer::from_text("alpha, beta_gamma 42\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
        Ok(Cursor { row: 0, column: 7 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 7 }, CursorMove::WordRight),
        Ok(Cursor { row: 0, column: 18 })
    );
}

#[test]
fn cursor_moves_left_by_word_boundaries() {
    let buffer = TextBuffer::from_text("alpha, beta gamma\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 17 }, CursorMove::WordLeft),
        Ok(Cursor { row: 0, column: 12 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 12 }, CursorMove::WordLeft),
        Ok(Cursor { row: 0, column: 7 })
    );
}

#[test]
fn cursor_word_movement_crosses_lines_and_handles_unicode() {
    let buffer = TextBuffer::from_text("héllo\n  世界 next\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
        Ok(Cursor { row: 1, column: 2 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 1, column: 5 }, CursorMove::WordLeft),
        Ok(Cursor { row: 1, column: 2 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 1, column: 2 }, CursorMove::WordLeft),
        Ok(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn cursor_word_movement_respects_grapheme_clusters() {
    let buffer = TextBuffer::from_text("e\u{301}, next\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 2 }, CursorMove::WordLeft),
        Ok(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
        Ok(Cursor { row: 0, column: 4 })
    );

    let family = "👨‍👩‍👧‍👦";
    let buffer = TextBuffer::from_text(&format!("{family} next\n"));
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
        Ok(Cursor {
            row: 0,
            column: family.chars().count() + 1,
        })
    );
}

#[test]
fn cursor_word_movement_clamps_at_document_edges() {
    let buffer = TextBuffer::from_text("alpha\n");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordLeft),
        Ok(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 5 }, CursorMove::WordRight),
        Ok(Cursor { row: 0, column: 5 })
    );
}

#[test]
fn buffer_deletes_previous_word_by_boundaries() {
    let mut buffer = TextBuffer::from_text("alpha, beta gamma\n");

    let moved = buffer
        .delete_previous_word(Cursor { row: 0, column: 17 })
        .expect("delete previous word");

    assert_eq!(moved, Cursor { row: 0, column: 12 });
    assert_eq!(buffer.line(0), Some("alpha, beta "));
    assert!(buffer.is_dirty());
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.line(0), Some("alpha, beta gamma"));
}

#[test]
fn buffer_deletes_next_word_by_boundaries() {
    let mut buffer = TextBuffer::from_text("alpha, beta gamma\n");

    let moved = buffer
        .delete_next_word(Cursor { row: 0, column: 5 })
        .expect("delete next word");

    assert_eq!(moved, Cursor { row: 0, column: 5 });
    assert_eq!(buffer.line(0), Some("alpha gamma"));
    assert!(buffer.is_dirty());
}

#[test]
fn buffer_deletes_to_line_end_by_character_column() {
    let mut buffer = TextBuffer::from_text("héllo world\nnext\n");

    let moved = buffer
        .delete_to_line_end(Cursor { row: 0, column: 2 })
        .expect("delete to line end");

    assert_eq!(moved, Cursor { row: 0, column: 2 });
    assert_eq!(buffer.lines(), &["hé".to_string(), "next".to_string()]);
    assert!(buffer.is_dirty());
    assert!(buffer.undo_last_edit());
    assert_eq!(
        buffer.lines(),
        &["héllo world".to_string(), "next".to_string()]
    );
}

#[test]
fn buffer_word_deletion_crosses_lines_and_handles_unicode() {
    let mut buffer = TextBuffer::from_text("héllo\n  世界 next\n");

    let moved = buffer
        .delete_next_word(Cursor { row: 0, column: 5 })
        .expect("delete next word across line");

    assert_eq!(moved, Cursor { row: 0, column: 5 });
    assert_eq!(buffer.lines(), &["héllo next".to_string()]);
    assert!(buffer.has_trailing_newline());
    assert!(buffer.undo_last_edit());
    assert_eq!(
        buffer.lines(),
        &["héllo".to_string(), "  世界 next".to_string()]
    );

    let moved = buffer
        .delete_previous_word(Cursor { row: 1, column: 5 })
        .expect("delete previous word across line");

    assert_eq!(moved, Cursor { row: 1, column: 2 });
    assert_eq!(buffer.lines(), &["héllo".to_string(), "  next".to_string()]);
}

#[test]
fn buffer_word_deletion_respects_grapheme_clusters() {
    let mut buffer = TextBuffer::from_text("e\u{301} next\n");
    let moved = buffer
        .delete_next_word(Cursor { row: 0, column: 0 })
        .expect("delete combined word");
    assert_eq!(moved, Cursor { row: 0, column: 0 });
    assert_eq!(buffer.to_text(), " next\n");

    let family = "👨‍👩‍👧‍👦";
    let mut buffer = TextBuffer::from_text(&format!("{family} next\n"));
    let moved = buffer
        .delete_next_word(Cursor { row: 0, column: 0 })
        .expect("delete next word after emoji cluster");
    assert_eq!(moved, Cursor { row: 0, column: 0 });
    assert_eq!(buffer.to_text(), "\n");

    let mut buffer = TextBuffer::from_text("start e\u{301}\n");
    let moved = buffer
        .delete_previous_word(Cursor { row: 0, column: 8 })
        .expect("delete previous combined word");
    assert_eq!(moved, Cursor { row: 0, column: 6 });
    assert_eq!(buffer.to_text(), "start \n");
}

#[test]
fn cursor_stays_inside_buffer_edges() {
    let buffer = TextBuffer::from_text("abc");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Left),
        Ok(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 3 }, CursorMove::Right),
        Ok(Cursor { row: 0, column: 3 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Up),
        Ok(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Down),
        Ok(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn cursor_vertical_movement_clamps_to_target_line_length() {
    let buffer = TextBuffer::from_text("abcd\né\nxyz");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 4 }, CursorMove::Down),
        Ok(Cursor { row: 1, column: 1 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 1, column: 1 }, CursorMove::Down),
        Ok(Cursor { row: 2, column: 1 })
    );
    assert_eq!(
        buffer.move_cursor(Cursor { row: 2, column: 3 }, CursorMove::Up),
        Ok(Cursor { row: 1, column: 1 })
    );
}

#[test]
fn cursor_rejects_invalid_start_position() {
    let buffer = TextBuffer::from_text("abc");

    assert_eq!(
        buffer.move_cursor(Cursor { row: 0, column: 4 }, CursorMove::Right),
        Err(BufferError::ColumnOutOfBounds {
            column: 4,
            columns: 3
        })
    );
}

#[test]
fn delete_removes_character_at_cursor() {
    let mut buffer = TextBuffer::from_text("aé!");

    buffer.delete_char(0, 1).expect("delete char");

    assert_eq!(buffer.lines(), &["a!".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn delete_at_line_end_joins_next_line() {
    let mut buffer = TextBuffer::from_text("abc\ndef");

    buffer.delete_char(0, 3).expect("delete newline");

    assert_eq!(buffer.lines(), &["abcdef".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn replace_char_replaces_one_character_and_undoes_as_one_edit() {
    let mut buffer = TextBuffer::from_text("aé!");

    buffer.replace_char(0, 1, 'x').expect("replace char");

    assert_eq!(buffer.lines(), &["ax!".to_string()]);
    assert!(buffer.is_dirty());
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["aé!".to_string()]);
    assert!(!buffer.undo_last_edit());
}

#[test]
fn replace_char_replaces_whole_grapheme_cluster() {
    let mut buffer = TextBuffer::from_text("a🇺🇸e\u{301}!");

    buffer.replace_char(0, 1, 'x').expect("replace flag");
    assert_eq!(buffer.lines(), &["axe\u{301}!".to_string()]);

    buffer
        .replace_char(0, 2, 'y')
        .expect("replace combining grapheme");
    assert_eq!(buffer.lines(), &["axy!".to_string()]);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["axe\u{301}!".to_string()]);
    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["a🇺🇸e\u{301}!".to_string()]);
}

#[test]
fn replace_char_at_line_end_inserts_without_deleting_newline() {
    let mut buffer = TextBuffer::from_text("abc\ndef");

    buffer.replace_char(0, 3, '!').expect("insert at line end");

    assert_eq!(buffer.lines(), &["abc!".to_string(), "def".to_string()]);
}

#[test]
fn backspace_removes_character_before_cursor() {
    let mut buffer = TextBuffer::from_text("abc");

    let cursor = buffer
        .delete_before_cursor(Cursor { row: 0, column: 2 })
        .expect("backspace char");

    assert_eq!(cursor, Cursor { row: 0, column: 1 });
    assert_eq!(buffer.lines(), &["ac".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn backspace_at_line_start_joins_previous_line() {
    let mut buffer = TextBuffer::from_text("abc\ndef");

    let cursor = buffer
        .delete_before_cursor(Cursor { row: 1, column: 0 })
        .expect("backspace newline");

    assert_eq!(cursor, Cursor { row: 0, column: 3 });
    assert_eq!(buffer.lines(), &["abcdef".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn undo_restores_previous_text_after_insert() {
    let mut buffer = TextBuffer::from_text("abc");

    buffer.insert_char(0, 1, '!').expect("insert char");
    assert_eq!(buffer.lines(), &["a!bc".to_string()]);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["abc".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn redo_restores_undone_text() {
    let mut buffer = TextBuffer::from_text("abc");

    buffer.insert_char(0, 3, '!').expect("insert char");
    assert_eq!(buffer.lines(), &["abc!".to_string()]);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["abc".to_string()]);

    assert!(buffer.redo_last_undo());
    assert_eq!(buffer.lines(), &["abc!".to_string()]);
    assert!(buffer.is_dirty());
}

#[test]
fn new_edit_clears_redo_history() {
    let mut buffer = TextBuffer::from_text("abc");

    buffer.insert_char(0, 3, '!').expect("insert char");
    assert!(buffer.undo_last_edit());
    buffer
        .insert_char(0, 3, '?')
        .expect("insert replacement char");

    assert!(!buffer.redo_last_undo());
    assert_eq!(buffer.lines(), &["abc?".to_string()]);
}

#[test]
fn undo_restores_previous_text_after_line_join() {
    let mut buffer = TextBuffer::from_text("abc\ndef");

    buffer.delete_char(0, 3).expect("delete newline");
    assert_eq!(buffer.lines(), &["abcdef".to_string()]);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.lines(), &["abc".to_string(), "def".to_string()]);
}

#[test]
fn mark_clean_clears_undo_history() {
    let mut buffer = TextBuffer::from_text("abc");

    buffer.insert_char(0, 3, '!').expect("insert char");
    buffer.mark_clean();

    assert!(!buffer.undo_last_edit());
    assert!(!buffer.redo_last_undo());
    assert_eq!(buffer.lines(), &["abc!".to_string()]);
    assert!(!buffer.is_dirty());
}
