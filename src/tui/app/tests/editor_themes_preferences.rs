use super::*;
use crate::tui::input::*;
#[cfg(feature = "syntax")]
use crate::tui::menu::*;
use crate::tui::render::*;
use crate::tui::theme::*;

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
#[cfg(feature = "syntax")]
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
#[cfg(feature = "syntax")]
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
#[cfg(not(feature = "syntax"))]
fn syntax_theme_shortcut_reports_unavailable_in_lean_build() {
    let mut document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();
    let original_theme = runtime.settings.syntax_theme_id;

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )
    ));
    assert_eq!(runtime.settings.syntax_theme_id, original_theme);
    assert_eq!(
        runtime.status,
        "Syntax highlighting unavailable in this build"
    );
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
    let sample = SyntaxColor {
        r: 120,
        g: 140,
        b: 230,
        a: 255,
    };

    assert_eq!(
        syntax_color_to_terminal(sample, EditorThemeId::Nocturne),
        Color::Rgb {
            r: 132,
            g: 172,
            b: 255,
        }
    );
    assert_eq!(
        syntax_color_to_terminal(sample, EditorThemeId::Terror),
        Color::Rgb {
            r: 136,
            g: 172,
            b: 255,
        }
    );
    assert_ne!(
        syntax_color_to_terminal(sample, EditorThemeId::Nocturne),
        syntax_color_to_terminal(sample, EditorThemeId::Paper)
    );
}
