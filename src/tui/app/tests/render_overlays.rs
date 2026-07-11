use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;
use crate::tui::sidebar::*;

#[test]
fn render_shows_keyboard_menu_dropdown() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "Menu: File",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::File,
                selected: 1,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" File "));
    assert!(output.contains(" Save"));
    assert!(output.contains("Ctrl-S"));
    assert!(output.contains(" Quit"));
    assert!(output.contains("Ctrl-Q"));
    assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
    assert!(output.contains("\u{1b}[2;12H"));
    assert!(output.ends_with("\u{1b}[3;14H"));
}

#[test]
fn render_help_menu_shows_compact_help_document_entry() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 12,
            status: "Menu: Help",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Help,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        120,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" Help "));
    assert!(output.contains("Open help document"));
    assert!(output.contains("Files and tabs"));
    assert!(output.contains("Ctrl-B / Ctrl-Enter / Ctrl-F4"));
    assert!(output.contains("Search and go"));
    assert!(output.contains("Ctrl-F / F3 / Shift-F3 / Ctrl-G"));
    assert!(output.contains("Editing"));
    assert!(output.contains("Ctrl-Z/Y / Ctrl-K / Insert"));
    assert!(output.contains("View and reader"));
    assert!(output.contains("Ctrl-L/T/R/W"));
    assert!(output.contains("Workspaces"));
    assert!(output.contains("F10 -> Workspace"));
    assert!(output.contains("Save and quit"));
    assert!(output.contains("Ctrl-S / Ctrl-Q"));
}

#[test]
fn render_anchors_edit_menu_under_header_label_without_color_spill_clear() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 4,
            status: "Menu: Edit",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Edit,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[2;18H"));
    assert!(output.contains("Find"));
    assert!(output.contains("Ctrl-F"));
    assert!(output.contains("Find next"));
    assert!(output.contains("F3"));
    assert!(output.contains("Delete previous word"));
    assert!(output.contains("Ctrl-Backspace"));
    assert!(output.contains("Delete next word"));
    assert!(output.contains("Ctrl-Delete"));
    assert!(output.contains("Delete to line end"));
    assert!(output.contains("Ctrl-K"));
    assert!(!output.contains("\u{1b}[46m\u{1b}[3;18H\u{1b}[2K"));
    assert!(output.ends_with("\u{1b}[2;20H"));
}

#[test]
fn render_tabs_menu_shows_tab_commands() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 6,
            status: "Menu: Tabs",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Tabs,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        100,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" Tabs "));
    assert!(output.contains("Previous tab"));
    assert!(output.contains("Ctrl-PageUp"));
    assert!(output.contains("Next tab"));
    assert!(output.contains("Ctrl-PageDown"));
    assert!(output.contains("Close tab"));
    assert!(output.contains("Ctrl-F4"));
    assert!(output.contains("Open sidebar file as tab"));
}

#[test]
fn help_line_uses_compact_bounded_controls() {
    let help = compose_help_line(102);

    assert_eq!(
        help.trim_end(),
        " F2 Command | F10 Menu/Help | Ctrl-S Save | Ctrl-B Files | Ctrl-Q Quit"
    );
    assert!(text_display_width(&help) <= 102);
}

#[test]
fn command_palette_filters_menu_commands_by_label_and_shortcut() {
    let wrap = command_palette_candidates("word wrap");
    assert_eq!(wrap.len(), 1);
    assert_eq!(wrap[0].command, MenuCommand::ToggleWrap);

    let save = command_palette_candidates("ctrl-s");
    assert!(save
        .iter()
        .any(|entry| entry.command == MenuCommand::Save && entry.group == MenuGroup::File));
    assert!(!save
        .iter()
        .any(|entry| entry.command == MenuCommand::HelpOnly));
}

#[test]
fn command_palette_executes_selected_workspace_command() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime::default();

    open_command_palette(&mut runtime);
    for key in [
        KeyCode::Char('w'),
        KeyCode::Char('o'),
        KeyCode::Char('r'),
        KeyCode::Char('d'),
        KeyCode::Char(' '),
        KeyCode::Char('w'),
        KeyCode::Char('r'),
        KeyCode::Char('a'),
        KeyCode::Char('p'),
        KeyCode::Enter,
    ] {
        assert!(!handle_command_palette_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.command_palette, None);
    assert!(runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap on");
}

#[test]
fn render_command_palette_overlay_shows_matching_commands() {
    let palette = CommandPaletteState {
        query: String::from("reader"),
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_command_palette_overlay(
        &mut output,
        &palette,
        10,
        0,
        90,
        1,
        EditorTheme::default(),
        false,
    )
    .expect("render command palette");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Command: reader"));
    assert!(output.contains("Reader mode"));
    assert!(output.contains("Reader slower"));
    assert!(output.contains("Reader faster"));
}

#[test]
fn render_command_palette_overlay_without_colors_skips_color_escapes() {
    let palette = CommandPaletteState {
        query: String::from("reader"),
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_command_palette_overlay(
        &mut output,
        &palette,
        10,
        0,
        90,
        1,
        EditorTheme::default(),
        true,
    )
    .expect("render command palette");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}

#[test]
fn render_file_sidebar_without_colors_skips_color_escapes() {
    let sidebar = FileSidebarState {
        current_dir: PathBuf::from("/tmp"),
        entries: vec![
            kfnotepad::FileSidebarEntry {
                label: String::from("../"),
                path: PathBuf::from("/"),
                kind: kfnotepad::FileSidebarEntryKind::Parent,
            },
            kfnotepad::FileSidebarEntry {
                label: String::from("notes.md"),
                path: PathBuf::from("/tmp/notes.md"),
                kind: kfnotepad::FileSidebarEntryKind::File,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    render_file_sidebar(&mut output, &sidebar, 8, EditorTheme::default(), true)
        .expect("render file sidebar");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}
