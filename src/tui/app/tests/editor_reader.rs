use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;

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
