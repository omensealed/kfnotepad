use super::settings::path_to_hex;
use super::*;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kfnotepad-lib-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create temp area");
        let root = root.canonicalize().expect("canonicalize temp area");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TempArea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_no_temp_files(directory: &Path) {
    let entries = fs::read_dir(directory).expect("read temp dir");
    for entry in entries {
        let entry = entry.expect("read temp entry");
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.contains(".kfnotepad-"),
            "unexpected temporary config file left behind: {file_name}"
        );
    }
}

#[test]
fn parses_help_flag() {
    assert_eq!(parse_args(&["--help".to_string()]), Ok(Command::Help));
}

#[test]
fn parses_managed_notes_flags() {
    assert_eq!(
        parse_args(&["--notes".to_string()]),
        Ok(Command::ListManagedNotes)
    );
    assert_eq!(
        parse_args(&["--note".to_string(), "Daily Note".to_string()]),
        Ok(Command::OpenManagedNote("Daily Note".to_string()))
    );
    assert_eq!(
        parse_args(&["--note".to_string(), "   ".to_string()]),
        Err("managed note title must not be empty".to_string())
    );
}

#[test]
fn rejects_unknown_option() {
    assert_eq!(
        parse_args(&["--bogus".to_string()]),
        Err("unknown option: --bogus".to_string())
    );
}

#[test]
fn summarizes_text_without_mutation() {
    assert_eq!(
        summarize_text("one\ntwo\n"),
        FileSummary {
            bytes: 8,
            lines: 2,
            trailing_newline: true
        }
    );
}

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

#[test]
fn find_next_finds_query_from_cursor() {
    let buffer = TextBuffer::from_text("alpha\nbeta alphabet\n");

    assert_eq!(
        buffer.find_next("alpha", Cursor { row: 0, column: 1 }),
        Some(Cursor { row: 1, column: 5 })
    );
}

#[test]
fn find_next_wraps_to_top() {
    let buffer = TextBuffer::from_text("first match\nsecond\n");

    assert_eq!(
        buffer.find_next("first", Cursor { row: 1, column: 0 }),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_next_handles_unicode_columns() {
    let buffer = TextBuffer::from_text("aé match\n");

    assert_eq!(
        buffer.find_next("match", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_next_finds_tabs_and_emoji() {
    let buffer = TextBuffer::from_text("alpha\t🏳️‍🌈\tbeta\n");

    assert_eq!(
        buffer.find_next("🏳️‍🌈", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 6 })
    );
    assert_eq!(
        buffer.find_next("beta", Cursor { row: 0, column: 7 }),
        Some(Cursor { row: 0, column: 11 })
    );
}

#[test]
fn find_next_expands_partial_grapheme_matches_to_cluster_start() {
    let buffer = TextBuffer::from_text("🇺🇸 e\u{301}x\n");

    assert_eq!(
        buffer.find_next("🇸", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.find_next("\u{301}", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_previous_expands_partial_grapheme_matches_to_cluster_start() {
    let buffer = TextBuffer::from_text("🇺🇸 e\u{301}x\n");

    assert_eq!(
        buffer.find_previous("🇸", Cursor { row: 0, column: 2 }),
        Some(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.find_previous("\u{301}", Cursor { row: 0, column: 5 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_previous_handles_tabs_and_emoji() {
    let buffer = TextBuffer::from_text("alpha\t🏳️‍🌈\tbeta\n");

    assert_eq!(
        buffer.find_previous("🏳️‍🌈", Cursor { row: 0, column: 14 }),
        Some(Cursor { row: 0, column: 6 })
    );
    assert_eq!(
        buffer.find_previous("alpha", Cursor { row: 0, column: 14 }),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_next_can_ignore_case_without_changing_default_search() {
    let buffer = TextBuffer::from_text("Alpha\nbeta alpha\n");

    assert_eq!(
        buffer.find_next("alpha", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 1, column: 5 })
    );
    assert_eq!(
        buffer.find_next_with_mode(
            "alpha",
            Cursor { row: 0, column: 0 },
            SearchMode {
                case_sensitive: false,
            },
        ),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_case_insensitive_maps_expanded_unicode_lowercase_to_original_columns() {
    let buffer = TextBuffer::from_text("aßb SS\nİstanbul\n");
    let insensitive = SearchMode {
        case_sensitive: false,
    };

    assert_eq!(
        buffer.find_next_with_mode("ss", Cursor { row: 0, column: 0 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_next_with_mode("i", Cursor { row: 1, column: 0 }, insensitive),
        Some(Cursor { row: 1, column: 0 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("ss", Cursor { row: 0, column: 6 }, insensitive),
        Some(Cursor { row: 0, column: 4 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("ss", Cursor { row: 0, column: 3 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
}

#[test]
fn find_case_insensitive_handles_partial_folded_graphemes_without_duplicates() {
    let buffer = TextBuffer::from_text("aßb\nİx\n");
    let insensitive = SearchMode {
        case_sensitive: false,
    };

    assert_eq!(
        buffer.find_next_with_mode("s", Cursor { row: 0, column: 0 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("s", Cursor { row: 0, column: 3 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_next_with_mode("\u{307}", Cursor { row: 1, column: 0 }, insensitive),
        Some(Cursor { row: 1, column: 0 })
    );
}

#[test]
fn find_previous_finds_query_before_cursor() {
    let buffer = TextBuffer::from_text("alpha\nbeta alpha\n");

    assert_eq!(
        buffer.find_previous("alpha", Cursor { row: 1, column: 10 }),
        Some(Cursor { row: 1, column: 5 })
    );
}

#[test]
fn find_previous_wraps_to_bottom() {
    let buffer = TextBuffer::from_text("first\nsecond match\n");

    assert_eq!(
        buffer.find_previous("match", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 1, column: 7 })
    );
}

#[test]
fn find_previous_handles_unicode_columns() {
    let buffer = TextBuffer::from_text("aé match\n");

    assert_eq!(
        buffer.find_previous("match", Cursor { row: 0, column: 8 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn repeat_search_with_mode_wraps_and_honors_case() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("Alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 1, column: 4 };

    assert_eq!(
        repeat_search_next_with_mode(
            &document,
            &mut cursor,
            "alpha",
            SearchMode {
                case_sensitive: false,
            },
        ),
        SearchRepeatResult::Found {
            query: "alpha".to_string()
        }
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}

#[test]
fn editor_tab_state_starts_at_document_origin() {
    assert_eq!(
        EditorTabState::default(),
        EditorTabState {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
        }
    );
}

#[test]
fn editor_workspace_keeps_borrowed_document_editable() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha\n"),
    };

    {
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let active_tab = workspace.active_tab_mut();
        active_tab.state.cursor = Cursor { row: 0, column: 5 };
        active_tab
            .document
            .as_mut()
            .buffer
            .insert_char(0, 5, '!')
            .expect("insert through active tab");

        assert_eq!(active_tab.state.cursor, Cursor { row: 0, column: 5 });
        assert!(active_tab.document.as_ref().buffer.is_dirty());
    }

    assert_eq!(document.buffer.to_text(), "alpha!\n");
    assert!(document.buffer.is_dirty());
}

#[test]
fn editor_workspace_switches_and_closes_tabs_without_terminal_types() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut first);

    assert!(!workspace.select_next_tab());
    workspace.push_owned_tab(second);
    assert_eq!(workspace.active, 1);
    assert_eq!(
        workspace.tab_strip_items(),
        vec![
            TabStripItem {
                label: String::from("first.txt"),
                active: false,
                dirty: false,
            },
            TabStripItem {
                label: String::from("second.txt"),
                active: true,
                dirty: false,
            },
        ]
    );

    assert!(workspace.select_next_tab());
    assert_eq!(workspace.active, 0);
    assert!(workspace.select_previous_tab());
    assert_eq!(workspace.active, 1);

    assert_eq!(
        workspace.close_active_tab(false),
        CloseActiveTabResult::Closed {
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
}

#[test]
fn editor_workspace_requires_confirmation_before_dirty_tab_close() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    second
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty second tab");
    let mut workspace = EditorWorkspace::from_document(&mut first);
    workspace.push_owned_tab(second);

    assert_eq!(
        workspace.close_active_tab(false),
        CloseActiveTabResult::Dirty
    );
    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(
        workspace.close_active_tab(true),
        CloseActiveTabResult::Closed {
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tabs.len(), 1);
}

#[test]
fn shared_document_commands_edit_and_clamp_cursor_without_terminal_types() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha beta\nsecond\n"),
    };
    let mut cursor = Cursor { row: 0, column: 10 };

    assert_eq!(
        delete_previous_word(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        undo_document_edit(&mut document, &mut cursor),
        UndoRedoResult::Applied
    );
    assert_eq!(document.buffer.to_text(), "alpha beta\nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        redo_document_edit(&mut document, &mut cursor),
        UndoRedoResult::Applied
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        delete_to_line_end(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    cursor.column = 0;
    assert_eq!(
        delete_next_word(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), " \nsecond\n");
}

#[test]
fn shared_navigation_commands_move_without_terminal_types() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha beta\nsecond line\nthird\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };

    move_document_cursor(&document, &mut cursor, CursorMove::WordRight);
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    page_down(&document, &mut cursor, 2);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });

    go_to_document_end(&document, &mut cursor);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });

    page_up(&document, &mut cursor, 10);
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    go_to_document_start(&mut cursor);
    assert_eq!(cursor, Cursor { row: 0, column: 0 });

    cursor = Cursor { row: 0, column: 99 };
    assert_eq!(
        go_to_line(&document, &mut cursor, "2"),
        GoToLineResult::Moved { line_number: 2 }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 11 });
    assert_eq!(
        go_to_line(&document, &mut cursor, ""),
        GoToLineResult::Empty
    );
    assert_eq!(
        go_to_line(&document, &mut cursor, "abc"),
        GoToLineResult::Invalid
    );
    assert_eq!(
        go_to_line(&document, &mut cursor, "99"),
        GoToLineResult::OutOfRange { line_number: 99 }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 11 });
}

#[test]
fn shared_repeat_search_updates_cursor_and_reports_result() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };

    assert_eq!(
        repeat_search_next(&document, &mut cursor, ""),
        SearchRepeatResult::NoPreviousSearch
    );
    assert_eq!(
        repeat_search_next(&document, &mut cursor, "alpha"),
        SearchRepeatResult::Found {
            query: String::from("alpha"),
        }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(
        repeat_search_previous(&document, &mut cursor, "alpha"),
        SearchRepeatResult::Found {
            query: String::from("alpha"),
        }
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(
        repeat_search_next(&document, &mut cursor, "missing"),
        SearchRepeatResult::NoMatch {
            query: String::from("missing"),
        }
    );
}

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
fn shared_file_sidebar_lists_parent_dirs_and_files_in_order() {
    let temp = TempArea::new("sidebar-list");
    fs::create_dir(temp.path("z-dir")).expect("create z dir");
    fs::create_dir(temp.path("a-dir")).expect("create a dir");
    fs::write(temp.path("z.txt"), "z\n").expect("write z file");
    fs::write(temp.path("a.txt"), "a\n").expect("write a file");

    let sidebar = FileSidebarState::load(temp.root.clone()).expect("load sidebar");
    let labels: Vec<_> = sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();

    assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);
    assert_eq!(sidebar.selected_entry().expect("selected").label, "../");
}

#[test]
fn shared_file_sidebar_loads_subdirectories_and_parent_entries() {
    let temp = TempArea::new("sidebar-nav");
    fs::create_dir(temp.path("sub")).expect("create sub dir");
    fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write sub file");

    let sidebar = FileSidebarState::load(temp.root.clone()).expect("load root sidebar");
    let sub = sidebar
        .entries
        .iter()
        .find(|entry| entry.label == "sub/")
        .expect("subdirectory entry")
        .clone();
    assert_eq!(sub.kind, FileSidebarEntryKind::Directory);

    let sub_sidebar = FileSidebarState::load(sub.path).expect("load sub sidebar");
    assert_eq!(
        sub_sidebar.current_dir,
        temp.path("sub")
            .canonicalize()
            .expect("canonicalize subdirectory")
    );
    assert_eq!(
        sub_sidebar.entries.first().expect("parent entry").kind,
        FileSidebarEntryKind::Parent
    );
}

#[test]
fn shared_file_sidebar_selection_wraps_and_scrolls_without_terminal_types() {
    let mut sidebar = FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..5)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 0,
    };

    sidebar.select_previous_wrapping(3);
    assert_eq!(sidebar.selected, 4);
    assert_eq!(sidebar.scroll, 2);

    sidebar.select_next_wrapping(3);
    assert_eq!(sidebar.selected, 0);
    assert_eq!(sidebar.scroll, 0);

    assert!(sidebar.scroll_selection_down(3));
    assert_eq!(sidebar.selected, 1);
    assert_eq!(sidebar.scroll, 0);
    assert!(sidebar.scroll_selection_down(3));
    assert!(sidebar.scroll_selection_down(3));
    assert_eq!(sidebar.selected, 3);
    assert_eq!(sidebar.scroll, 1);
    assert!(sidebar.scroll_selection_up(3));
    assert_eq!(sidebar.selected, 2);
    assert_eq!(sidebar.scroll, 1);

    sidebar.selected = 0;
    sidebar.scroll = 0;
    assert!(!sidebar.scroll_selection_up(3));
    assert_eq!(sidebar.selected, 0);
    assert_eq!(sidebar.scroll, 0);
}

#[test]
fn shared_file_sidebar_mouse_row_selects_visible_entry() {
    let mut sidebar = FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..4)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 1,
    };

    assert_eq!(sidebar.selected_entry_for_mouse_row(0), None);
    assert_eq!(
        sidebar
            .selected_entry_for_mouse_row(2)
            .expect("visible entry")
            .label,
        "file-2.txt"
    );
    assert_eq!(sidebar.selected, 2);
    assert_eq!(sidebar.selected_entry_for_mouse_row(5), None);
    assert_eq!(sidebar.selected, 2);
}

#[test]
fn gui_workspace_opens_two_documents_as_focused_tiles() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);

    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(workspace.focused, GuiTileId(0));
    assert_eq!(
        workspace.active_tile().document.path,
        PathBuf::from("first.txt")
    );

    let second_id = workspace.open_tile(second);

    assert_eq!(second_id, GuiTileId(1));
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(workspace.active, second_id);
    assert_eq!(workspace.focused, second_id);
    assert_eq!(
        workspace.focused_tile().document.path,
        PathBuf::from("second.txt")
    );

    assert!(workspace.focus_tile(GuiTileId(0)));
    assert_eq!(
        workspace.active_tile().document.path,
        PathBuf::from("first.txt")
    );
    assert!(!workspace.focus_tile(GuiTileId(99)));
    assert_eq!(workspace.active, GuiTileId(0));
}

#[test]
fn gui_workspace_blocks_invalid_open_without_mutation() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);

    let result = workspace.open_validated_tile(Err(OpenError::Directory {
        path: PathBuf::from("dir"),
    }));

    assert!(matches!(
        result,
        Err(GuiTileOpenError::Invalid {
            source: OpenError::Directory { .. }
        })
    ));
    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(workspace.focused, GuiTileId(0));
}

#[test]
fn gui_workspace_dirty_close_requires_confirmation() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    second
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty second tile");
    let mut workspace = GuiWorkspace::from_document(first);
    let second_id = workspace.open_tile(second);

    assert_eq!(
        workspace.close_tile(second_id, false),
        GuiCloseTileResult::Dirty { tile_id: second_id }
    );
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(
        workspace.close_tile(second_id, true),
        GuiCloseTileResult::Closed {
            tile_id: second_id,
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(
        workspace.close_tile(GuiTileId(0), true),
        GuiCloseTileResult::OnlyTile
    );
}

#[test]
fn gui_workspace_tracks_minimize_and_layout_intents() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);
    let second_id = workspace.open_tile(second);

    assert!(workspace.set_tile_minimized(second_id, true));
    assert!(workspace.tile(second_id).expect("tile").minimized);
    assert!(workspace.set_tile_minimized(second_id, false));
    assert_eq!(workspace.focused, second_id);
    assert!(!workspace.set_tile_minimized(GuiTileId(99), true));

    assert!(workspace.request_split(second_id, GuiSplitDirection::Vertical));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Split {
            tile_id: second_id,
            direction: GuiSplitDirection::Vertical,
        })
    );
    assert!(workspace.request_move(second_id, GuiTileMoveDirection::Left));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Move {
            tile_id: second_id,
            direction: GuiTileMoveDirection::Left,
        })
    );
    assert!(workspace.request_resize(second_id, GuiTileResizeDirection::Wider));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Resize {
            tile_id: second_id,
            direction: GuiTileResizeDirection::Wider,
        })
    );
    assert!(!workspace.request_split(GuiTileId(99), GuiSplitDirection::Horizontal));
    workspace.clear_layout_intent();
    assert_eq!(workspace.pending_layout_intent, None);
}

#[test]
fn gui_workspace_reports_save_status_from_buffer_and_failures() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);
    let tile_id = workspace.active;

    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    workspace
        .active_tile_mut()
        .document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty tile");
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Modified
    );

    assert!(workspace.mark_tile_save_failed(tile_id, "permission denied"));
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::SaveFailed {
            message: String::from("permission denied"),
        }
    );
    workspace.active_tile_mut().document.buffer.mark_clean();
    assert!(workspace.clear_tile_save_error(tile_id));
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    assert!(!workspace.mark_tile_save_failed(GuiTileId(99), "missing"));
}

#[test]
fn gui_file_browser_lists_and_navigates_without_iced_types() {
    let temp = TempArea::new("gui-browser-nav");
    fs::create_dir(temp.path("z-dir")).expect("create z dir");
    fs::create_dir(temp.path("a-dir")).expect("create a dir");
    fs::write(temp.path("z.txt"), "z\n").expect("write z file");
    fs::write(temp.path("a.txt"), "a\n").expect("write a file");
    fs::write(temp.path("a-dir").join("inside.txt"), "inside\n").expect("write nested file");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");

    let labels: Vec<_> = browser
        .sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();
    assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);

    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "a-dir/")
        .expect("a-dir entry");
    assert_eq!(
        browser.activate_selected().expect("activate directory"),
        GuiFileBrowserActivation::Navigated {
            current_dir: temp
                .path("a-dir")
                .canonicalize()
                .expect("canonicalize a-dir"),
        }
    );
    assert_eq!(
        browser.selected_entry().expect("parent entry").kind,
        FileSidebarEntryKind::Parent
    );
}

#[test]
fn gui_file_browser_file_activation_opens_new_tile_through_existing_adapter() {
    let temp = TempArea::new("gui-browser-open");
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let canonical_next_path = next_path.canonicalize().expect("canonicalize next file");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    let mut workspace = GuiWorkspace::from_document(first);

    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "next.txt")
        .expect("next file entry");

    let activation = browser.activate_selected().expect("activate file");
    assert_eq!(
        activation,
        GuiFileBrowserActivation::OpenTile {
            path: canonical_next_path.clone(),
        }
    );

    let GuiFileBrowserActivation::OpenTile { path } = activation else {
        panic!("expected open tile activation");
    };
    let tile_id = workspace
        .open_validated_tile(open_text_file(&path))
        .expect("open validated tile");
    assert_eq!(tile_id, GuiTileId(1));
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(workspace.active_tile().document.path, canonical_next_path);
    assert_eq!(
        workspace.active_tile().document.buffer.lines(),
        &["next".to_string()]
    );
}

#[test]
fn gui_file_browser_refresh_picks_up_external_files_and_preserves_selection() {
    let temp = TempArea::new("gui-browser-refresh");
    fs::write(temp.path("keep.txt"), "keep\n").expect("write keep");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "keep.txt")
        .expect("keep entry");

    fs::write(temp.path("added.txt"), "added\n").expect("write added");

    browser.refresh().expect("refresh browser");

    let labels = browser
        .sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"added.txt"));
    assert_eq!(
        browser.selected_entry().expect("selected").label,
        "keep.txt"
    );
}

#[test]
fn gui_file_browser_refresh_clamps_selection_when_selected_entry_disappears() {
    let temp = TempArea::new("gui-browser-refresh-clamp");
    let removed = temp.path("removed.txt");
    fs::write(&removed, "removed\n").expect("write removed");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "removed.txt")
        .expect("removed entry");

    fs::remove_file(removed).expect("remove selected file");

    browser.refresh().expect("refresh browser");

    assert!(browser.sidebar.selected < browser.sidebar.entries.len());
    assert!(!browser
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "removed.txt"));
}

#[test]
fn gui_file_browser_rejects_invalid_roots_and_empty_selections() {
    let temp = TempArea::new("gui-browser-invalid");
    let missing = temp.path("missing");
    assert!(matches!(
        GuiFileBrowser::load(missing),
        Err(FileSidebarError::ReadDir { .. })
    ));

    let mut browser = GuiFileBrowser {
        sidebar: FileSidebarState {
            current_dir: temp.root.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
        },
    };
    assert!(matches!(
        browser.activate_selected(),
        Err(GuiFileBrowserError::EmptySelection)
    ));
}

#[test]
fn gui_file_browser_mouse_row_activation_selects_visible_file() {
    let temp = TempArea::new("gui-browser-mouse");
    fs::write(temp.path("first.txt"), "first\n").expect("write first");
    fs::write(temp.path("second.txt"), "second\n").expect("write second");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.scroll = 1;

    assert_eq!(browser.activate_mouse_row(0).expect("row zero"), None);
    assert_eq!(
        browser.activate_mouse_row(2).expect("activate row"),
        Some(GuiFileBrowserActivation::OpenTile {
            path: temp
                .path("second.txt")
                .canonicalize()
                .expect("canonicalize second file"),
        })
    );
    assert_eq!(
        browser.selected_entry().expect("selected").label,
        "second.txt"
    );
}

#[test]
fn consecutive_typed_inserts_coalesce_as_one_undo_step() {
    let mut buffer = TextBuffer::from_text("");

    buffer.insert_char(0, 0, 'a').expect("insert first");
    buffer.insert_char(0, 1, 'b').expect("insert second");
    buffer.insert_char(0, 2, 'c').expect("insert third");

    assert_eq!(buffer.to_text(), "abc");
    assert_eq!(buffer.undo_history.len(), 1);

    assert!(buffer.undo_last_edit());
    assert_eq!(buffer.to_text(), "");
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
            .insert_char(1, 0, 'x')
            .expect("insert character");
    });

    assert_eq!(document.buffer.to_text(), "hello\nx");
    assert_eq!(document.buffer.undo_history.len(), 1);
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
        .map(|snapshot| snapshot.byte_size)
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

    assert!(undo_steps > 0);
    let max_entries_by_budget = (MAX_UNDO_BYTES / (initial_text.len() + 1)) + 3;
    assert!(undo_steps <= max_entries_by_budget);
    assert!(!buffer.to_text().is_empty());
    assert!(buffer.to_text().len() >= initial_text.len());
    assert!(buffer.to_text().len() < after_edit_len);
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
    assert_eq!(history[0].lines[0], "c");
    assert_eq!(history[1].lines[0], "d");
}
