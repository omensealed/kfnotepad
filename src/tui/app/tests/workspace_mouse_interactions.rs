use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;

#[test]
fn mouse_click_moves_cursor_in_editor_body() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(8, 2),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 1, column: 2 });
    assert!(!runtime.quit_confirmation_pending);
}

#[test]
fn mouse_click_respects_horizontal_offset() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(5, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 3,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 3 });
}

#[test]
fn mouse_click_respects_reserved_sidebar_width() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click((SIDEBAR_WIDTH + 8) as u16, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: SIDEBAR_WIDTH,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 2 });
}

#[test]
fn mouse_click_on_wrapped_visual_row_renders_and_edits_same_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdefghij\nsecond\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        settings: EditorSettings {
            wrap_lines: true,
            ..EditorSettings::default()
        },
        ..EditorRuntime::default()
    };
    let context = MouseContext {
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        gutter_width: 2,
        terminal_width: 10,
        sidebar_width: 0,
        body_top: 1,
    };

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(6, 2),
            context
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 8 });

    let frame = RenderFrame {
        theme: EditorTheme::for_id(runtime.settings.theme_id),
        gutter_width: context.gutter_width,
        terminal_width: context.terminal_width,
        origin_column: 0,
        body_top: context.body_top,
        no_color: false,
    };
    let view = EditorView {
        cursor,
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        status: "",
        settings: runtime.settings,
        menu: None,
        sidebar_width: 0,
        tab_strip: &[],
        search_highlight: None,
    };

    assert_eq!(cursor_screen_row(&document, view, frame), 2);
    assert_eq!(cursor_screen_column(&document, cursor, view, frame), 6);
    assert!(cursor_row_is_visible(&document, view, frame));

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.lines()[0], "abcdefghXij");
    assert_eq!(document.buffer.lines()[1], "second");
}

#[test]
fn mouse_click_opens_menu_group_and_runs_dropdown_item() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(18, 0),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Edit,
            selected: 0
        })
    );

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(19, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(runtime.menu, None);
    assert!(runtime.search_active);
    assert_eq!(runtime.status, "Search: ");
}

#[test]
fn mouse_move_is_ignored_without_requesting_redraw() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            MouseEvent {
                kind: MouseEventKind::Moved,
                column: 10,
                row: 1,
                modifiers: KeyModifiers::NONE,
            },
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Ignored
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}
