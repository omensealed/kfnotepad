use super::*;

#[test]
fn render_marks_dirty_buffer_and_controls() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 5, '!').expect("edit buffer");
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 6 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 20,
            status: "Ctrl-S save | Ctrl-Q quit",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("kfnotepad"));
    assert!(output.contains("note.txt"));
    assert!(output.contains("modified"));
    assert!(output.contains(" 1"));
    assert!(output.contains("hello!"));
    assert!(output.contains("Ln 1, Col 7"));
    assert!(output.contains("wrap:off"));
    assert!(output.contains("modified"));
    assert!(output.contains("F10 Menu/Help"));
}

#[test]
fn render_workspace_manager_overlay_shows_actions_and_projects() {
    let manager = WorkspaceManagerState {
        entries: vec![
            WorkspaceManagerEntry {
                name: String::from("Alpha"),
                files: 2,
            },
            WorkspaceManagerEntry {
                name: String::from("Beta"),
                files: 1,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_workspace_manager_overlay(
        &mut output,
        &manager,
        12,
        0,
        80,
        1,
        EditorTheme::for_id(EditorThemeId::Nocturne),
        false,
    )
    .expect("render manager");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Workspaces"));
    assert!(output.contains("Alpha"));
    assert!(output.contains("> Beta"));
    assert!(output.contains("D delete"));
    assert!(output.contains("+ Workspaces "));
    assert!(output.contains("|"));
}

#[test]
fn render_workspace_manager_overlay_without_colors_skips_color_escapes() {
    let manager = WorkspaceManagerState {
        entries: vec![
            WorkspaceManagerEntry {
                name: String::from("Alpha"),
                files: 2,
            },
            WorkspaceManagerEntry {
                name: String::from("Beta"),
                files: 1,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_workspace_manager_overlay(
        &mut output,
        &manager,
        12,
        0,
        80,
        1,
        EditorTheme::for_id(EditorThemeId::Nocturne),
        true,
    )
    .expect("render manager");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}
