use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, MouseButton, MouseEvent, MouseEventKind, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, supports_keyboard_enhancement, Clear, ClearType,
    EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue};
#[cfg(test)]
use kfnotepad::parse_editor_settings_config;
#[cfg(test)]
use kfnotepad::GuiFontFamily;
use kfnotepad::{
    delete_gui_workspace_project, delete_next_word as shared_delete_next_word,
    delete_previous_word as shared_delete_previous_word,
    delete_to_line_end as shared_delete_to_line_end, editor_config_path,
    go_to_document_end as shared_go_to_document_end,
    go_to_document_start as shared_go_to_document_start, go_to_line as shared_go_to_line,
    gui_workspace_project_path, gui_workspace_projects_dir, help_text, list_gui_workspace_projects,
    list_managed_notes, load_editor_settings, managed_notes_dir, move_document_cursor,
    open_or_create_managed_note, open_text_file, page_down as shared_page_down,
    page_up as shared_page_up, parse_args, parse_gui_workspace_project, redo_document_edit,
    repeat_search_next_with_mode, repeat_search_previous_with_mode, save_editor_settings,
    save_gui_workspace_project, save_text_document, summarize_path, summarize_text,
    tui_help_document_text, undo_document_edit, CloseActiveTabResult, Command, Cursor, CursorMove,
    EditResult, EditorSettings, EditorTab, EditorTabDocument, EditorTabState, EditorThemeId,
    EditorWorkspace, FileSidebarEntry, FileSidebarEntryKind, FileSidebarState, GoToLineResult,
    GuiWorkspaceProject, GuiWorkspaceProjectDeleteResult, SearchMode, SearchRepeatResult,
    SyntaxHighlighter, TabStripItem, TextDocument, UndoRedoResult,
    DEFAULT_GUI_READER_LINES_PER_MINUTE, MAX_GUI_READER_LINES_PER_MINUTE,
    MIN_GUI_READER_LINES_PER_MINUTE, VERSION,
};
use syntect::highlighting::Style as SyntectStyle;
use unicode_width::UnicodeWidthChar;

const TAB_WIDTH: usize = 4;
const SIDEBAR_WIDTH: usize = 22;
const EDITOR_BODY_PADDING: usize = 1;
const MOUSE_WHEEL_ROWS: usize = 3;
const TUI_READER_TICK_MS: u64 = 250;
const TUI_HELP_DOCUMENT_PATH: &str = "kfnotepad-help.md";
const TUI_CURRENT_WORKSPACE_NAME: &str = "current workspace";
const TUI_WORKSPACE_DIR_NAME: &str = "tui";
const MENU_GROUPS: [MenuGroup; 7] = [
    MenuGroup::File,
    MenuGroup::Edit,
    MenuGroup::View,
    MenuGroup::Go,
    MenuGroup::Tabs,
    MenuGroup::Workspace,
    MenuGroup::Help,
];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_args(&args) {
        Ok(Command::Help) => {
            print!("{}", help_text());
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("kfnotepad {VERSION}");
            ExitCode::SUCCESS
        }
        Ok(Command::LaunchEmpty) => run_empty_command(),
        Ok(Command::InspectFile(path)) => run_file_command(&path),
        Ok(Command::ListManagedNotes) => run_list_managed_notes_command(),
        Ok(Command::OpenManagedNote(title)) => run_managed_note_command(&title),
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            eprintln!("Try `kfnotepad --help`.");
            ExitCode::from(2)
        }
    }
}

fn run_empty_command() -> ExitCode {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        if let Some((project_path, settings)) = current_tui_restore_project_request() {
            match load_tui_workspace_project(&project_path).and_then(|project| {
                workspace_from_project_documents(&project, env::current_dir().unwrap_or_default())
            }) {
                Ok(restored) => {
                    let status = restored.status_message();
                    return match run_editor_workspace(restored.workspace, Some(settings), status) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(error) => {
                            eprintln!("kfnotepad: terminal error: {error}");
                            ExitCode::from(1)
                        }
                    };
                }
                Err(error) => {
                    eprintln!("kfnotepad: workspace auto-restore failed: {error}");
                }
            }
        }
    }

    println!("kfnotepad executable baseline is ready.");
    println!("Run `kfnotepad --help` for supported commands.");
    ExitCode::SUCCESS
}

fn run_file_command(path: &str) -> ExitCode {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        match open_text_file(Path::new(path)) {
            Ok(mut document) => match run_editor(&mut document) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("kfnotepad: terminal error: {error}");
                    ExitCode::from(1)
                }
            },
            Err(error) => {
                eprintln!("kfnotepad: {error}");
                ExitCode::from(2)
            }
        }
    } else {
        match summarize_path(Path::new(path)) {
            Ok(summary) => {
                println!(
                    "Opened {path}: {} bytes, {} lines, trailing_newline={}",
                    summary.bytes, summary.lines, summary.trailing_newline
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("kfnotepad: {error}");
                ExitCode::from(2)
            }
        }
    }
}

fn run_list_managed_notes_command() -> ExitCode {
    let notes_dir = match current_managed_notes_dir() {
        Ok(notes_dir) => notes_dir,
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            return ExitCode::from(2);
        }
    };

    match list_managed_notes(&notes_dir) {
        Ok(notes) => {
            for note in notes {
                println!("{}", note.file_name);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_managed_note_command(title: &str) -> ExitCode {
    let notes_dir = match current_managed_notes_dir() {
        Ok(notes_dir) => notes_dir,
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            return ExitCode::from(2);
        }
    };

    match open_or_create_managed_note(&notes_dir, title) {
        Ok(mut document) if io::stdin().is_terminal() && io::stdout().is_terminal() => {
            match run_editor(&mut document) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("kfnotepad: terminal error: {error}");
                    ExitCode::from(1)
                }
            }
        }
        Ok(document) => {
            let summary = summarize_text(&document.buffer.to_text());
            let file_name = document
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("managed-note");
            println!(
                "Opened managed note {file_name}: {} bytes, {} lines, trailing_newline={}",
                summary.bytes, summary.lines, summary.trailing_newline
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            ExitCode::from(2)
        }
    }
}

fn current_managed_notes_dir() -> Result<PathBuf, kfnotepad::ManagedNotesError> {
    let xdg_data_home = env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let home = env::var_os("HOME").map(PathBuf::from);
    managed_notes_dir(xdg_data_home.as_deref(), home.as_deref())
}

fn run_editor(document: &mut TextDocument) -> io::Result<()> {
    let workspace = EditorWorkspace::from_document(document);
    run_editor_workspace(workspace, None, None)
}

fn run_editor_workspace(
    mut workspace: EditorWorkspace<'_>,
    loaded_settings: Option<EditorSettings>,
    initial_status: Option<String>,
) -> io::Result<()> {
    let highlighter = SyntaxHighlighter::default();
    let config_path = current_editor_config_path();
    let workspace_projects_dir = current_workspace_projects_dir();
    let mut runtime = EditorRuntime {
        settings: loaded_settings.unwrap_or_else(|| {
            config_path
                .as_deref()
                .map(load_editor_settings)
                .transpose()
                .unwrap_or_else(|error| {
                    eprintln!("kfnotepad: cannot load editor config: {error}");
                    None
                })
                .unwrap_or_default()
        }),
        config_path,
        workspace_projects_dir,
        ..EditorRuntime::default()
    };
    if let Some(status) = initial_status {
        runtime.status = status;
    }
    let mut terminal = TerminalSession::enter()?;
    let mut redraw = true;
    let mut current_visible_rows = visible_editor_rows(0);
    let mut current_terminal_width = terminal_width();
    let mut current_gutter_width = line_number_width(workspace.active_tab().document.as_ref());
    autosave_tui_current_workspace(&workspace, &mut runtime);

    loop {
        if redraw {
            let tab_items = workspace.tab_strip_items();
            let active_tab = workspace.active_tab_mut();
            current_terminal_width = terminal_width();
            let sidebar_width = runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH);
            let editor_width = current_terminal_width.saturating_sub(sidebar_width).max(1);
            let tab_extra_rows = tab_strip_height_for_width(&tab_items, editor_width)
                .saturating_sub(1)
                .into();
            current_visible_rows = visible_editor_rows(tab_extra_rows);
            runtime.page_rows = current_visible_rows;
            current_gutter_width = line_number_width(active_tab.document.as_ref());
            active_tab.state.viewport_start = if runtime.settings.gui_reader_mode_enabled {
                clamp_passive_viewport(
                    active_tab.document.as_ref(),
                    active_tab.state.viewport_start,
                    current_visible_rows,
                    runtime.settings,
                )
            } else {
                clamp_viewport(
                    active_tab.document.as_ref(),
                    active_tab.state.cursor,
                    active_tab.state.viewport_start,
                    current_visible_rows,
                    runtime.settings,
                    current_gutter_width,
                    editor_width,
                )
            };
            active_tab.state.horizontal_offset = if runtime.settings.wrap_lines {
                0
            } else {
                clamp_horizontal_viewport(
                    active_tab.document.as_ref(),
                    active_tab.state.cursor,
                    runtime.settings,
                    current_gutter_width,
                    current_terminal_width,
                    active_tab.state.horizontal_offset,
                )
            };
            render_editor(
                &mut terminal.stdout,
                active_tab.document.as_ref(),
                EditorView {
                    cursor: active_tab.state.cursor,
                    viewport_start: active_tab.state.viewport_start,
                    horizontal_offset: active_tab.state.horizontal_offset,
                    visible_rows: current_visible_rows,
                    status: &runtime.status,
                    settings: runtime.settings,
                    menu: runtime.menu,
                    sidebar_width: runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH),
                    tab_strip: &tab_items,
                    search_highlight: runtime.search_highlight(),
                },
                &highlighter,
            )?;
            if let Some(manager) = &runtime.workspace_manager {
                write_workspace_manager_overlay(
                    &mut terminal.stdout,
                    manager,
                    current_visible_rows,
                    runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH),
                    current_terminal_width,
                    tab_strip_height_for_width(&tab_items, editor_width),
                    EditorTheme::for_id(runtime.settings.theme_id),
                )?;
            }
            if let Some(palette) = &runtime.command_palette {
                write_command_palette_overlay(
                    &mut terminal.stdout,
                    palette,
                    current_visible_rows,
                    runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH),
                    current_terminal_width,
                    tab_strip_height_for_width(&tab_items, editor_width),
                    EditorTheme::for_id(runtime.settings.theme_id),
                )?;
            }
            if let Some(sidebar) = &runtime.sidebar {
                render_file_sidebar(
                    &mut terminal.stdout,
                    sidebar,
                    current_visible_rows,
                    EditorTheme::for_id(runtime.settings.theme_id),
                )?;
            }
            redraw = false;
        }

        let event = if runtime.settings.gui_reader_mode_enabled {
            if poll(Duration::from_millis(TUI_READER_TICK_MS))? {
                Some(read()?)
            } else {
                let active_tab = workspace.active_tab_mut();
                if apply_reader_tick(
                    active_tab.document.as_ref(),
                    &mut active_tab.state,
                    &mut runtime,
                    current_visible_rows,
                ) {
                    redraw = true;
                }
                None
            }
        } else {
            Some(read()?)
        };

        let Some(event) = event else {
            continue;
        };

        match event {
            Event::Key(event) => {
                if handle_workspace_key_event(&mut workspace, &mut runtime, event) {
                    redraw = true;
                    continue;
                }
                if runtime.menu.is_some() {
                    if handle_workspace_menu_key_event(&mut workspace, &mut runtime, event) {
                        break;
                    }
                    redraw = true;
                    continue;
                }
                if runtime.command_palette.is_some() {
                    if handle_command_palette_key_event(&mut workspace, &mut runtime, event) {
                        break;
                    }
                    redraw = true;
                    continue;
                }
                if runtime.workspace_prompt.is_some() {
                    handle_workspace_prompt_key_event(&mut workspace, &mut runtime, event);
                    redraw = true;
                    continue;
                }
                if runtime.workspace_manager.is_some() {
                    handle_workspace_manager_key_event(&mut workspace, &mut runtime, event);
                    redraw = true;
                    continue;
                }
                if runtime.sidebar.is_some() {
                    handle_workspace_sidebar_key_event(&mut workspace, &mut runtime, event);
                    redraw = true;
                    continue;
                }
                let active_tab = workspace.active_tab_mut();
                if handle_key_event(
                    active_tab.document.as_mut(),
                    &mut active_tab.state.cursor,
                    &mut runtime,
                    event,
                ) {
                    break;
                }
                redraw = true;
            }
            Event::Mouse(event) => {
                let sidebar_width = runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH);
                let editor_width = current_terminal_width.saturating_sub(sidebar_width).max(1);
                let body_top =
                    tab_strip_height_for_width(&workspace.tab_strip_items(), editor_width);
                let viewport_start = workspace.active_tab().state.viewport_start;
                let horizontal_offset = workspace.active_tab().state.horizontal_offset;
                match handle_workspace_mouse_event(
                    &mut workspace,
                    &mut runtime,
                    event,
                    MouseContext {
                        viewport_start,
                        horizontal_offset,
                        visible_rows: current_visible_rows,
                        gutter_width: current_gutter_width,
                        terminal_width: current_terminal_width,
                        sidebar_width,
                        body_top,
                    },
                ) {
                    InputResult::Quit => break,
                    InputResult::Handled => redraw = true,
                    InputResult::Ignored => {}
                }
            }
            Event::Resize(_, _) => redraw = true,
            _ => {}
        }
    }

    Ok(())
}

fn render_file_sidebar(
    writer: &mut impl Write,
    sidebar: &FileSidebarState,
    visible_rows: usize,
    theme: EditorTheme,
) -> io::Result<()> {
    let width = SIDEBAR_WIDTH;
    let title = fit_text_end(
        &format!(" Files: {} ", sidebar.current_dir.display()),
        width,
    );
    let mut remaining = width;
    queue!(
        writer,
        MoveTo(0, 0),
        SetForegroundColor(theme.status_fg),
        SetBackgroundColor(theme.status_bg),
        SetAttribute(Attribute::Bold),
    )?;
    print_truncated(writer, &title, &mut remaining)?;

    for row in 0..visible_rows {
        let entry_index = sidebar.scroll + row;
        let screen_row = (row + 1) as u16;
        let mut remaining = width;
        queue!(writer, MoveTo(0, screen_row),)?;
        if entry_index == sidebar.selected {
            queue!(
                writer,
                SetForegroundColor(theme.status_fg),
                SetBackgroundColor(theme.status_bg),
                SetAttribute(Attribute::Bold),
            )?;
        } else {
            queue!(
                writer,
                SetForegroundColor(theme.header_fg),
                SetBackgroundColor(theme.header_bg),
                SetAttribute(Attribute::Reset),
            )?;
        }
        let label = sidebar
            .entries
            .get(entry_index)
            .map(|entry| entry.label.as_str())
            .unwrap_or("");
        print_truncated(writer, &format!(" {label}"), &mut remaining)?;
    }

    let selected_row = sidebar.selected.saturating_sub(sidebar.scroll) + 1;
    queue!(
        writer,
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(2, selected_row.min(visible_rows) as u16)
    )?;
    writer.flush()
}

fn write_workspace_manager_overlay(
    writer: &mut impl Write,
    manager: &WorkspaceManagerState,
    visible_rows: usize,
    sidebar_width: usize,
    terminal_width: usize,
    body_top: u16,
    theme: EditorTheme,
) -> io::Result<()> {
    let origin_column = sidebar_width as u16;
    let main_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let width = if main_width < 24 {
        main_width
    } else {
        main_width.clamp(24, 64)
    };
    let x = ((main_width.saturating_sub(width)) / 2) as u16;
    let max_rows = visible_rows.saturating_sub(2).max(4);
    let entry_rows = manager.entries.len().min(max_rows.saturating_sub(3)).max(1);
    let height = entry_rows + 3;
    let y = body_top.saturating_add(1);
    let inner_width = width.saturating_sub(2);

    for row in 0..height {
        let mut remaining = inner_width;
        queue!(
            writer,
            MoveTo(
                origin_column.saturating_add(x),
                y.saturating_add(row as u16)
            ),
            SetForegroundColor(theme.status_fg),
            SetBackgroundColor(theme.status_bg),
            SetAttribute(Attribute::Reset),
        )?;
        if row == 0 {
            queue!(writer, SetAttribute(Attribute::Bold))?;
            write!(writer, "+")?;
            print_truncated(writer, " Workspaces ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", "-".repeat(remaining))?;
            }
            write!(writer, "+")?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
        } else if row == height - 1 {
            write!(writer, "+")?;
            if inner_width > 0 {
                write!(writer, "{}", "-".repeat(inner_width))?;
            }
            write!(writer, "+")?;
        } else if row == height - 2 {
            write!(writer, "|")?;
            print_truncated(
                writer,
                " Enter open | S save over | D delete | N new | Esc ",
                &mut remaining,
            )?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else if manager.entries.is_empty() {
            write!(writer, "|")?;
            print_truncated(writer, " No saved workspaces ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else {
            let index = manager.scroll + row - 1;
            write!(writer, "|")?;
            if let Some(entry) = manager.entries.get(index) {
                if index == manager.selected {
                    queue!(
                        writer,
                        SetForegroundColor(theme.search_fg),
                        SetBackgroundColor(theme.search_bg),
                        SetAttribute(Attribute::Bold),
                    )?;
                }
                let marker = if index == manager.selected { ">" } else { " " };
                let files = if entry.files == 1 { "file" } else { "files" };
                print_truncated(
                    writer,
                    &format!(" {marker} {}  {} {files} ", entry.name, entry.files),
                    &mut remaining,
                )?;
                queue!(
                    writer,
                    SetForegroundColor(theme.status_fg),
                    SetBackgroundColor(theme.status_bg),
                    SetAttribute(Attribute::Reset),
                )?;
            }
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        }
    }

    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)?;
    writer.flush()
}

fn write_command_palette_overlay(
    writer: &mut impl Write,
    palette: &CommandPaletteState,
    visible_rows: usize,
    sidebar_width: usize,
    terminal_width: usize,
    body_top: u16,
    theme: EditorTheme,
) -> io::Result<()> {
    let origin_column = sidebar_width as u16;
    let main_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let width = if main_width < 28 {
        main_width
    } else {
        main_width.clamp(28, 72)
    };
    let x = ((main_width.saturating_sub(width)) / 2) as u16;
    let max_rows = visible_rows.saturating_sub(2).max(5);
    let candidates = command_palette_candidates(&palette.query);
    let entry_rows = candidates.len().min(max_rows.saturating_sub(3)).max(1);
    let height = entry_rows + 3;
    let y = body_top.saturating_add(1);
    let inner_width = width.saturating_sub(2);
    let query_width = inner_width.saturating_sub(text_display_width(" Command: "));
    let query_display = fit_text_end(&palette.query, query_width.max(1));

    for row in 0..height {
        let mut remaining = inner_width;
        queue!(
            writer,
            MoveTo(
                origin_column.saturating_add(x),
                y.saturating_add(row as u16)
            ),
            SetForegroundColor(theme.status_fg),
            SetBackgroundColor(theme.status_bg),
            SetAttribute(Attribute::Reset),
        )?;
        if row == 0 {
            write!(writer, "+")?;
            queue!(writer, SetAttribute(Attribute::Bold))?;
            print_truncated(
                writer,
                &format!(" Command: {query_display}"),
                &mut remaining,
            )?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "+")?;
        } else if row == height - 1 {
            write!(writer, "+")?;
            if inner_width > 0 {
                write!(writer, "{}", "-".repeat(inner_width))?;
            }
            write!(writer, "+")?;
        } else if candidates.is_empty() {
            write!(writer, "|")?;
            print_truncated(writer, " No matching commands ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else {
            let index = palette.scroll + row - 1;
            write!(writer, "|")?;
            if let Some(entry) = candidates.get(index) {
                if index == palette.selected {
                    queue!(
                        writer,
                        SetForegroundColor(theme.search_fg),
                        SetBackgroundColor(theme.search_bg),
                        SetAttribute(Attribute::Bold),
                    )?;
                }
                let marker = if index == palette.selected { ">" } else { " " };
                let shortcut = entry.shortcut.unwrap_or("");
                let suffix = if shortcut.is_empty() {
                    String::new()
                } else {
                    format!("  {shortcut}")
                };
                print_truncated(
                    writer,
                    &format!(" {marker} {}: {}{suffix}", entry.group.label(), entry.label),
                    &mut remaining,
                )?;
                queue!(
                    writer,
                    SetForegroundColor(theme.status_fg),
                    SetBackgroundColor(theme.status_bg),
                    SetAttribute(Attribute::Reset),
                )?;
            }
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        }
    }

    let cursor_offset =
        text_display_width("+ Command: ").saturating_add(text_display_width(&query_display));
    queue!(
        writer,
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(
            origin_column
                .saturating_add(x)
                .saturating_add(cursor_offset.min(width.saturating_sub(1)) as u16),
            y
        ),
        Show
    )?;
    writer.flush()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputResult {
    Ignored,
    Handled,
    Quit,
}

#[derive(Clone, Copy)]
struct MouseContext {
    viewport_start: usize,
    horizontal_offset: usize,
    visible_rows: usize,
    gutter_width: usize,
    terminal_width: usize,
    sidebar_width: usize,
    body_top: u16,
}

fn handle_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('q') {
        return request_quit(document, runtime);
    }

    if runtime.sidebar.is_some() {
        return handle_sidebar_key_event(document, cursor, runtime, event);
    }

    if runtime.menu.is_some() {
        return handle_menu_key_event(document, cursor, runtime, event);
    }

    if runtime.goto_line_active {
        handle_goto_line_key_event(document, cursor, runtime, event);
        return false;
    }

    if runtime.search_active {
        handle_search_key_event(document, cursor, runtime, event);
        return false;
    }

    match (event.modifiers, event.code) {
        (_, KeyCode::F(10)) => open_menu(runtime),
        (KeyModifiers::SHIFT, KeyCode::F(3)) => repeat_search_previous(document, cursor, runtime),
        (_, KeyCode::F(3)) => repeat_search(document, cursor, runtime),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => save_document(document, runtime),
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            toggle_file_sidebar(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
            undo_document(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
            redo_document(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
            start_search(runtime);
        }
        (modifiers, KeyCode::Char('f') | KeyCode::Char('F'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            toggle_search_case(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            start_goto_line(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            toggle_line_numbers(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            cycle_theme(runtime);
        }
        (modifiers, KeyCode::Char('t') | KeyCode::Char('T'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            cycle_syntax_theme(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            toggle_wrap(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            toggle_reader_mode(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
            delete_to_line_end(document, cursor, runtime);
        }
        (_, KeyCode::Insert) => {
            toggle_overwrite_mode(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::WordLeft);
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::WordRight);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
            runtime.quit_confirmation_pending = false;
            cursor.column = 0;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(columns) = document.buffer.line_char_count(cursor.row) {
                cursor.column = columns;
            }
        }
        (_, KeyCode::Left) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Left);
        }
        (_, KeyCode::Right) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Right);
        }
        (_, KeyCode::Up) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Up);
        }
        (_, KeyCode::Down) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Down);
        }
        (_, KeyCode::PageUp) => {
            page_up(document, cursor, runtime);
        }
        (_, KeyCode::PageDown) => {
            page_down(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Home) => {
            go_to_document_start(cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::End) => {
            go_to_document_end(document, cursor, runtime);
        }
        (_, KeyCode::Home) => {
            runtime.quit_confirmation_pending = false;
            cursor.column = 0;
        }
        (_, KeyCode::End) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(columns) = document.buffer.line_char_count(cursor.row) {
                cursor.column = columns;
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Backspace) => {
            delete_previous_word(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Delete) => {
            delete_next_word(document, cursor, runtime);
        }
        (_, KeyCode::Backspace) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(moved) = document.buffer.delete_before_cursor(*cursor) {
                *cursor = moved;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::Delete) => {
            runtime.quit_confirmation_pending = false;
            if document
                .buffer
                .delete_char(cursor.row, cursor.column)
                .is_ok()
            {
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::Enter) => {
            runtime.quit_confirmation_pending = false;
            if document
                .buffer
                .insert_newline(cursor.row, cursor.column)
                .is_ok()
            {
                cursor.row += 1;
                cursor.column = 0;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::BackTab) | (KeyModifiers::SHIFT, KeyCode::Tab) => {
            unindent_at_cursor(document, cursor, runtime);
        }
        (_, KeyCode::Tab) => {
            indent_at_cursor(document, cursor, runtime);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value)) => {
            insert_typed_character(document, cursor, runtime, value);
        }
        _ => {}
    }
    false
}

fn handle_workspace_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    if runtime.sidebar.is_some()
        && event.modifiers.contains(KeyModifiers::CONTROL)
        && event.code == KeyCode::Enter
    {
        open_selected_sidebar_entry_in_new_tab(workspace, runtime);
        return true;
    }

    if runtime.sidebar.is_some()
        || runtime.menu.is_some()
        || runtime.command_palette.is_some()
        || runtime.goto_line_active
        || runtime.search_active
        || runtime.workspace_prompt.is_some()
        || runtime.workspace_manager.is_some()
    {
        return false;
    }

    match (event.modifiers, event.code) {
        (_, KeyCode::F(2)) => {
            open_command_palette(runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            create_new_file_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::F(4)) => {
            close_active_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::PageUp) => {
            select_previous_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::PageDown) => {
            select_next_tab(workspace, runtime);
            true
        }
        _ => false,
    }
}

fn handle_workspace_menu_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc | KeyCode::F(10) => {
            runtime.menu = None;
            runtime.status = String::from("Menu closed");
        }
        KeyCode::BackTab => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Tab => {
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                select_previous_menu_group(runtime);
            } else {
                select_next_menu_group(runtime);
            }
        }
        KeyCode::Left => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Right => {
            select_next_menu_group(runtime);
        }
        KeyCode::Home => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = 0;
            }
        }
        KeyCode::End => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = menu.group.items().len().saturating_sub(1);
            }
        }
        KeyCode::Up => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len();
                menu.selected = if menu.selected == 0 {
                    item_count.saturating_sub(1)
                } else {
                    menu.selected.saturating_sub(1)
                };
            }
        }
        KeyCode::Down => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len().max(1);
                menu.selected = (menu.selected + 1) % item_count;
            }
        }
        KeyCode::Enter => {
            let command = runtime.menu.and_then(|menu| {
                menu.group
                    .items()
                    .get(menu.selected)
                    .map(|item| item.command)
            });
            runtime.menu = None;
            if let Some(command) = command {
                return run_workspace_menu_command(command, workspace, runtime);
            }
        }
        _ => {}
    }
    false
}

fn handle_command_palette_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc => {
            runtime.command_palette = None;
            runtime.status = String::from("Command palette closed");
        }
        KeyCode::Up => move_command_palette_selection(runtime, -1),
        KeyCode::Down => move_command_palette_selection(runtime, 1),
        KeyCode::PageUp => move_command_palette_selection(runtime, -5),
        KeyCode::PageDown => move_command_palette_selection(runtime, 5),
        KeyCode::Home => set_command_palette_selection(runtime, 0),
        KeyCode::End => {
            let last = runtime
                .command_palette
                .as_ref()
                .map(|palette| {
                    command_palette_candidates(&palette.query)
                        .len()
                        .saturating_sub(1)
                })
                .unwrap_or(0);
            set_command_palette_selection(runtime, last);
        }
        KeyCode::Backspace => {
            if let Some(palette) = runtime.command_palette.as_mut() {
                palette.query.pop();
            }
            normalize_command_palette_selection(runtime);
        }
        KeyCode::Enter => {
            let command = selected_command_palette_entry(runtime).map(|entry| entry.command);
            runtime.command_palette = None;
            if let Some(command) = command {
                return run_workspace_menu_command(command, workspace, runtime);
            }
            runtime.status = String::from("No matching command");
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            if let Some(palette) = runtime.command_palette.as_mut() {
                palette.query.push(value);
            }
            normalize_command_palette_selection(runtime);
        }
        _ => {}
    }
    false
}

fn handle_menu_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc | KeyCode::F(10) => {
            runtime.menu = None;
            runtime.status = String::from("Menu closed");
        }
        KeyCode::BackTab => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Tab => {
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                select_previous_menu_group(runtime);
            } else {
                select_next_menu_group(runtime);
            }
        }
        KeyCode::Left => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Right => {
            select_next_menu_group(runtime);
        }
        KeyCode::Home => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = 0;
            }
        }
        KeyCode::End => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = menu.group.items().len().saturating_sub(1);
            }
        }
        KeyCode::Up => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len();
                menu.selected = if menu.selected == 0 {
                    item_count.saturating_sub(1)
                } else {
                    menu.selected.saturating_sub(1)
                };
            }
        }
        KeyCode::Down => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len().max(1);
                menu.selected = (menu.selected + 1) % item_count;
            }
        }
        KeyCode::Enter => {
            let command = runtime.menu.and_then(|menu| {
                menu.group
                    .items()
                    .get(menu.selected)
                    .map(|item| item.command)
            });
            runtime.menu = None;
            if let Some(command) = command {
                return run_menu_command(command, document, cursor, runtime);
            }
        }
        _ => {}
    }
    false
}

fn handle_sidebar_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Up => select_previous_sidebar_entry(runtime),
        KeyCode::Down => select_next_sidebar_entry(runtime),
        KeyCode::Enter => activate_selected_sidebar_entry(document, cursor, runtime),
        KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        _ => {}
    }
    false
}

fn handle_workspace_sidebar_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    if runtime.sidebar_prompt.is_some() {
        handle_sidebar_prompt_key_event(workspace, runtime, event);
        return;
    }

    match event.code {
        KeyCode::Esc => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Up => select_previous_sidebar_entry(runtime),
        KeyCode::Down => select_next_sidebar_entry(runtime),
        KeyCode::Enter => {
            activate_selected_sidebar_entry_for_workspace(workspace, runtime);
        }
        KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            start_sidebar_create_file(runtime);
        }
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            start_sidebar_create_directory(runtime);
        }
        KeyCode::Delete => {
            start_sidebar_delete(runtime);
        }
        _ => {}
    }
}

fn handle_sidebar_prompt_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = String::from("Files prompt cancelled");
        }
        KeyCode::Backspace => {
            runtime.sidebar_query.pop();
            refresh_sidebar_prompt_status(runtime);
        }
        KeyCode::Enter => {
            apply_sidebar_prompt(workspace, runtime);
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            runtime.sidebar_query.push(value);
            refresh_sidebar_prompt_status(runtime);
        }
        _ => {}
    }
}

#[cfg(test)]
fn handle_mouse_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> InputResult {
    let mut workspace = EditorWorkspace::from_document(document);
    workspace.active_tab_mut().state.cursor = *cursor;
    let result = handle_workspace_mouse_event(&mut workspace, runtime, event, context);
    *cursor = workspace.active_tab().state.cursor;
    result
}

fn handle_workspace_mouse_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> InputResult {
    if runtime.sidebar.is_some() && (event.column as usize) < context.sidebar_width {
        return match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                activate_sidebar_entry_at_mouse_for_workspace(workspace, runtime, event.row);
                InputResult::Handled
            }
            MouseEventKind::ScrollUp => {
                if scroll_sidebar_up(runtime, context.visible_rows) {
                    InputResult::Handled
                } else {
                    InputResult::Ignored
                }
            }
            MouseEventKind::ScrollDown => {
                if scroll_sidebar_down(runtime, context.visible_rows) {
                    InputResult::Handled
                } else {
                    InputResult::Ignored
                }
            }
            _ => InputResult::Ignored,
        };
    }

    match event.kind {
        MouseEventKind::ScrollUp => {
            let active_tab = workspace.active_tab_mut();
            if scroll_editor_by_mouse(
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
                context,
                event.row,
                CursorMove::Up,
            ) {
                return InputResult::Handled;
            }
            return InputResult::Ignored;
        }
        MouseEventKind::ScrollDown => {
            let active_tab = workspace.active_tab_mut();
            if scroll_editor_by_mouse(
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
                context,
                event.row,
                CursorMove::Down,
            ) {
                return InputResult::Handled;
            }
            return InputResult::Ignored;
        }
        _ => {}
    }

    if event.kind != MouseEventKind::Down(MouseButton::Left) {
        return InputResult::Ignored;
    }

    let frame = RenderFrame {
        theme: EditorTheme::for_id(runtime.settings.theme_id),
        gutter_width: context.gutter_width,
        terminal_width: context
            .terminal_width
            .saturating_sub(context.sidebar_width)
            .max(1),
        origin_column: context.sidebar_width as u16,
        body_top: context.body_top,
    };

    if let Some(menu) = runtime.menu {
        if let Some(command) = menu_command_at_mouse(event.column, event.row, menu, frame) {
            runtime.menu = None;
            return if run_workspace_menu_command(command, workspace, runtime) {
                InputResult::Quit
            } else {
                InputResult::Handled
            };
        }
    }

    if event.row == 0 {
        if let Some(group) = menu_group_at_mouse(event.column, frame) {
            runtime.menu = Some(MenuState { group, selected: 0 });
            runtime.status = format!("Menu: {}", group.label());
            return InputResult::Handled;
        }
        return InputResult::Ignored;
    }

    let tab_items = workspace.tab_strip_items();
    if let Some(index) = tab_index_at_mouse(event.column, event.row, &tab_items, frame) {
        workspace.active = index;
        runtime.quit_confirmation_pending = false;
        runtime.close_tab_confirmation_pending = false;
        runtime.menu = None;
        stop_reader_mode(runtime, "Reader mode stopped for tab switch");
        runtime.status = active_tab_status(workspace);
        autosave_tui_current_workspace(workspace, runtime);
        return InputResult::Handled;
    }

    let active_tab = workspace.active_tab_mut();
    if let Some(clicked) = cursor_at_mouse(
        active_tab.document.as_ref(),
        event.column,
        event.row,
        runtime,
        context,
    ) {
        runtime.quit_confirmation_pending = false;
        runtime.menu = None;
        active_tab.state.cursor = clicked;
        return InputResult::Handled;
    }

    InputResult::Ignored
}

fn menu_group_at_mouse(column: u16, frame: RenderFrame) -> Option<MenuGroup> {
    if !show_menu_bar(frame) {
        return None;
    }

    let column = (column as usize).checked_sub(frame.origin_column as usize)?;
    let mut start = text_display_width(" kfnotepad ");
    for group in MENU_GROUPS {
        let end = start + text_display_width(&format!(" {} ", group.label()));
        if (start..end).contains(&column) {
            return Some(group);
        }
        start = end;
    }
    None
}

fn menu_command_at_mouse(
    column: u16,
    row: u16,
    menu: MenuState,
    frame: RenderFrame,
) -> Option<MenuCommand> {
    if row == 0 {
        return None;
    }

    let column_start = frame
        .origin_column
        .saturating_add(menu_dropdown_column(menu.group, frame));
    let available_width = frame.terminal_width.saturating_sub(column_start as usize);
    let width = menu
        .group
        .items()
        .iter()
        .map(menu_item_display_width)
        .max()
        .unwrap_or(4)
        + 4;
    let width = width.min(available_width);
    let item_index = row as usize - 1;

    if width == 0
        || column < column_start
        || column as usize >= column_start as usize + width
        || item_index >= menu.group.items().len()
    {
        return None;
    }

    menu.group.items().get(item_index).map(|item| item.command)
}

fn tab_index_at_mouse(
    column: u16,
    row: u16,
    tab_strip: &[TabStripItem],
    frame: RenderFrame,
) -> Option<usize> {
    if tab_strip.len() <= 1 || row == 0 || row >= frame.body_top {
        return None;
    }

    let column = (column as usize).checked_sub(frame.origin_column as usize)?;
    let target_row = row - 1;
    let mut current_row = 0u16;
    let mut start = 0usize;
    for (index, item) in tab_strip.iter().enumerate() {
        let width = text_display_width(&compose_tab_label(index, item));
        if width > frame.terminal_width.saturating_sub(start) && start > 0 {
            current_row += 1;
            start = 0;
        }
        if current_row > target_row {
            return None;
        }
        let end = start.saturating_add(width);
        if current_row == target_row && (start..end).contains(&column) {
            return Some(index);
        }
        start = end;
    }
    None
}

fn cursor_at_mouse(
    document: &TextDocument,
    column: u16,
    row: u16,
    runtime: &EditorRuntime,
    context: MouseContext,
) -> Option<Cursor> {
    let body_row = row.checked_sub(context.body_top)? as usize;
    if body_row >= context.visible_rows {
        return None;
    }

    let gutter_columns = if runtime.settings.show_line_numbers {
        context.gutter_width + 1
    } else {
        0
    };
    let body_column = (column as usize).saturating_sub(gutter_columns);
    let body_column = body_column.saturating_sub(context.sidebar_width);
    let body_column = body_column.saturating_sub(EDITOR_BODY_PADDING);

    if runtime.settings.wrap_lines {
        let (document_row, wrapped_row_offset) =
            wrapped_document_row_at_screen_row(document, runtime.settings, context, body_row)?;
        let line = document.buffer.lines().get(document_row)?;
        let body_width = visible_text_columns(
            runtime.settings,
            context.gutter_width,
            context.terminal_width,
        );
        let target_display_column = wrapped_row_offset
            .saturating_mul(body_width)
            .saturating_add(body_column);
        return Some(Cursor {
            row: document_row,
            column: char_column_for_display_column(line, target_display_column),
        });
    }

    let document_row = context.viewport_start + body_row;
    let line = document.buffer.lines().get(document_row)?;
    Some(Cursor {
        row: document_row,
        column: char_column_for_display_column(
            line,
            context.horizontal_offset.saturating_add(body_column),
        ),
    })
}

fn wrapped_document_row_at_screen_row(
    document: &TextDocument,
    settings: EditorSettings,
    context: MouseContext,
    body_row: usize,
) -> Option<(usize, usize)> {
    let body_width = visible_text_columns(settings, context.gutter_width, context.terminal_width);
    let mut screen_row = 0usize;

    for (document_row, line) in document
        .buffer
        .lines()
        .iter()
        .enumerate()
        .skip(context.viewport_start)
    {
        let chunk_count = wrapped_line_chunks(line, body_width).len().max(1);
        if body_row < screen_row + chunk_count {
            return Some((document_row, body_row - screen_row));
        }
        screen_row += chunk_count;
        if screen_row > context.visible_rows {
            break;
        }
    }

    None
}

fn select_previous_menu_group(runtime: &mut EditorRuntime) {
    if let Some(menu) = &mut runtime.menu {
        menu.group = menu.group.previous();
        menu.selected = 0;
        runtime.status = format!("Menu: {}", menu.group.label());
    }
}

fn select_next_menu_group(runtime: &mut EditorRuntime) {
    if let Some(menu) = &mut runtime.menu {
        menu.group = menu.group.next();
        menu.selected = 0;
        runtime.status = format!("Menu: {}", menu.group.label());
    }
}

fn run_menu_command(
    command: MenuCommand,
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) -> bool {
    match command {
        MenuCommand::NewFile => {
            runtime.status = String::from("New file unavailable in this context");
        }
        MenuCommand::Save => save_document(document, runtime),
        MenuCommand::Quit => return request_quit(document, runtime),
        MenuCommand::ToggleSidebar => toggle_file_sidebar(runtime),
        MenuCommand::Find => start_search(runtime),
        MenuCommand::ToggleSearchCase => toggle_search_case(runtime),
        MenuCommand::Undo => undo_document(document, cursor, runtime),
        MenuCommand::Redo => redo_document(document, cursor, runtime),
        MenuCommand::DeletePreviousWord => delete_previous_word(document, cursor, runtime),
        MenuCommand::DeleteNextWord => delete_next_word(document, cursor, runtime),
        MenuCommand::DeleteToLineEnd => delete_to_line_end(document, cursor, runtime),
        MenuCommand::FindNext => repeat_search(document, cursor, runtime),
        MenuCommand::FindPrevious => repeat_search_previous(document, cursor, runtime),
        MenuCommand::GoToLine => start_goto_line(runtime),
        MenuCommand::ToggleLineNumbers => toggle_line_numbers(runtime),
        MenuCommand::CycleTheme => cycle_theme(runtime),
        MenuCommand::CycleSyntaxTheme => cycle_syntax_theme(runtime),
        MenuCommand::ToggleReaderMode => toggle_reader_mode(runtime),
        MenuCommand::DecreaseReaderSpeed => adjust_reader_speed(runtime, -10),
        MenuCommand::IncreaseReaderSpeed => adjust_reader_speed(runtime, 10),
        MenuCommand::ToggleWrap => toggle_wrap(runtime),
        MenuCommand::PageUp => page_up(document, cursor, runtime),
        MenuCommand::PageDown => page_down(document, cursor, runtime),
        MenuCommand::DocumentStart => go_to_document_start(cursor, runtime),
        MenuCommand::DocumentEnd => go_to_document_end(document, cursor, runtime),
        MenuCommand::PreviousWord => go_to_previous_word(document, cursor, runtime),
        MenuCommand::NextWord => go_to_next_word(document, cursor, runtime),
        MenuCommand::PreviousTab | MenuCommand::NextTab | MenuCommand::CloseTab => {
            runtime.status = String::from("Tab command unavailable in this context");
        }
        MenuCommand::SaveCurrentWorkspace
        | MenuCommand::SaveNamedWorkspace
        | MenuCommand::ListWorkspaces
        | MenuCommand::OpenWorkspace
        | MenuCommand::DeleteWorkspace
        | MenuCommand::OpenCurrentWorkspace
        | MenuCommand::ToggleRestoreLastWorkspace
        | MenuCommand::OpenCommandPalette
        | MenuCommand::OpenHelp => {
            runtime.status = String::from("Workspace command unavailable in this context");
        }
        MenuCommand::HelpOnly => {
            runtime.status = String::from("Help: choose a menu item or use the shown shortcut");
        }
    }
    false
}

fn run_workspace_menu_command(
    command: MenuCommand,
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) -> bool {
    match command {
        MenuCommand::NewFile => create_new_file_tab(workspace, runtime),
        MenuCommand::PreviousTab => select_previous_tab(workspace, runtime),
        MenuCommand::NextTab => select_next_tab(workspace, runtime),
        MenuCommand::CloseTab => close_active_tab(workspace, runtime),
        MenuCommand::SaveCurrentWorkspace => save_workspace_project_named(
            workspace,
            runtime,
            TUI_CURRENT_WORKSPACE_NAME,
            "current workspace",
        ),
        MenuCommand::SaveNamedWorkspace => start_workspace_save_prompt(runtime),
        MenuCommand::ListWorkspaces => open_workspace_manager(runtime),
        MenuCommand::OpenWorkspace => start_workspace_open_prompt(runtime),
        MenuCommand::DeleteWorkspace => start_workspace_delete_prompt(runtime),
        MenuCommand::OpenCurrentWorkspace => {
            open_workspace_project_named(workspace, runtime, TUI_CURRENT_WORKSPACE_NAME)
        }
        MenuCommand::ToggleRestoreLastWorkspace => toggle_restore_last_workspace(runtime),
        MenuCommand::OpenHelp => open_tui_help_document(workspace, runtime),
        MenuCommand::OpenCommandPalette => open_command_palette(runtime),
        _ => {
            let active_tab = workspace.active_tab_mut();
            return run_menu_command(
                command,
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
            );
        }
    }
    false
}

fn open_tui_help_document(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    let help_path = PathBuf::from(TUI_HELP_DOCUMENT_PATH);
    if let Some(index) = workspace
        .tabs
        .iter()
        .position(|tab| tab.document.as_ref().path == help_path)
    {
        workspace.active = index;
        runtime.menu = None;
        runtime.search_active = false;
        runtime.goto_line_active = false;
        stop_reader_mode(runtime, "Reader mode stopped for help");
        runtime.status = String::from("Focused help");
        return;
    }

    let document = TextDocument {
        path: help_path,
        buffer: kfnotepad::TextBuffer::from_text(tui_help_document_text()),
    };
    workspace.push_owned_tab(document);
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    stop_reader_mode(runtime, "Reader mode stopped for help");
    runtime.status = String::from("Opened help");
}

fn create_new_file_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    let path = next_tui_untitled_path(workspace, runtime);
    let document = TextDocument {
        path: path.clone(),
        buffer: kfnotepad::TextBuffer::from_text(""),
    };
    workspace.push_owned_tab(document);
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    stop_reader_mode(runtime, "Reader mode stopped for new file");
    runtime.status = format!("New file tab: {}", display_file_name(&path));
    autosave_tui_current_workspace(workspace, runtime);
}

fn next_tui_untitled_path(workspace: &EditorWorkspace<'_>, runtime: &EditorRuntime) -> PathBuf {
    let directory = runtime
        .sidebar
        .as_ref()
        .map(|sidebar| sidebar.current_dir.clone())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    for index in 1.. {
        let file_name = if index == 1 {
            "untitled.txt".to_string()
        } else {
            format!("untitled-{index}.txt")
        };
        let candidate = directory.join(file_name);
        let already_open = workspace
            .tabs
            .iter()
            .any(|tab| tab.document.as_ref().path == candidate);
        if !already_open && !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("untitled candidate search is unbounded")
}

fn open_menu(runtime: &mut EditorRuntime) {
    runtime.command_palette = None;
    runtime.menu = Some(MenuState::default());
    runtime.status = String::from("Menu: File");
}

fn open_command_palette(runtime: &mut EditorRuntime) {
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.workspace_prompt = None;
    runtime.workspace_manager = None;
    runtime.sidebar_prompt = None;
    runtime.command_palette = Some(CommandPaletteState::default());
    runtime.status = String::from("Command palette");
}

fn command_palette_candidates(query: &str) -> Vec<CommandPaletteEntry> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .collect();
    let mut entries = Vec::new();

    for group in MENU_GROUPS {
        for item in group.items() {
            if item.command == MenuCommand::HelpOnly {
                continue;
            }
            let searchable = format!(
                "{} {} {}",
                group.label(),
                item.label,
                item.shortcut.unwrap_or("")
            )
            .to_ascii_lowercase();
            if tokens.iter().all(|token| searchable.contains(token)) {
                entries.push(CommandPaletteEntry {
                    group,
                    label: item.label,
                    shortcut: item.shortcut,
                    command: item.command,
                });
            }
        }
    }

    entries
}

fn selected_command_palette_entry(runtime: &EditorRuntime) -> Option<CommandPaletteEntry> {
    let palette = runtime.command_palette.as_ref()?;
    command_palette_candidates(&palette.query)
        .get(palette.selected)
        .copied()
}

fn move_command_palette_selection(runtime: &mut EditorRuntime, delta: isize) {
    let Some(palette) = runtime.command_palette.as_ref() else {
        return;
    };
    let len = command_palette_candidates(&palette.query).len();
    if len == 0 {
        set_command_palette_selection(runtime, 0);
        return;
    }
    let current = palette.selected.min(len.saturating_sub(1));
    let next = current
        .saturating_add_signed(delta)
        .min(len.saturating_sub(1));
    set_command_palette_selection(runtime, next);
}

fn set_command_palette_selection(runtime: &mut EditorRuntime, selected: usize) {
    let Some(palette) = runtime.command_palette.as_mut() else {
        return;
    };
    let len = command_palette_candidates(&palette.query).len();
    palette.selected = if len == 0 {
        0
    } else {
        selected.min(len.saturating_sub(1))
    };
    let visible_rows = 8usize;
    if palette.selected < palette.scroll {
        palette.scroll = palette.selected;
    } else if palette.selected >= palette.scroll.saturating_add(visible_rows) {
        palette.scroll = palette.selected.saturating_sub(visible_rows - 1);
    }
    runtime.status = palette_status(palette, len);
}

fn normalize_command_palette_selection(runtime: &mut EditorRuntime) {
    set_command_palette_selection(runtime, 0);
}

fn palette_status(palette: &CommandPaletteState, len: usize) -> String {
    if palette.query.is_empty() {
        format!("Command palette: {len} commands")
    } else {
        format!("Command palette: {} match(es)", len)
    }
}

fn request_quit(document: &TextDocument, runtime: &mut EditorRuntime) -> bool {
    if document.buffer.is_dirty() {
        if runtime.quit_confirmation_pending {
            return true;
        }
        runtime.quit_confirmation_pending = true;
        runtime.status = String::from("Unsaved changes. Press Ctrl-Q again to quit.");
        return false;
    }
    true
}

fn save_document(document: &mut TextDocument, runtime: &mut EditorRuntime) {
    match save_text_document(document) {
        Ok(()) => {
            runtime.quit_confirmation_pending = false;
            runtime.status = String::from("Saved");
        }
        Err(error) => runtime.status = format!("Save failed: {error}"),
    }
}

fn toggle_file_sidebar(runtime: &mut EditorRuntime) {
    if runtime.sidebar.is_some() {
        close_file_sidebar(runtime);
        runtime.status = String::from("Files closed");
        return;
    }

    let directory = runtime.last_sidebar_dir.clone().map(Ok).unwrap_or_else(|| {
        env::current_dir().map_err(|error| format!("cannot resolve current directory: {error}"))
    });

    match directory.and_then(|path| FileSidebarState::load(path).map_err(|error| error.to_string()))
    {
        Ok(sidebar) => {
            runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
            runtime.sidebar = Some(sidebar);
            runtime.status = String::from("Files");
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

fn close_file_sidebar(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = runtime.sidebar.take() {
        runtime.last_sidebar_dir = Some(sidebar.current_dir);
    }
    runtime.sidebar_prompt = None;
    runtime.sidebar_query.clear();
}

fn select_previous_sidebar_entry(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = &mut runtime.sidebar {
        sidebar.select_previous_wrapping(runtime.page_rows);
    }
}

fn select_next_sidebar_entry(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = &mut runtime.sidebar {
        sidebar.select_next_wrapping(runtime.page_rows);
    }
}

fn scroll_sidebar_up(runtime: &mut EditorRuntime, visible_rows: usize) -> bool {
    let Some(sidebar) = &mut runtime.sidebar else {
        return false;
    };
    sidebar.scroll_selection_up(visible_rows)
}

fn scroll_sidebar_down(runtime: &mut EditorRuntime, visible_rows: usize) -> bool {
    let Some(sidebar) = &mut runtime.sidebar else {
        return false;
    };
    sidebar.scroll_selection_down(visible_rows)
}

fn start_sidebar_create_file(runtime: &mut EditorRuntime) {
    runtime.sidebar_prompt = Some(SidebarPrompt::CreateFile);
    runtime.sidebar_query.clear();
    runtime.status = String::from("New file name: ");
}

fn start_sidebar_create_directory(runtime: &mut EditorRuntime) {
    runtime.sidebar_prompt = Some(SidebarPrompt::CreateDirectory);
    runtime.sidebar_query.clear();
    runtime.status = String::from("New directory name: ");
}

fn start_sidebar_delete(runtime: &mut EditorRuntime) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        runtime.status = String::from("No file selected");
        return;
    };

    if entry.kind == FileSidebarEntryKind::Parent {
        runtime.status = String::from("Cannot delete parent entry");
        return;
    }

    let recursive = entry.kind == FileSidebarEntryKind::Directory;
    runtime.sidebar_prompt = Some(SidebarPrompt::DeleteConfirm { entry, recursive });
    runtime.sidebar_query.clear();
    runtime.status = if recursive {
        String::from("Delete directory and all contents? type yes: ")
    } else {
        String::from("Delete file? type yes: ")
    };
}

fn refresh_sidebar_prompt_status(runtime: &mut EditorRuntime) {
    runtime.status = match runtime.sidebar_prompt.as_ref() {
        Some(SidebarPrompt::CreateFile) => format!("New file name: {}", runtime.sidebar_query),
        Some(SidebarPrompt::CreateDirectory) => {
            format!("New directory name: {}", runtime.sidebar_query)
        }
        Some(SidebarPrompt::DeleteConfirm { recursive, .. }) => {
            if *recursive {
                format!(
                    "Delete directory and all contents? type yes: {}",
                    runtime.sidebar_query
                )
            } else {
                format!("Delete file? type yes: {}", runtime.sidebar_query)
            }
        }
        None => runtime.status.clone(),
    };
}

fn apply_sidebar_prompt(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    let Some(prompt) = runtime.sidebar_prompt.clone() else {
        return;
    };

    match prompt {
        SidebarPrompt::CreateFile => create_sidebar_file(runtime),
        SidebarPrompt::CreateDirectory => create_sidebar_directory(runtime),
        SidebarPrompt::DeleteConfirm { entry, recursive } => {
            delete_sidebar_entry(workspace, runtime, &entry, recursive);
        }
    }
}

fn create_sidebar_file(runtime: &mut EditorRuntime) {
    let name = match validated_sidebar_child_name(&runtime.sidebar_query) {
        Ok(name) => name.to_string(),
        Err(error) => {
            runtime.status = error;
            return;
        }
    };
    let Some(parent) = sidebar_target_directory(runtime) else {
        runtime.status = String::from("Files unavailable");
        return;
    };
    let path = parent.join(&name);

    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
            }
            refresh_sidebar_after_path_in_dir(runtime, &parent, &path);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Created file {name}");
        }
        Err(error) => runtime.status = format!("Create file failed: {error}"),
    }
}

fn create_sidebar_directory(runtime: &mut EditorRuntime) {
    let name = match validated_sidebar_child_name(&runtime.sidebar_query) {
        Ok(name) => name.to_string(),
        Err(error) => {
            runtime.status = error;
            return;
        }
    };
    let Some(parent) = sidebar_target_directory(runtime) else {
        runtime.status = String::from("Files unavailable");
        return;
    };
    let path = parent.join(&name);

    match fs::create_dir(&path) {
        Ok(()) => {
            refresh_sidebar_after_path_in_dir(runtime, &parent, &path);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Created directory {name}/");
        }
        Err(error) => runtime.status = format!("Create directory failed: {error}"),
    }
}

fn delete_sidebar_entry(
    workspace: &EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    entry: &FileSidebarEntry,
    recursive: bool,
) {
    if runtime.sidebar_query.trim() != "yes" {
        runtime.status = String::from("Delete cancelled; type yes to confirm");
        return;
    }

    if open_dirty_tab_uses_path(workspace, &entry.path) {
        runtime.status = String::from("Cannot delete an open modified file");
        return;
    }

    let metadata = match fs::symlink_metadata(&entry.path) {
        Ok(metadata) => metadata,
        Err(error) => {
            runtime.status = format!("Delete failed: {error}");
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        runtime.status = String::from("Refusing to delete symlink");
        return;
    }

    let result = match entry.kind {
        FileSidebarEntryKind::File if metadata.is_file() => fs::remove_file(&entry.path),
        FileSidebarEntryKind::Directory if metadata.is_dir() && recursive => {
            fs::remove_dir_all(&entry.path)
        }
        FileSidebarEntryKind::Parent => {
            runtime.status = String::from("Cannot delete parent entry");
            return;
        }
        _ => {
            runtime.status = String::from("Delete target changed type");
            return;
        }
    };

    match result {
        Ok(()) => {
            let deleted_label = entry.label.clone();
            refresh_sidebar_after_delete(runtime);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Deleted {deleted_label}");
        }
        Err(error) => runtime.status = format!("Delete failed: {error}"),
    }
}

fn validated_sidebar_child_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(String::from("Name is empty"));
    }
    if name == "." || name == ".." {
        return Err(String::from("Name cannot be . or .."));
    }
    if name.starts_with('.') {
        return Err(String::from(
            "Hidden names are not created from the sidebar",
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(String::from("Name must be local, not a path"));
    }
    if name.chars().any(char::is_control) {
        return Err(String::from("Name contains a control character"));
    }
    Ok(name)
}

fn sidebar_target_directory(runtime: &EditorRuntime) -> Option<PathBuf> {
    let sidebar = runtime.sidebar.as_ref()?;
    let selected = sidebar.selected_entry();
    match selected.map(|entry| entry.kind) {
        Some(FileSidebarEntryKind::Directory) => selected.map(|entry| entry.path.clone()),
        Some(FileSidebarEntryKind::Parent | FileSidebarEntryKind::File) | None => {
            Some(sidebar.current_dir.clone())
        }
    }
}

fn refresh_sidebar_after_path_in_dir(runtime: &mut EditorRuntime, directory: &Path, path: &Path) {
    match FileSidebarState::load(directory.to_path_buf()) {
        Ok(mut refreshed) => {
            if let Some(index) = refreshed
                .entries
                .iter()
                .position(|entry| entry.path == path)
            {
                refreshed.selected = index;
                refreshed.keep_selection_visible(runtime.page_rows);
            }
            runtime.last_sidebar_dir = Some(refreshed.current_dir.clone());
            runtime.sidebar = Some(refreshed);
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

fn refresh_sidebar_after_delete(runtime: &mut EditorRuntime) {
    let Some(sidebar) = runtime.sidebar.as_ref() else {
        return;
    };
    let current_dir = sidebar.current_dir.clone();
    let old_selected = sidebar.selected;
    match FileSidebarState::load(current_dir) {
        Ok(mut refreshed) => {
            if !refreshed.entries.is_empty() {
                refreshed.selected = old_selected.min(refreshed.entries.len() - 1);
                refreshed.keep_selection_visible(runtime.page_rows);
            }
            runtime.last_sidebar_dir = Some(refreshed.current_dir.clone());
            runtime.sidebar = Some(refreshed);
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

fn open_dirty_tab_uses_path(workspace: &EditorWorkspace<'_>, path: &Path) -> bool {
    workspace.tabs.iter().any(|tab| {
        let document = tab.document.as_ref();
        document.path == path && document.buffer.is_dirty()
    })
}

fn scroll_editor_by_mouse(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    context: MouseContext,
    row: u16,
    direction: CursorMove,
) -> bool {
    if runtime.menu.is_some() || runtime.search_active || runtime.goto_line_active {
        return false;
    }
    if row == 0 || row as usize > context.visible_rows {
        return false;
    }

    let original = *cursor;
    for _ in 0..MOUSE_WHEEL_ROWS {
        move_cursor(document, cursor, direction);
    }
    if *cursor == original {
        return false;
    }
    runtime.quit_confirmation_pending = false;
    runtime.status = match direction {
        CursorMove::Up => String::from("Scroll up"),
        CursorMove::Down => String::from("Scroll down"),
        _ => runtime.status.clone(),
    };
    true
}

fn activate_selected_sidebar_entry(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        return;
    };
    activate_sidebar_entry(document, cursor, runtime, entry);
}

fn activate_selected_sidebar_entry_for_workspace(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        return;
    };
    activate_sidebar_entry_for_workspace(workspace, runtime, entry);
}

fn activate_sidebar_entry_at_mouse_for_workspace(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    row: u16,
) {
    let Some(sidebar) = runtime.sidebar.as_mut() else {
        return;
    };
    if row == 0 {
        close_file_sidebar(runtime);
        runtime.status = String::from("Files closed");
        return;
    }
    let Some(entry) = sidebar.selected_entry_for_mouse_row(row) else {
        return;
    };
    activate_sidebar_entry_for_workspace(workspace, runtime, entry);
}

fn activate_sidebar_entry(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    entry: FileSidebarEntry,
) {
    match entry.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
            match FileSidebarState::load(entry.path) {
                Ok(sidebar) => {
                    runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
                    runtime.sidebar = Some(sidebar);
                    runtime.status = String::from("Files");
                }
                Err(error) => runtime.status = format!("Files unavailable: {error}"),
            }
        }
        FileSidebarEntryKind::File => {
            if document.buffer.is_dirty() {
                runtime.status = String::from("Save before opening another file");
                return;
            }
            match open_text_file(&entry.path) {
                Ok(next_document) => {
                    *document = next_document;
                    *cursor = Cursor { row: 0, column: 0 };
                    close_file_sidebar(runtime);
                    runtime.search_active = false;
                    runtime.goto_line_active = false;
                    stop_reader_mode(runtime, "Reader mode stopped for file open");
                    runtime.status = format!("Opened {}", entry.label);
                }
                Err(error) => runtime.status = format!("Open failed: {error}"),
            }
        }
    }
}

fn activate_sidebar_entry_for_workspace(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    entry: FileSidebarEntry,
) {
    match entry.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
            match FileSidebarState::load(entry.path) {
                Ok(sidebar) => {
                    runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
                    runtime.sidebar = Some(sidebar);
                    runtime.status = String::from("Files");
                }
                Err(error) => runtime.status = format!("Files unavailable: {error}"),
            }
        }
        FileSidebarEntryKind::File => {
            focus_or_open_file_tab(workspace, runtime, &entry.path, &entry.label);
        }
    }
}

fn focus_or_open_file_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    path: &Path,
    label: &str,
) {
    if let Some(index) = workspace
        .tabs
        .iter()
        .position(|tab| tab.document.as_ref().path == path)
    {
        workspace.active = index;
        close_file_sidebar(runtime);
        runtime.search_active = false;
        runtime.goto_line_active = false;
        runtime.quit_confirmation_pending = false;
        runtime.close_tab_confirmation_pending = false;
        stop_reader_mode(runtime, "Reader mode stopped for tab focus");
        runtime.status = format!("Focused tab {label}");
        autosave_tui_current_workspace(workspace, runtime);
        return;
    }

    match open_text_file(path) {
        Ok(document) => {
            workspace.push_owned_tab(document);
            close_file_sidebar(runtime);
            runtime.search_active = false;
            runtime.goto_line_active = false;
            runtime.quit_confirmation_pending = false;
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for file open");
            runtime.status = format!("Opened tab {label}");
            autosave_tui_current_workspace(workspace, runtime);
        }
        Err(error) => runtime.status = format!("Open failed: {error}"),
    }
}

fn open_selected_sidebar_entry_in_new_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        return;
    };

    match entry.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
            match FileSidebarState::load(entry.path) {
                Ok(sidebar) => {
                    runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
                    runtime.sidebar = Some(sidebar);
                    runtime.status = String::from("Files");
                }
                Err(error) => runtime.status = format!("Files unavailable: {error}"),
            }
        }
        FileSidebarEntryKind::File => {
            focus_or_open_file_tab(workspace, runtime, &entry.path, &entry.label);
        }
    }
}

fn start_workspace_save_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::SaveNamed);
    runtime.workspace_query.clear();
    load_workspace_prompt_candidates(runtime);
    runtime.workspace_prompt_candidate_index = None;
    runtime.status = if runtime.workspace_prompt_candidates.is_empty() {
        String::from("Save workspace as: ")
    } else {
        String::from("Save workspace as:  (Up/Down picks existing)")
    };
}

fn start_workspace_open_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_open_confirmation_pending = false;
    runtime.workspace_pending_open = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::OpenNamed);
    load_workspace_prompt_candidates(runtime);
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.workspace_query.clear();
        runtime.workspace_prompt_candidate_index = None;
        runtime.workspace_prompt = None;
        runtime.status = String::from("No workspace projects saved");
    } else {
        runtime.workspace_prompt_candidate_index = Some(0);
        runtime.workspace_query = runtime.workspace_prompt_candidates[0].clone();
        refresh_workspace_prompt_status(runtime);
    }
}

fn start_workspace_delete_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_pending_delete = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::DeleteNamed);
    load_workspace_prompt_candidates(runtime);
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.workspace_query.clear();
        runtime.workspace_prompt_candidate_index = None;
        runtime.workspace_prompt = None;
        runtime.status = String::from("No workspace projects saved");
    } else {
        runtime.workspace_prompt_candidate_index = Some(0);
        runtime.workspace_query = runtime.workspace_prompt_candidates[0].clone();
        refresh_workspace_prompt_status(runtime);
    }
}

fn handle_workspace_prompt_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.status = String::from("Workspace prompt cancelled");
        }
        KeyCode::Up => {
            select_workspace_prompt_candidate(runtime, -1);
        }
        KeyCode::Down => {
            select_workspace_prompt_candidate(runtime, 1);
        }
        KeyCode::Backspace => {
            runtime.workspace_query.pop();
            runtime.workspace_prompt_candidate_index = None;
            refresh_workspace_prompt_status(runtime);
        }
        KeyCode::Enter => {
            apply_workspace_prompt(workspace, runtime);
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            runtime.workspace_query.push(value);
            runtime.workspace_prompt_candidate_index = None;
            refresh_workspace_prompt_status(runtime);
        }
        _ => {}
    }
}

fn handle_workspace_manager_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.workspace_manager = None;
            runtime.status = String::from("Workspace manager closed");
        }
        KeyCode::Up => move_workspace_manager_selection(runtime, -1),
        KeyCode::Down => move_workspace_manager_selection(runtime, 1),
        KeyCode::PageUp => move_workspace_manager_selection(runtime, -5),
        KeyCode::PageDown => move_workspace_manager_selection(runtime, 5),
        KeyCode::Home => set_workspace_manager_selection(runtime, 0),
        KeyCode::End => {
            let last = runtime
                .workspace_manager
                .as_ref()
                .map(|manager| manager.entries.len().saturating_sub(1))
                .unwrap_or(0);
            set_workspace_manager_selection(runtime, last);
        }
        KeyCode::Enter => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                open_workspace_project_named(workspace, runtime, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Delete | KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                prepare_delete_workspace_project(runtime, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                save_workspace_project_named(workspace, runtime, &name, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            runtime.workspace_manager = None;
            start_workspace_save_prompt(runtime);
        }
        _ => {}
    }
}

fn selected_workspace_manager_name(runtime: &EditorRuntime) -> Option<String> {
    runtime
        .workspace_manager
        .as_ref()
        .and_then(|manager| manager.entries.get(manager.selected))
        .map(|entry| entry.name.clone())
}

fn move_workspace_manager_selection(runtime: &mut EditorRuntime, delta: isize) {
    let Some(manager) = runtime.workspace_manager.as_ref() else {
        return;
    };
    if manager.entries.is_empty() {
        runtime.status = String::from("No workspace projects saved; press N to save a new project");
        return;
    }
    let len = manager.entries.len();
    let selected = manager.selected;
    let next = if delta.is_negative() {
        selected.saturating_sub(delta.unsigned_abs())
    } else {
        (selected + delta as usize).min(len.saturating_sub(1))
    };
    set_workspace_manager_selection(runtime, next);
}

fn set_workspace_manager_selection(runtime: &mut EditorRuntime, selected: usize) {
    let Some(manager) = runtime.workspace_manager.as_mut() else {
        return;
    };
    if manager.entries.is_empty() {
        manager.selected = 0;
        manager.scroll = 0;
        runtime.status = String::from("No workspace projects saved; press N to save a new project");
        return;
    }
    manager.selected = selected.min(manager.entries.len().saturating_sub(1));
    manager.scroll = manager.selected.saturating_sub(6);
    if let Some(entry) = manager.entries.get(manager.selected) {
        runtime.status = format!(
            "Workspace: {} (Enter open | S save over | D delete)",
            entry.name
        );
    }
}

fn refresh_workspace_prompt_status(runtime: &mut EditorRuntime) {
    runtime.status = match runtime.workspace_prompt.as_ref() {
        Some(WorkspacePrompt::SaveNamed) => {
            format!("Save workspace as: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::OpenNamed) => {
            format!("Open workspace: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::DeleteNamed) => {
            format!("Delete workspace: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::ConfirmOpen) => {
            format!(
                "Replace dirty workspace? type yes: {}",
                runtime.workspace_query
            )
        }
        Some(WorkspacePrompt::ConfirmDelete) => {
            format!(
                "Delete workspace project? type yes: {}",
                runtime.workspace_query
            )
        }
        None => runtime.status.clone(),
    };
}

fn load_workspace_prompt_candidates(runtime: &mut EditorRuntime) {
    runtime.workspace_prompt_candidates.clear();
    runtime.workspace_prompt_candidate_index = None;
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        return;
    };
    if let Ok(projects) = list_gui_workspace_projects(projects_dir) {
        runtime.workspace_prompt_candidates = projects
            .into_iter()
            .map(|entry| entry.project.name)
            .collect::<Vec<_>>();
    }
}

fn select_workspace_prompt_candidate(runtime: &mut EditorRuntime, delta: isize) {
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.status = String::from("No workspace projects saved");
        return;
    }

    let len = runtime.workspace_prompt_candidates.len();
    let current = runtime.workspace_prompt_candidate_index.unwrap_or(0);
    let next = if delta.is_negative() {
        current
            .checked_sub(delta.unsigned_abs())
            .unwrap_or(len.saturating_sub(1))
    } else {
        (current + delta as usize) % len
    };
    runtime.workspace_prompt_candidate_index = Some(next);
    runtime.workspace_query = runtime.workspace_prompt_candidates[next].clone();
    refresh_workspace_prompt_status(runtime);
}

fn apply_workspace_prompt(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    match runtime.workspace_prompt {
        Some(WorkspacePrompt::SaveNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            save_workspace_project_named(workspace, runtime, &name, &name);
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
        }
        Some(WorkspacePrompt::OpenNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            open_workspace_project_named(workspace, runtime, &name);
        }
        Some(WorkspacePrompt::DeleteNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            prepare_delete_workspace_project(runtime, &name);
        }
        Some(WorkspacePrompt::ConfirmOpen) => {
            if runtime.workspace_query.trim() != "yes" {
                runtime.status = String::from("Workspace open cancelled; type yes to confirm");
                return;
            }
            let Some(project) = runtime.workspace_pending_open.clone() else {
                runtime.status = String::from("No workspace pending");
                runtime.workspace_prompt = None;
                runtime.workspace_query.clear();
                return;
            };
            replace_workspace_from_project(workspace, runtime, &project);
        }
        Some(WorkspacePrompt::ConfirmDelete) => {
            confirm_delete_workspace_project(runtime);
        }
        None => {}
    }
}

fn save_workspace_project_named(
    workspace: &EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    project_name: &str,
    status_name: &str,
) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace save failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, project_name) else {
        runtime.status = String::from("Workspace save failed: invalid project name");
        return;
    };
    let Some(project) = current_tui_workspace_project(workspace, project_name) else {
        runtime.status = String::from("Workspace save failed: no files to save");
        return;
    };

    match save_gui_workspace_project(&path, &project) {
        Ok(()) => runtime.status = format!("Workspace saved: {status_name}"),
        Err(error) => runtime.status = format!("Workspace save failed: {error}"),
    }
}

fn prepare_delete_workspace_project(runtime: &mut EditorRuntime, name: &str) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace delete failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, name) else {
        runtime.status = String::from("Workspace delete failed: invalid project name");
        return;
    };
    if !path.exists() {
        runtime.status = format!("Workspace not found: {name}");
        return;
    }

    runtime.workspace_pending_delete = Some((name.to_string(), path));
    runtime.workspace_prompt = Some(WorkspacePrompt::ConfirmDelete);
    runtime.workspace_query.clear();
    runtime.workspace_prompt_candidates.clear();
    runtime.workspace_prompt_candidate_index = None;
    runtime.status = String::from("Delete workspace project? type yes: ");
}

fn confirm_delete_workspace_project(runtime: &mut EditorRuntime) {
    if runtime.workspace_query.trim() != "yes" {
        runtime.status = String::from("Workspace delete cancelled; type yes to confirm");
        return;
    }
    let Some((name, path)) = runtime.workspace_pending_delete.clone() else {
        runtime.status = String::from("No workspace pending delete");
        runtime.workspace_prompt = None;
        runtime.workspace_query.clear();
        return;
    };
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        runtime.status = String::from("Workspace delete failed: cannot resolve config directory");
        return;
    };

    match delete_gui_workspace_project(projects_dir, &path) {
        Ok(GuiWorkspaceProjectDeleteResult::Deleted) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.status = format!("Deleted workspace: {name}");
        }
        Ok(GuiWorkspaceProjectDeleteResult::Missing) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_delete = None;
            runtime.status = format!("Workspace already missing: {name}");
        }
        Err(error) => runtime.status = format!("Workspace delete failed: {error}"),
    }
}

fn autosave_tui_current_workspace(workspace: &EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    if !runtime.settings.gui_restore_last_workspace {
        return;
    }
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME) else {
        return;
    };
    let Some(project) = current_tui_workspace_project(workspace, TUI_CURRENT_WORKSPACE_NAME) else {
        return;
    };

    if let Err(error) = save_gui_workspace_project(&path, &project) {
        runtime.status = format!("{}; workspace autosave failed: {error}", runtime.status);
    }
}

fn current_tui_workspace_project(
    workspace: &EditorWorkspace<'_>,
    project_name: &str,
) -> Option<GuiWorkspaceProject> {
    let files = workspace
        .tabs
        .iter()
        .map(|tab| tab.document.as_ref().path.clone())
        .collect::<Vec<_>>();
    if files.is_empty() {
        return None;
    }
    Some(GuiWorkspaceProject {
        name: project_name.to_string(),
        files,
        active_ordinal: workspace.active.min(workspace.tabs.len().saturating_sub(1)),
        layout: None,
    })
}

fn open_workspace_manager(runtime: &mut EditorRuntime) {
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        runtime.status = String::from("Workspaces unavailable: cannot resolve config directory");
        return;
    };
    match list_gui_workspace_projects(projects_dir) {
        Ok(projects) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.workspace_manager = Some(WorkspaceManagerState {
                entries: projects
                    .into_iter()
                    .map(|entry| WorkspaceManagerEntry {
                        name: entry.project.name,
                        files: entry.project.files.len(),
                    })
                    .collect(),
                selected: 0,
                scroll: 0,
            });
            runtime.status = if runtime
                .workspace_manager
                .as_ref()
                .is_some_and(|manager| manager.entries.is_empty())
            {
                String::from("No workspace projects saved; press N to save a new project")
            } else {
                String::from("Workspace manager: Enter open | S save over | D delete | N new | Esc")
            };
        }
        Err(error) => runtime.status = format!("Workspace list failed: {error}"),
    }
}

fn toggle_restore_last_workspace(runtime: &mut EditorRuntime) {
    runtime.settings.gui_restore_last_workspace = !runtime.settings.gui_restore_last_workspace;
    runtime.status = if runtime.settings.gui_restore_last_workspace {
        String::from("Restore last workspace: on")
    } else {
        String::from("Restore last workspace: off")
    };
    persist_runtime_settings(runtime);
}

fn open_workspace_project_named(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    name: &str,
) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace open failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, name) else {
        runtime.status = String::from("Workspace open failed: invalid project name");
        return;
    };

    let project = match load_tui_workspace_project(&path) {
        Ok(project) => project,
        Err(error) => {
            runtime.status = format!("Workspace open failed: {error}");
            return;
        }
    };

    if workspace_has_dirty_tabs(workspace) && !runtime.workspace_open_confirmation_pending {
        runtime.workspace_pending_open = Some(project);
        runtime.workspace_prompt = Some(WorkspacePrompt::ConfirmOpen);
        runtime.workspace_query.clear();
        runtime.workspace_open_confirmation_pending = true;
        runtime.status = String::from("Replace dirty workspace? type yes: ");
        return;
    }

    replace_workspace_from_project(workspace, runtime, &project);
}

fn replace_workspace_from_project(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    project: &GuiWorkspaceProject,
) {
    match workspace_from_project_documents(project, env::current_dir().unwrap_or_default()) {
        Ok(restored) => {
            let status = restored
                .status_message()
                .unwrap_or_else(|| format!("Opened workspace: {}", project.name));
            *workspace = restored.workspace;
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.workspace_manager = None;
            close_file_sidebar(runtime);
            runtime.search_active = false;
            runtime.goto_line_active = false;
            runtime.quit_confirmation_pending = false;
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for workspace open");
            runtime.status = status;
            autosave_tui_current_workspace(workspace, runtime);
        }
        Err(error) => {
            runtime.status = format!("Workspace open failed: {error}");
        }
    }
}

fn workspace_has_dirty_tabs(workspace: &EditorWorkspace<'_>) -> bool {
    workspace
        .tabs
        .iter()
        .any(|tab| tab.document.as_ref().buffer.is_dirty())
}

fn load_tui_workspace_project(path: &Path) -> Result<GuiWorkspaceProject, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    parse_gui_workspace_project(&text)
        .ok_or_else(|| format!("invalid workspace project {}", path.display()))
}

struct RestoredTuiWorkspace {
    project_name: String,
    workspace: EditorWorkspace<'static>,
    skipped_files: Vec<String>,
    created_blank: bool,
}

impl RestoredTuiWorkspace {
    fn status_message(&self) -> Option<String> {
        if self.skipped_files.is_empty() {
            return Some(format!("Opened workspace: {}", self.project_name));
        }

        let first = self
            .skipped_files
            .first()
            .map(String::as_str)
            .unwrap_or("unknown path");
        let loaded = if self.created_blank {
            "opened blank tab".to_string()
        } else {
            format!("loaded {} file(s)", self.workspace.tabs.len())
        };
        Some(format!(
            "Opened workspace: {}; skipped {} missing/unavailable file(s), {loaded}; first: {first}",
            self.project_name,
            self.skipped_files.len()
        ))
    }
}

fn workspace_from_project_documents(
    project: &GuiWorkspaceProject,
    current_dir: PathBuf,
) -> Result<RestoredTuiWorkspace, String> {
    let mut tabs = Vec::new();
    let mut active = 0usize;
    let mut active_loaded = false;
    let mut skipped_files = Vec::new();

    for (ordinal, path) in project.files.iter().enumerate() {
        let document = match open_text_file(path) {
            Ok(document) => document,
            Err(error) => {
                skipped_files.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        if ordinal == project.active_ordinal {
            active = tabs.len();
            active_loaded = true;
        }
        tabs.push(EditorTab {
            document: EditorTabDocument::Owned(document),
            state: EditorTabState::default(),
        });
    }

    let created_blank = tabs.is_empty();
    if tabs.is_empty() {
        tabs.push(EditorTab {
            document: EditorTabDocument::Owned(TextDocument {
                path: current_dir.join("untitled.txt"),
                buffer: kfnotepad::TextBuffer::from_text(""),
            }),
            state: EditorTabState::default(),
        });
        active_loaded = true;
    }
    if !active_loaded {
        active = tabs.len() - 1;
    }
    active = active.min(tabs.len() - 1);
    Ok(RestoredTuiWorkspace {
        project_name: project.name.clone(),
        workspace: EditorWorkspace { tabs, active },
        skipped_files,
        created_blank,
    })
}

fn undo_document(document: &mut TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.status = match undo_document_edit(document, cursor) {
        UndoRedoResult::Applied => {
            stop_reader_mode_for_edit(runtime);
            String::from("Undone")
        }
        UndoRedoResult::NothingToApply => String::from("Nothing to undo"),
    };
}

fn redo_document(document: &mut TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.status = match redo_document_edit(document, cursor) {
        UndoRedoResult::Applied => {
            stop_reader_mode_for_edit(runtime);
            String::from("Redone")
        }
        UndoRedoResult::NothingToApply => String::from("Nothing to redo"),
    };
}

fn delete_previous_word(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_previous_word(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}

fn delete_next_word(document: &mut TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_next_word(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}

fn delete_to_line_end(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_to_line_end(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}

fn toggle_overwrite_mode(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.overwrite_mode = !runtime.overwrite_mode;
    runtime.status = if runtime.overwrite_mode {
        String::from("Overwrite on")
    } else {
        String::from("Insert mode")
    };
}

fn insert_typed_character(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    value: char,
) {
    runtime.quit_confirmation_pending = false;
    let result = if runtime.overwrite_mode {
        document
            .buffer
            .replace_char(cursor.row, cursor.column, value)
    } else {
        document
            .buffer
            .insert_char(cursor.row, cursor.column, value)
    };
    if result.is_ok() {
        cursor.column += 1;
        stop_reader_mode_for_edit(runtime);
        runtime.status = if runtime.overwrite_mode {
            String::from("Modified overwrite")
        } else {
            String::from("Modified")
        };
    }
}

fn start_search(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.goto_line_active = false;
    runtime.goto_line_query.clear();
    runtime.search_active = true;
    runtime.search_query.clear();
    runtime.search_history_index = None;
    runtime.status = String::from("Search: ");
}

fn repeat_search(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    let query = runtime.last_search_query.clone();
    runtime.status = search_repeat_status(repeat_search_next_with_mode(
        document,
        cursor,
        &query,
        current_search_mode(runtime),
    ));
}

fn repeat_search_previous(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    let query = runtime.last_search_query.clone();
    runtime.status = search_repeat_status(repeat_search_previous_with_mode(
        document,
        cursor,
        &query,
        current_search_mode(runtime),
    ));
}

fn search_repeat_status(result: SearchRepeatResult) -> String {
    match result {
        SearchRepeatResult::NoPreviousSearch => String::from("No previous search"),
        SearchRepeatResult::Found { query } => format!("Found: {query}"),
        SearchRepeatResult::NoMatch { query } => format!("No match: {query}"),
    }
}

fn go_to_line_status(result: GoToLineResult) -> String {
    match result {
        GoToLineResult::Empty => String::from("Line number is empty"),
        GoToLineResult::Invalid => String::from("Line number is invalid"),
        GoToLineResult::OutOfRange { line_number } => {
            format!("Line out of range: {line_number}")
        }
        GoToLineResult::Moved { line_number } => format!("Line {line_number}"),
    }
}

fn start_goto_line(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.search_active = false;
    runtime.search_query.clear();
    runtime.goto_line_active = true;
    runtime.goto_line_query.clear();
    runtime.status = String::from("Go to line: ");
}

fn toggle_line_numbers(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.show_line_numbers = !runtime.settings.show_line_numbers;
    runtime.status = if runtime.settings.show_line_numbers {
        String::from("Line numbers on")
    } else {
        String::from("Line numbers off")
    };
    persist_runtime_settings(runtime);
}

fn cycle_theme(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.theme_id = runtime.settings.theme_id.next();
    runtime.status = format!("Theme: {}", runtime.settings.theme_id.label());
    persist_runtime_settings(runtime);
}

fn cycle_syntax_theme(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.syntax_theme_id = runtime.settings.syntax_theme_id.next();
    runtime.status = format!("Syntax theme: {}", runtime.settings.syntax_theme_id.label());
    persist_runtime_settings(runtime);
}

fn toggle_search_case(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.search_case_sensitive = !runtime.settings.search_case_sensitive;
    persist_runtime_settings(runtime);
    runtime.status = if runtime.settings.search_case_sensitive {
        String::from("Search exact case")
    } else {
        String::from("Search ignore case")
    };
}

fn current_search_mode(runtime: &EditorRuntime) -> SearchMode {
    SearchMode {
        case_sensitive: runtime.settings.search_case_sensitive,
    }
}

fn toggle_wrap(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.wrap_lines = !runtime.settings.wrap_lines;
    runtime.status = if runtime.settings.wrap_lines {
        String::from("Wrap on")
    } else {
        String::from("Wrap off")
    };
    persist_runtime_settings(runtime);
}

fn toggle_reader_mode(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.reader_scroll_milli_lines = 0;
    runtime.settings.gui_reader_mode_enabled = !runtime.settings.gui_reader_mode_enabled;
    runtime.status = if runtime.settings.gui_reader_mode_enabled {
        format!(
            "Reader mode on: {} lines/min",
            runtime.settings.gui_reader_lines_per_minute
        )
    } else {
        String::from("Reader mode off")
    };
    persist_runtime_settings(runtime);
}

fn adjust_reader_speed(runtime: &mut EditorRuntime, delta: i16) {
    runtime.quit_confirmation_pending = false;
    runtime.reader_scroll_milli_lines = 0;
    let current = i16::try_from(runtime.settings.gui_reader_lines_per_minute)
        .unwrap_or(DEFAULT_GUI_READER_LINES_PER_MINUTE as i16);
    let next = current.saturating_add(delta).clamp(
        MIN_GUI_READER_LINES_PER_MINUTE as i16,
        MAX_GUI_READER_LINES_PER_MINUTE as i16,
    ) as u16;
    runtime.settings.gui_reader_lines_per_minute = next;
    runtime.status = format!("Reader speed: {next} lines/min");
    persist_runtime_settings(runtime);
}

fn stop_reader_mode(runtime: &mut EditorRuntime, status: impl Into<String>) {
    if runtime.settings.gui_reader_mode_enabled {
        runtime.settings.gui_reader_mode_enabled = false;
        runtime.reader_scroll_milli_lines = 0;
        runtime.status = status.into();
        persist_runtime_settings(runtime);
    }
}

fn stop_reader_mode_for_edit(runtime: &mut EditorRuntime) {
    stop_reader_mode(runtime, "Reader mode stopped for edit");
}

fn apply_reader_tick(
    document: &TextDocument,
    tab_state: &mut EditorTabState,
    runtime: &mut EditorRuntime,
    visible_rows: usize,
) -> bool {
    if !runtime.settings.gui_reader_mode_enabled {
        return false;
    }

    let line_count = document.buffer.line_count().max(1);
    let max_start = line_count.saturating_sub(visible_rows.max(1));
    if tab_state.viewport_start >= max_start {
        stop_reader_mode(runtime, "Reader mode stopped at document end");
        return true;
    }

    let speed = u32::from(runtime.settings.gui_reader_lines_per_minute.max(1));
    let milli_lines_per_tick = speed
        .saturating_mul(TUI_READER_TICK_MS as u32)
        .saturating_mul(1000)
        / 60_000;
    runtime.reader_scroll_milli_lines = runtime
        .reader_scroll_milli_lines
        .saturating_add(milli_lines_per_tick.max(1));
    let lines = (runtime.reader_scroll_milli_lines / 1000) as usize;
    if lines == 0 {
        return false;
    }

    runtime.reader_scroll_milli_lines %= 1000;
    let next_start = tab_state
        .viewport_start
        .saturating_add(lines)
        .min(max_start);
    if next_start == tab_state.viewport_start {
        stop_reader_mode(runtime, "Reader mode stopped at document end");
        return true;
    }

    tab_state.viewport_start = next_start;
    runtime.status = format!(
        "Reader mode: {} lines/min",
        runtime.settings.gui_reader_lines_per_minute
    );
    true
}

fn indent_at_cursor(document: &mut TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    for _ in 0..TAB_WIDTH {
        if document
            .buffer
            .insert_char(cursor.row, cursor.column, ' ')
            .is_err()
        {
            return;
        }
        cursor.column += 1;
    }
    stop_reader_mode_for_edit(runtime);
    runtime.status = String::from("Indented");
}

fn unindent_at_cursor(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    let Some(prefix) = document
        .buffer
        .line(cursor.row)
        .map(|line| line.chars().take(cursor.column).collect::<Vec<_>>())
    else {
        return;
    };
    let removable = prefix
        .iter()
        .rev()
        .take(TAB_WIDTH)
        .take_while(|character| **character == ' ')
        .count();

    if removable == 0 {
        runtime.status = String::from("No indentation to remove");
        return;
    }

    for _ in 0..removable {
        let delete_column = cursor.column.saturating_sub(1);
        if document
            .buffer
            .delete_char(cursor.row, delete_column)
            .is_err()
        {
            return;
        }
        cursor.column = delete_column;
    }
    stop_reader_mode_for_edit(runtime);
    runtime.status = String::from("Unindented");
}

fn page_up(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_page_up(document, cursor, runtime.page_rows);
    runtime.status = String::from("Page up");
}

fn page_down(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_page_down(document, cursor, runtime.page_rows);
    runtime.status = String::from("Page down");
}

fn go_to_document_start(cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_go_to_document_start(cursor);
    runtime.status = String::from("Top");
}

fn go_to_document_end(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_go_to_document_end(document, cursor);
    runtime.status = String::from("Bottom");
}

fn go_to_previous_word(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    move_document_cursor(document, cursor, CursorMove::WordLeft);
    runtime.status = String::from("Previous word");
}

fn go_to_next_word(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    move_document_cursor(document, cursor, CursorMove::WordRight);
    runtime.status = String::from("Next word");
}

fn handle_search_key_event(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match (event.modifiers, event.code) {
        (modifiers, KeyCode::Char('f') | KeyCode::Char('F'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            toggle_search_case(runtime);
            runtime.search_active = true;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        (_, KeyCode::Esc) => {
            runtime.search_active = false;
            runtime.search_history_index = None;
            runtime.status = String::from("Search canceled");
        }
        (_, KeyCode::Enter) => {
            if runtime.search_query.is_empty() {
                runtime.status = String::from("Search query is empty");
            } else {
                runtime.last_search_query = runtime.search_query.clone();
                remember_search_query(runtime);
                if let Some(found) = document.buffer.find_next_with_mode(
                    &runtime.search_query,
                    *cursor,
                    current_search_mode(runtime),
                ) {
                    *cursor = found;
                    runtime.status = format!("Found: {}", runtime.search_query);
                } else {
                    runtime.status = format!("No match: {}", runtime.search_query);
                }
            }
            runtime.search_active = false;
            runtime.search_history_index = None;
        }
        (_, KeyCode::Up) => recall_previous_search_history(runtime),
        (_, KeyCode::Down) => recall_next_search_history(runtime),
        (_, KeyCode::Backspace) => {
            runtime.search_query.pop();
            runtime.search_history_index = None;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value)) => {
            runtime.search_query.push(value);
            runtime.search_history_index = None;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        _ => {}
    }
}

fn remember_search_query(runtime: &mut EditorRuntime) {
    let query = runtime.search_query.trim();
    if query.is_empty() {
        return;
    }
    if let Some(existing) = runtime.search_history.iter().position(|item| item == query) {
        runtime.search_history.remove(existing);
    }
    runtime.search_history.insert(0, query.to_string());
    runtime.search_history.truncate(10);
}

fn recall_previous_search_history(runtime: &mut EditorRuntime) {
    if runtime.search_history.is_empty() {
        runtime.status = String::from("Search history empty");
        return;
    }
    let next_index = runtime
        .search_history_index
        .map_or(0, |index| (index + 1).min(runtime.search_history.len() - 1));
    runtime.search_history_index = Some(next_index);
    runtime.search_query = runtime.search_history[next_index].clone();
    runtime.status = format!("Search: {}", runtime.search_query);
}

fn recall_next_search_history(runtime: &mut EditorRuntime) {
    let Some(index) = runtime.search_history_index else {
        runtime.status = format!("Search: {}", runtime.search_query);
        return;
    };
    if index == 0 {
        runtime.search_history_index = None;
        runtime.search_query.clear();
    } else {
        let next_index = index - 1;
        runtime.search_history_index = Some(next_index);
        runtime.search_query = runtime.search_history[next_index].clone();
    }
    runtime.status = format!("Search: {}", runtime.search_query);
}

fn handle_goto_line_key_event(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match (event.modifiers, event.code) {
        (_, KeyCode::Esc) => {
            runtime.goto_line_active = false;
            runtime.status = String::from("Go to line canceled");
        }
        (_, KeyCode::Enter) => {
            runtime.status = go_to_line_status(shared_go_to_line(
                document,
                cursor,
                &runtime.goto_line_query,
            ));
            runtime.goto_line_active = false;
        }
        (_, KeyCode::Backspace) => {
            runtime.goto_line_query.pop();
            runtime.status = format!("Go to line: {}", runtime.goto_line_query);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value))
            if value.is_ascii_digit() =>
        {
            runtime.goto_line_query.push(value);
            runtime.status = format!("Go to line: {}", runtime.goto_line_query);
        }
        _ => {}
    }
}

fn move_cursor(document: &TextDocument, cursor: &mut Cursor, direction: CursorMove) {
    move_document_cursor(document, cursor, direction);
}

#[derive(Debug, PartialEq, Eq)]
struct EditorRuntime {
    status: String,
    quit_confirmation_pending: bool,
    close_tab_confirmation_pending: bool,
    search_active: bool,
    search_query: String,
    last_search_query: String,
    search_history: Vec<String>,
    search_history_index: Option<usize>,
    goto_line_active: bool,
    goto_line_query: String,
    menu: Option<MenuState>,
    page_rows: usize,
    settings: EditorSettings,
    config_path: Option<PathBuf>,
    workspace_projects_dir: Option<PathBuf>,
    workspace_prompt: Option<WorkspacePrompt>,
    workspace_query: String,
    workspace_pending_open: Option<GuiWorkspaceProject>,
    workspace_pending_delete: Option<(String, PathBuf)>,
    workspace_prompt_candidates: Vec<String>,
    workspace_prompt_candidate_index: Option<usize>,
    workspace_open_confirmation_pending: bool,
    workspace_manager: Option<WorkspaceManagerState>,
    sidebar: Option<FileSidebarState>,
    last_sidebar_dir: Option<PathBuf>,
    sidebar_prompt: Option<SidebarPrompt>,
    sidebar_query: String,
    overwrite_mode: bool,
    reader_scroll_milli_lines: u32,
    command_palette: Option<CommandPaletteState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspacePrompt {
    SaveNamed,
    OpenNamed,
    DeleteNamed,
    ConfirmOpen,
    ConfirmDelete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkspaceManagerState {
    entries: Vec<WorkspaceManagerEntry>,
    selected: usize,
    scroll: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkspaceManagerEntry {
    name: String,
    files: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CommandPaletteState {
    query: String,
    selected: usize,
    scroll: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CommandPaletteEntry {
    group: MenuGroup,
    label: &'static str,
    shortcut: Option<&'static str>,
    command: MenuCommand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SidebarPrompt {
    CreateFile,
    CreateDirectory,
    DeleteConfirm {
        entry: FileSidebarEntry,
        recursive: bool,
    },
}

fn select_previous_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    if !workspace.select_previous_tab() {
        runtime.status = String::from("Only one tab open");
        return;
    }
    stop_reader_mode(runtime, "Reader mode stopped for tab switch");
    runtime.status = active_tab_status(workspace);
    autosave_tui_current_workspace(workspace, runtime);
}

fn select_next_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    if !workspace.select_next_tab() {
        runtime.status = String::from("Only one tab open");
        return;
    }
    stop_reader_mode(runtime, "Reader mode stopped for tab switch");
    runtime.status = active_tab_status(workspace);
    autosave_tui_current_workspace(workspace, runtime);
}

fn close_active_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    match workspace.close_active_tab(runtime.close_tab_confirmation_pending) {
        CloseActiveTabResult::OnlyTab => {
            runtime.close_tab_confirmation_pending = false;
            runtime.status = String::from("Cannot close the only tab");
        }
        CloseActiveTabResult::Dirty => {
            runtime.close_tab_confirmation_pending = true;
            runtime.status = String::from("Unsaved changes. Press Ctrl-F4 again to close tab.");
        }
        CloseActiveTabResult::Closed { path } => {
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for tab close");
            runtime.status = format!("Closed tab: {}", display_file_name(&path));
            autosave_tui_current_workspace(workspace, runtime);
        }
    }
}

fn active_tab_status(workspace: &EditorWorkspace<'_>) -> String {
    let tab = workspace.active_tab();
    format!(
        "Tab {}/{}: {}",
        workspace.active + 1,
        workspace.tabs.len(),
        display_file_name(&tab.document.as_ref().path)
    )
}

fn display_file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled")
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self {
            status: String::from("Ctrl-S save | Ctrl-Q quit"),
            quit_confirmation_pending: false,
            close_tab_confirmation_pending: false,
            search_active: false,
            search_query: String::new(),
            last_search_query: String::new(),
            search_history: Vec::new(),
            search_history_index: None,
            goto_line_active: false,
            goto_line_query: String::new(),
            menu: None,
            page_rows: 20,
            settings: EditorSettings::default(),
            config_path: None,
            workspace_projects_dir: None,
            workspace_prompt: None,
            workspace_query: String::new(),
            workspace_pending_open: None,
            workspace_pending_delete: None,
            workspace_prompt_candidates: Vec::new(),
            workspace_prompt_candidate_index: None,
            workspace_open_confirmation_pending: false,
            workspace_manager: None,
            sidebar: None,
            last_sidebar_dir: None,
            sidebar_prompt: None,
            sidebar_query: String::new(),
            overwrite_mode: false,
            reader_scroll_milli_lines: 0,
            command_palette: None,
        }
    }
}

impl EditorRuntime {
    fn search_highlight(&self) -> Option<SearchHighlightView<'_>> {
        if self.last_search_query.is_empty() {
            return None;
        }
        Some(SearchHighlightView {
            query: &self.last_search_query,
            mode: current_search_mode(self),
        })
    }
}

fn current_editor_config_path() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = env::var_os("HOME").map(PathBuf::from);
    editor_config_path(xdg_config_home.as_deref(), home.as_deref())
}

fn current_workspace_projects_dir() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = env::var_os("HOME").map(PathBuf::from);
    gui_workspace_projects_dir(xdg_config_home.as_deref(), home.as_deref())
        .map(|path| path.join(TUI_WORKSPACE_DIR_NAME))
}

fn current_tui_restore_project_request() -> Option<(PathBuf, EditorSettings)> {
    let config_path = current_editor_config_path()?;
    let settings = load_editor_settings(&config_path).ok()?;
    if !settings.gui_restore_last_workspace {
        return None;
    }
    let projects_dir = current_workspace_projects_dir()?;
    let project_path = gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME)?;
    Some((project_path, settings))
}

fn persist_runtime_settings(runtime: &mut EditorRuntime) {
    let Some(config_path) = runtime.config_path.as_deref() else {
        return;
    };

    if let Err(error) = save_editor_settings(config_path, runtime.settings) {
        runtime.status = format!("{}; config not saved: {error}", runtime.status);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MenuState {
    group: MenuGroup,
    selected: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MenuGroup {
    #[default]
    File,
    Edit,
    View,
    Go,
    Tabs,
    Workspace,
    Help,
}

impl MenuGroup {
    fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Go => "Go",
            Self::Tabs => "Tabs",
            Self::Workspace => "Workspace",
            Self::Help => "Help",
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::File => Self::Help,
            Self::Edit => Self::File,
            Self::View => Self::Edit,
            Self::Go => Self::View,
            Self::Tabs => Self::Go,
            Self::Workspace => Self::Tabs,
            Self::Help => Self::Workspace,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::File => Self::Edit,
            Self::Edit => Self::View,
            Self::View => Self::Go,
            Self::Go => Self::Tabs,
            Self::Tabs => Self::Workspace,
            Self::Workspace => Self::Help,
            Self::Help => Self::File,
        }
    }

    fn items(self) -> &'static [MenuItem] {
        match self {
            Self::File => &[
                MenuItem {
                    label: "New file",
                    shortcut: Some("Ctrl-N"),
                    command: MenuCommand::NewFile,
                },
                MenuItem {
                    label: "Save",
                    shortcut: Some("Ctrl-S"),
                    command: MenuCommand::Save,
                },
                MenuItem {
                    label: "Files",
                    shortcut: Some("Ctrl-B"),
                    command: MenuCommand::ToggleSidebar,
                },
                MenuItem {
                    label: "Quit",
                    shortcut: Some("Ctrl-Q"),
                    command: MenuCommand::Quit,
                },
            ],
            Self::Edit => &[
                MenuItem {
                    label: "Find",
                    shortcut: Some("Ctrl-F"),
                    command: MenuCommand::Find,
                },
                MenuItem {
                    label: "Exact case",
                    shortcut: Some("Ctrl-Shift-F"),
                    command: MenuCommand::ToggleSearchCase,
                },
                MenuItem {
                    label: "Undo",
                    shortcut: Some("Ctrl-Z"),
                    command: MenuCommand::Undo,
                },
                MenuItem {
                    label: "Redo",
                    shortcut: Some("Ctrl-Y"),
                    command: MenuCommand::Redo,
                },
                MenuItem {
                    label: "Delete previous word",
                    shortcut: Some("Ctrl-Backspace"),
                    command: MenuCommand::DeletePreviousWord,
                },
                MenuItem {
                    label: "Delete next word",
                    shortcut: Some("Ctrl-Delete"),
                    command: MenuCommand::DeleteNextWord,
                },
                MenuItem {
                    label: "Delete to line end",
                    shortcut: Some("Ctrl-K"),
                    command: MenuCommand::DeleteToLineEnd,
                },
                MenuItem {
                    label: "Find next",
                    shortcut: Some("F3"),
                    command: MenuCommand::FindNext,
                },
                MenuItem {
                    label: "Find previous",
                    shortcut: Some("Shift-F3"),
                    command: MenuCommand::FindPrevious,
                },
            ],
            Self::View => &[
                MenuItem {
                    label: "Line numbers",
                    shortcut: Some("Ctrl-L"),
                    command: MenuCommand::ToggleLineNumbers,
                },
                MenuItem {
                    label: "Theme",
                    shortcut: Some("Ctrl-T"),
                    command: MenuCommand::CycleTheme,
                },
                MenuItem {
                    label: "Syntax theme",
                    shortcut: Some("Ctrl-Shift-T"),
                    command: MenuCommand::CycleSyntaxTheme,
                },
                MenuItem {
                    label: "Reader mode",
                    shortcut: Some("Ctrl-R"),
                    command: MenuCommand::ToggleReaderMode,
                },
                MenuItem {
                    label: "Reader slower",
                    shortcut: None,
                    command: MenuCommand::DecreaseReaderSpeed,
                },
                MenuItem {
                    label: "Reader faster",
                    shortcut: None,
                    command: MenuCommand::IncreaseReaderSpeed,
                },
                MenuItem {
                    label: "Word wrap",
                    shortcut: Some("Ctrl-W"),
                    command: MenuCommand::ToggleWrap,
                },
            ],
            Self::Go => &[
                MenuItem {
                    label: "Page up",
                    shortcut: Some("PageUp"),
                    command: MenuCommand::PageUp,
                },
                MenuItem {
                    label: "Page down",
                    shortcut: Some("PageDown"),
                    command: MenuCommand::PageDown,
                },
                MenuItem {
                    label: "Top",
                    shortcut: Some("Ctrl-Home"),
                    command: MenuCommand::DocumentStart,
                },
                MenuItem {
                    label: "Bottom",
                    shortcut: Some("Ctrl-End"),
                    command: MenuCommand::DocumentEnd,
                },
                MenuItem {
                    label: "Go to line",
                    shortcut: Some("Ctrl-G"),
                    command: MenuCommand::GoToLine,
                },
                MenuItem {
                    label: "Previous word",
                    shortcut: Some("Ctrl-Left"),
                    command: MenuCommand::PreviousWord,
                },
                MenuItem {
                    label: "Next word",
                    shortcut: Some("Ctrl-Right"),
                    command: MenuCommand::NextWord,
                },
            ],
            Self::Tabs => &[
                MenuItem {
                    label: "Previous tab",
                    shortcut: Some("Ctrl-PageUp"),
                    command: MenuCommand::PreviousTab,
                },
                MenuItem {
                    label: "Next tab",
                    shortcut: Some("Ctrl-PageDown"),
                    command: MenuCommand::NextTab,
                },
                MenuItem {
                    label: "Close tab",
                    shortcut: Some("Ctrl-F4"),
                    command: MenuCommand::CloseTab,
                },
                MenuItem {
                    label: "Open sidebar file as tab",
                    shortcut: Some("Ctrl-B, Ctrl-Enter"),
                    command: MenuCommand::HelpOnly,
                },
            ],
            Self::Workspace => &[
                MenuItem {
                    label: "Save current",
                    shortcut: None,
                    command: MenuCommand::SaveCurrentWorkspace,
                },
                MenuItem {
                    label: "Save named",
                    shortcut: None,
                    command: MenuCommand::SaveNamedWorkspace,
                },
                MenuItem {
                    label: "Manage projects",
                    shortcut: None,
                    command: MenuCommand::ListWorkspaces,
                },
                MenuItem {
                    label: "Open project",
                    shortcut: None,
                    command: MenuCommand::OpenWorkspace,
                },
                MenuItem {
                    label: "Delete project",
                    shortcut: None,
                    command: MenuCommand::DeleteWorkspace,
                },
                MenuItem {
                    label: "Open current",
                    shortcut: None,
                    command: MenuCommand::OpenCurrentWorkspace,
                },
                MenuItem {
                    label: "Restore last",
                    shortcut: None,
                    command: MenuCommand::ToggleRestoreLastWorkspace,
                },
            ],
            Self::Help => &[
                MenuItem {
                    label: "Open help document",
                    shortcut: Some("F10"),
                    command: MenuCommand::OpenHelp,
                },
                MenuItem {
                    label: "Command palette",
                    shortcut: Some("F2"),
                    command: MenuCommand::OpenCommandPalette,
                },
                MenuItem {
                    label: "Files and tabs",
                    shortcut: Some("Ctrl-B / Ctrl-Enter / Ctrl-F4"),
                    command: MenuCommand::HelpOnly,
                },
                MenuItem {
                    label: "Search and go",
                    shortcut: Some("Ctrl-F / F3 / Shift-F3 / Ctrl-G"),
                    command: MenuCommand::HelpOnly,
                },
                MenuItem {
                    label: "Editing",
                    shortcut: Some("Ctrl-Z/Y / Ctrl-K / Insert"),
                    command: MenuCommand::HelpOnly,
                },
                MenuItem {
                    label: "View and reader",
                    shortcut: Some("Ctrl-L/T/R/W"),
                    command: MenuCommand::HelpOnly,
                },
                MenuItem {
                    label: "Workspaces",
                    shortcut: Some("F10 -> Workspace"),
                    command: MenuCommand::HelpOnly,
                },
                MenuItem {
                    label: "Save and quit",
                    shortcut: Some("Ctrl-S / Ctrl-Q"),
                    command: MenuCommand::HelpOnly,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuItem {
    label: &'static str,
    shortcut: Option<&'static str>,
    command: MenuCommand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuCommand {
    NewFile,
    Save,
    Quit,
    ToggleSidebar,
    Find,
    ToggleSearchCase,
    Undo,
    Redo,
    DeletePreviousWord,
    DeleteNextWord,
    DeleteToLineEnd,
    FindNext,
    FindPrevious,
    GoToLine,
    ToggleLineNumbers,
    CycleTheme,
    CycleSyntaxTheme,
    ToggleReaderMode,
    DecreaseReaderSpeed,
    IncreaseReaderSpeed,
    ToggleWrap,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
    PreviousWord,
    NextWord,
    PreviousTab,
    NextTab,
    CloseTab,
    SaveCurrentWorkspace,
    SaveNamedWorkspace,
    ListWorkspaces,
    OpenWorkspace,
    DeleteWorkspace,
    OpenCurrentWorkspace,
    ToggleRestoreLastWorkspace,
    OpenHelp,
    OpenCommandPalette,
    HelpOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EditorTheme {
    header_fg: Color,
    header_bg: Color,
    gutter_fg: Color,
    status_fg: Color,
    status_bg: Color,
    search_fg: Color,
    search_bg: Color,
    help_fg: Color,
    help_bg: Color,
    dirty_fg: Color,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self::for_id(EditorThemeId::Nocturne)
    }
}

impl EditorTheme {
    fn for_id(theme_id: EditorThemeId) -> Self {
        match theme_id {
            EditorThemeId::Nocturne => Self {
                header_fg: Color::White,
                header_bg: Color::DarkBlue,
                gutter_fg: Color::DarkGrey,
                status_fg: Color::Black,
                status_bg: Color::Cyan,
                search_fg: Color::Rgb { r: 0, g: 0, b: 0 },
                search_bg: Color::Rgb {
                    r: 90,
                    g: 230,
                    b: 245,
                },
                help_fg: Color::Grey,
                help_bg: Color::Black,
                dirty_fg: Color::Yellow,
            },
            EditorThemeId::Aurora => Self {
                header_fg: Color::Black,
                header_bg: Color::Green,
                gutter_fg: Color::DarkCyan,
                status_fg: Color::Black,
                status_bg: Color::Magenta,
                search_fg: Color::Rgb { r: 0, g: 0, b: 0 },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 120,
                    b: 220,
                },
                help_fg: Color::Cyan,
                help_bg: Color::Black,
                dirty_fg: Color::Yellow,
            },
            EditorThemeId::Paper => Self {
                header_fg: Color::Rgb {
                    r: 245,
                    g: 226,
                    b: 244,
                },
                header_bg: Color::Rgb {
                    r: 118,
                    g: 67,
                    b: 169,
                },
                gutter_fg: Color::Rgb {
                    r: 155,
                    g: 48,
                    b: 96,
                },
                status_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                status_bg: Color::Rgb {
                    r: 236,
                    g: 180,
                    b: 224,
                },
                search_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 207,
                    b: 119,
                },
                help_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                help_bg: Color::Rgb {
                    r: 245,
                    g: 226,
                    b: 244,
                },
                dirty_fg: Color::Rgb {
                    r: 139,
                    g: 83,
                    b: 31,
                },
            },
            EditorThemeId::Terminal => Self {
                header_fg: Color::Rgb {
                    r: 168,
                    g: 255,
                    b: 168,
                },
                header_bg: Color::Rgb { r: 0, g: 36, b: 12 },
                gutter_fg: Color::DarkGreen,
                status_fg: Color::Black,
                status_bg: Color::Rgb {
                    r: 72,
                    g: 255,
                    b: 112,
                },
                search_fg: Color::Rgb { r: 0, g: 20, b: 7 },
                search_bg: Color::Rgb {
                    r: 154,
                    g: 255,
                    b: 104,
                },
                help_fg: Color::Rgb {
                    r: 114,
                    g: 215,
                    b: 132,
                },
                help_bg: Color::Black,
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 228,
                    b: 92,
                },
            },
            EditorThemeId::Abyss => Self {
                header_fg: Color::Rgb {
                    r: 102,
                    g: 229,
                    b: 255,
                },
                header_bg: Color::Rgb { r: 8, g: 15, b: 32 },
                gutter_fg: Color::Rgb {
                    r: 72,
                    g: 88,
                    b: 122,
                },
                status_fg: Color::Rgb {
                    r: 206,
                    g: 240,
                    b: 255,
                },
                status_bg: Color::Rgb {
                    r: 18,
                    g: 33,
                    b: 58,
                },
                search_fg: Color::Rgb { r: 4, g: 12, b: 26 },
                search_bg: Color::Rgb {
                    r: 112,
                    g: 236,
                    b: 255,
                },
                help_fg: Color::Rgb {
                    r: 141,
                    g: 188,
                    b: 205,
                },
                help_bg: Color::Rgb { r: 3, g: 7, b: 18 },
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 64,
                    b: 96,
                },
            },
            EditorThemeId::Terror => Self {
                header_fg: Color::Rgb {
                    r: 255,
                    g: 42,
                    b: 160,
                },
                header_bg: Color::Rgb { r: 45, g: 0, b: 58 },
                gutter_fg: Color::Rgb {
                    r: 166,
                    g: 80,
                    b: 190,
                },
                status_fg: Color::Rgb {
                    r: 255,
                    g: 188,
                    b: 236,
                },
                status_bg: Color::Rgb { r: 78, g: 0, b: 92 },
                search_fg: Color::Rgb { r: 31, g: 0, b: 36 },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 75,
                    b: 184,
                },
                help_fg: Color::Rgb {
                    r: 255,
                    g: 88,
                    b: 190,
                },
                help_bg: Color::Rgb { r: 24, g: 0, b: 30 },
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 238,
                    b: 70,
                },
            },
        }
    }
}

fn render_editor(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
) -> io::Result<()> {
    render_editor_with_width(writer, document, view, highlighter, terminal_width())
}

fn render_editor_with_width(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    terminal_width: usize,
) -> io::Result<()> {
    let theme = EditorTheme::for_id(view.settings.theme_id);
    let sidebar_width = view.sidebar_width.min(terminal_width.saturating_sub(1));
    let terminal_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let frame = RenderFrame {
        theme,
        gutter_width: line_number_width(document),
        terminal_width,
        origin_column: sidebar_width as u16,
        body_top: tab_strip_height_for_width(view.tab_strip, terminal_width),
    };
    write_header(writer, document, view.menu, frame)?;
    if !view.tab_strip.is_empty() {
        write_tab_strip(writer, view.tab_strip, frame)?;
    }

    if view.settings.wrap_lines {
        write_wrapped_editor_lines(writer, document, view, highlighter, frame)?;
    } else {
        let highlighted_lines =
            highlighter.highlight_visible_lines(document, view.viewport_start, view.visible_rows);
        for body_index in 0..view.visible_rows {
            let document_row = view.viewport_start + body_index;
            let screen_row = frame.body_top + body_index as u16;
            if let Some(line) = document.buffer.lines().get(document_row) {
                write_editor_line(
                    writer,
                    EditorLineView {
                        screen_row,
                        document_row,
                        line,
                        settings: view.settings,
                        horizontal_offset: view.horizontal_offset,
                        highlighted_line: highlighted_lines.get(body_index).cloned().flatten(),
                        search_highlight: view.search_highlight,
                    },
                    frame,
                )?;
            } else {
                clear_editor_body_row(writer, screen_row, frame)?;
            }
        }
    }

    if let Some(menu) = view.menu {
        write_menu_dropdown(writer, menu, frame)?;
    }

    let status_screen_row = frame.body_top + view.visible_rows as u16;
    let status_cursor_column = write_status_line(
        writer,
        StatusLineView {
            document,
            cursor: view.cursor,
            status: view.status,
            settings: view.settings,
            horizontal_offset: view.horizontal_offset,
            screen_row: status_screen_row,
        },
        frame,
    )?;
    write_help_line(writer, status_screen_row + 1, frame)?;
    if let Some(status_cursor_column) = status_cursor_column {
        queue!(
            writer,
            Show,
            frame.move_to(status_cursor_column, status_screen_row)
        )?;
    } else if let Some(menu) = view.menu {
        queue!(
            writer,
            Show,
            frame.move_to(
                menu_dropdown_column(menu.group, frame).saturating_add(2),
                (menu.selected + 1) as u16
            )
        )?;
    } else if !cursor_row_is_visible(document, view, frame) {
        queue!(writer, Hide)?;
    } else {
        write_cursor_cell_highlight(writer, document, view, frame)?;
        queue!(
            writer,
            Show,
            frame.move_to(
                cursor_screen_column(document, view.cursor, view, frame),
                cursor_screen_row(document, view, frame)
            )
        )?;
    }
    writer.flush()
}

fn clear_editor_body_row(
    writer: &mut impl Write,
    screen_row: u16,
    frame: RenderFrame,
) -> io::Result<()> {
    queue!(
        writer,
        frame.move_to(0, screen_row),
        Clear(ClearType::CurrentLine)
    )
}

fn tab_strip_height_for_width(tab_strip: &[TabStripItem], terminal_width: usize) -> u16 {
    if tab_strip.len() <= 1 {
        return 1;
    }
    1 + tab_strip_rows_for_width(tab_strip, terminal_width) as u16
}

fn tab_strip_rows_for_width(tab_strip: &[TabStripItem], terminal_width: usize) -> usize {
    let width = terminal_width.max(1);
    let mut rows = 1usize;
    let mut remaining = width;
    for (index, item) in tab_strip.iter().enumerate() {
        let label_width = text_display_width(&compose_tab_label(index, item)).max(1);
        if label_width > remaining && remaining < width {
            rows += 1;
            remaining = width;
        }
        remaining = remaining.saturating_sub(label_width.min(remaining));
    }
    rows
}

fn write_tab_strip(
    writer: &mut impl Write,
    tab_strip: &[TabStripItem],
    frame: RenderFrame,
) -> io::Result<()> {
    let mut row = 1u16;
    let mut remaining = frame.terminal_width;
    clear_tab_strip_rows(writer, frame)?;
    for (index, item) in tab_strip.iter().enumerate() {
        if row >= frame.body_top {
            break;
        }
        let label = compose_tab_label(index, item);
        let label_width = text_display_width(&label).max(1);
        if label_width > remaining && remaining < frame.terminal_width {
            row += 1;
            remaining = frame.terminal_width;
        }
        if row >= frame.body_top {
            break;
        }
        queue!(
            writer,
            frame.move_to(frame.terminal_width.saturating_sub(remaining) as u16, row),
        )?;
        if item.active {
            queue!(
                writer,
                SetForegroundColor(frame.theme.header_fg),
                SetBackgroundColor(frame.theme.header_bg),
                SetAttribute(Attribute::Bold),
            )?;
        } else {
            queue!(
                writer,
                SetForegroundColor(frame.theme.help_fg),
                SetBackgroundColor(frame.theme.help_bg),
                SetAttribute(Attribute::Reset),
            )?;
        }
        print_truncated(writer, &label, &mut remaining)?;
    }
    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)
}

fn clear_tab_strip_rows(writer: &mut impl Write, frame: RenderFrame) -> io::Result<()> {
    for row in 1..frame.body_top {
        queue!(
            writer,
            frame.move_to(0, row),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(frame.theme.help_fg),
            SetBackgroundColor(frame.theme.help_bg),
        )?;
    }
    Ok(())
}

fn compose_tab_label(index: usize, item: &TabStripItem) -> String {
    let dirty = if item.dirty { "*" } else { "" };
    format!(" {}:{}{} ", index + 1, item.label, dirty)
}

#[derive(Clone, Copy)]
struct RenderFrame {
    theme: EditorTheme,
    gutter_width: usize,
    terminal_width: usize,
    origin_column: u16,
    body_top: u16,
}

impl RenderFrame {
    fn move_to(self, column: u16, row: u16) -> MoveTo {
        MoveTo(self.origin_column.saturating_add(column), row)
    }
}

#[derive(Clone, Copy)]
struct EditorView<'a> {
    cursor: Cursor,
    viewport_start: usize,
    horizontal_offset: usize,
    visible_rows: usize,
    status: &'a str,
    settings: EditorSettings,
    menu: Option<MenuState>,
    sidebar_width: usize,
    tab_strip: &'a [TabStripItem],
    search_highlight: Option<SearchHighlightView<'a>>,
}

#[derive(Clone, Copy)]
struct SearchHighlightView<'a> {
    query: &'a str,
    mode: SearchMode,
}

struct EditorLineView<'a> {
    screen_row: u16,
    document_row: usize,
    line: &'a str,
    settings: EditorSettings,
    horizontal_offset: usize,
    highlighted_line: Option<Vec<(SyntectStyle, String)>>,
    search_highlight: Option<SearchHighlightView<'a>>,
}

fn write_header(
    writer: &mut impl Write,
    document: &TextDocument,
    menu: Option<MenuState>,
    frame: RenderFrame,
) -> io::Result<()> {
    let state = if document.buffer.is_dirty() {
        " modified "
    } else {
        " saved "
    };
    let label = " kfnotepad ";
    let show_menu_bar = show_menu_bar(frame);
    let reserved = text_display_width(label)
        + text_display_width(state)
        + if show_menu_bar {
            text_display_width(menu_bar_text())
        } else {
            0
        };
    let path_width = frame.terminal_width.saturating_sub(reserved).max(1);
    let path = fit_text_end(&format!(" {} ", document.path.display()), path_width);

    let mut remaining = frame.terminal_width;
    queue!(
        writer,
        frame.move_to(0, 0),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(frame.theme.header_fg),
        SetBackgroundColor(frame.theme.header_bg),
        SetAttribute(Attribute::Bold),
    )?;
    print_truncated(writer, label, &mut remaining)?;
    if show_menu_bar {
        write_menu_bar(writer, menu, frame, &mut remaining)?;
    }
    queue!(
        writer,
        SetAttribute(Attribute::Reset),
        SetForegroundColor(frame.theme.header_fg),
        SetBackgroundColor(frame.theme.header_bg),
    )?;
    print_truncated(writer, &path, &mut remaining)?;
    if document.buffer.is_dirty() {
        queue!(
            writer,
            SetForegroundColor(frame.theme.dirty_fg),
            SetBackgroundColor(frame.theme.header_bg),
            SetAttribute(Attribute::Bold),
        )?;
        print_truncated(writer, state, &mut remaining)?;
        queue!(writer, SetAttribute(Attribute::Reset))?;
    } else {
        queue!(
            writer,
            SetForegroundColor(frame.theme.header_fg),
            SetBackgroundColor(frame.theme.header_bg),
        )?;
        print_truncated(writer, state, &mut remaining)?;
    }
    queue!(writer, ResetColor)
}

fn show_menu_bar(frame: RenderFrame) -> bool {
    text_display_width(" kfnotepad ")
        + text_display_width(menu_bar_text())
        + text_display_width(" modified ")
        <= frame.terminal_width
}

fn write_menu_bar(
    writer: &mut impl Write,
    menu: Option<MenuState>,
    frame: RenderFrame,
    remaining: &mut usize,
) -> io::Result<()> {
    for group in MENU_GROUPS {
        let active = menu.is_some_and(|menu| menu.group == group);
        if active {
            queue!(
                writer,
                SetForegroundColor(frame.theme.status_fg),
                SetBackgroundColor(frame.theme.status_bg),
                SetAttribute(Attribute::Bold),
            )?;
        } else {
            queue!(
                writer,
                SetForegroundColor(frame.theme.header_fg),
                SetBackgroundColor(frame.theme.header_bg),
                SetAttribute(Attribute::Reset),
            )?;
        }
        print_truncated(writer, &format!(" {} ", group.label()), remaining)?;
    }
    queue!(
        writer,
        SetForegroundColor(frame.theme.header_fg),
        SetBackgroundColor(frame.theme.header_bg),
        SetAttribute(Attribute::Reset),
    )?;
    print_truncated(writer, "|", remaining)
}

fn write_menu_dropdown(
    writer: &mut impl Write,
    menu: MenuState,
    frame: RenderFrame,
) -> io::Result<()> {
    let column = menu_dropdown_column(menu.group, frame);
    let available_width = frame.terminal_width.saturating_sub(column as usize);
    if available_width == 0 {
        return Ok(());
    }
    let width = menu
        .group
        .items()
        .iter()
        .map(menu_item_display_width)
        .max()
        .unwrap_or(4)
        + 4;
    let width = width.min(available_width);
    for (index, item) in menu.group.items().iter().enumerate() {
        let mut remaining = width;
        queue!(
            writer,
            ResetColor,
            SetAttribute(Attribute::Reset),
            frame.move_to(column, (index + 1) as u16)
        )?;
        if index == menu.selected {
            queue!(
                writer,
                SetForegroundColor(frame.theme.status_fg),
                SetBackgroundColor(frame.theme.status_bg),
                SetAttribute(Attribute::Bold),
            )?;
        } else {
            queue!(
                writer,
                SetForegroundColor(frame.theme.header_fg),
                SetBackgroundColor(frame.theme.header_bg),
                SetAttribute(Attribute::Reset),
            )?;
        }
        print_truncated(
            writer,
            &format_menu_item(item, width.saturating_sub(2)),
            &mut remaining,
        )?;
    }
    queue!(writer, ResetColor, SetAttribute(Attribute::Reset))
}

fn menu_item_display_width(item: &MenuItem) -> usize {
    text_display_width(&format_menu_item(item, 0))
}

fn format_menu_item(item: &MenuItem, inner_width: usize) -> String {
    match item.shortcut {
        Some(shortcut) => {
            let label_width = text_display_width(item.label);
            let shortcut_width = text_display_width(shortcut);
            let gap = inner_width
                .saturating_sub(label_width + shortcut_width)
                .max(2);
            format!("  {}{}{}", item.label, " ".repeat(gap), shortcut)
        }
        None => format!("  {}", item.label),
    }
}

fn menu_dropdown_column(group: MenuGroup, frame: RenderFrame) -> u16 {
    let mut column = text_display_width(" kfnotepad ");
    for menu_group in MENU_GROUPS {
        if menu_group == group {
            break;
        }
        column += text_display_width(&format!(" {} ", menu_group.label()));
    }
    column.min(frame.terminal_width.saturating_sub(1)) as u16
}

fn menu_bar_text() -> &'static str {
    " File  Edit  View  Go  Tabs  Workspace  Help |"
}

fn write_editor_line(
    writer: &mut impl Write,
    view: EditorLineView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    clear_editor_body_row(writer, view.screen_row, frame)?;
    if view.settings.show_line_numbers {
        queue!(writer, SetForegroundColor(frame.theme.gutter_fg),)?;
        print_truncated(
            writer,
            &format!(
                "{:>width$} ",
                view.document_row + 1,
                width = frame.gutter_width
            ),
            &mut remaining,
        )?;
        queue!(writer, ResetColor)?;
    }
    write_editor_body_padding(writer, &mut remaining)?;

    let search_ranges = view
        .search_highlight
        .map(|highlight| search_match_ranges(view.line, highlight.query, highlight.mode))
        .unwrap_or_default();
    if let Some(segments) = view.highlighted_line {
        write_highlighted_line_window(
            writer,
            segments,
            view.horizontal_offset,
            &mut remaining,
            view.settings.syntax_theme_id,
            &search_ranges,
            frame,
        )
    } else {
        let mut display_column = 0;
        let mut source_column = 0;
        print_line_window_with_search(
            writer,
            LineWindowSearchView {
                text: view.line,
                start_column: view.horizontal_offset,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: &search_ranges,
                base_fg: None,
                frame,
            },
        )?;
        Ok(())
    }
}

fn write_wrapped_editor_lines(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    frame: RenderFrame,
) -> io::Result<()> {
    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let mut screen_row = frame.body_top;
    let highlighted_lines =
        highlighter_lines_for_wrapped_view(document, view, highlighter, body_width.max(1));

    for (index, line) in document
        .buffer
        .lines()
        .iter()
        .enumerate()
        .skip(view.viewport_start)
    {
        let highlighted_line = highlighted_lines
            .get(index.saturating_sub(view.viewport_start))
            .cloned()
            .flatten();
        for (chunk_index, chunk) in wrapped_line_chunks(line, body_width)
            .into_iter()
            .enumerate()
        {
            if screen_row >= frame.body_top + view.visible_rows as u16 {
                return clear_remaining_editor_rows(writer, screen_row, view.visible_rows, frame);
            }
            write_wrapped_editor_chunk(
                writer,
                WrappedEditorChunkView {
                    screen_row,
                    document_row: index,
                    chunk_index,
                    line,
                    chunk: &chunk.text,
                    chunk_start_column: chunk.start_column,
                    highlighted_line: highlighted_line.clone(),
                    settings: view.settings,
                    search_highlight: view.search_highlight,
                },
                frame,
            )?;
            screen_row += 1;
        }
    }

    clear_remaining_editor_rows(writer, screen_row, view.visible_rows, frame)
}

fn clear_remaining_editor_rows(
    writer: &mut impl Write,
    mut screen_row: u16,
    visible_rows: usize,
    frame: RenderFrame,
) -> io::Result<()> {
    let end_row = frame.body_top + visible_rows as u16;
    while screen_row < end_row {
        clear_editor_body_row(writer, screen_row, frame)?;
        screen_row += 1;
    }
    Ok(())
}

struct WrappedEditorChunkView<'a> {
    screen_row: u16,
    document_row: usize,
    chunk_index: usize,
    line: &'a str,
    chunk: &'a str,
    chunk_start_column: usize,
    highlighted_line: Option<Vec<(SyntectStyle, String)>>,
    settings: EditorSettings,
    search_highlight: Option<SearchHighlightView<'a>>,
}

fn write_wrapped_editor_chunk(
    writer: &mut impl Write,
    view: WrappedEditorChunkView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    clear_editor_body_row(writer, view.screen_row, frame)?;
    if view.settings.show_line_numbers {
        queue!(writer, SetForegroundColor(frame.theme.gutter_fg),)?;
        let gutter = if view.chunk_index == 0 {
            format!(
                "{:>width$} ",
                view.document_row + 1,
                width = frame.gutter_width
            )
        } else {
            format!("{:>width$} ", "", width = frame.gutter_width)
        };
        print_truncated(writer, &gutter, &mut remaining)?;
        queue!(writer, ResetColor)?;
    }
    write_editor_body_padding(writer, &mut remaining)?;
    let search_ranges = view
        .search_highlight
        .map(|highlight| search_match_ranges(view.line, highlight.query, highlight.mode))
        .unwrap_or_default();
    if let Some(segments) = view.highlighted_line {
        let mut chunk_remaining = remaining.min(text_display_width(view.chunk));
        write_highlighted_line_window(
            writer,
            segments,
            view.chunk_start_column,
            &mut chunk_remaining,
            view.settings.syntax_theme_id,
            &search_ranges,
            frame,
        )
    } else {
        let mut display_column = 0;
        let mut source_column = 0;
        print_line_window_with_search(
            writer,
            LineWindowSearchView {
                text: view.chunk,
                start_column: 0,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: &search_ranges,
                base_fg: None,
                frame,
            },
        )
    }
}

fn highlighter_lines_for_wrapped_view(
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    body_width: usize,
) -> Vec<Option<Vec<(SyntectStyle, String)>>> {
    let visible_source_lines = wrapped_visible_source_line_count(document, view, body_width);
    highlighter.highlight_visible_lines(document, view.viewport_start, visible_source_lines)
}

fn wrapped_visible_source_line_count(
    document: &TextDocument,
    view: EditorView<'_>,
    body_width: usize,
) -> usize {
    let mut visual_rows = 0usize;
    let mut source_rows = 0usize;
    for line in document.buffer.lines().iter().skip(view.viewport_start) {
        if visual_rows >= view.visible_rows {
            break;
        }
        visual_rows += wrapped_line_chunks(line, body_width).len();
        source_rows += 1;
    }
    source_rows.max(1)
}

fn write_editor_body_padding(writer: &mut impl Write, remaining: &mut usize) -> io::Result<()> {
    for _ in 0..EDITOR_BODY_PADDING {
        if *remaining == 0 {
            break;
        }
        queue!(writer, Print(" "))?;
        *remaining = remaining.saturating_sub(1);
    }
    Ok(())
}

fn write_highlighted_line_window(
    writer: &mut impl Write,
    segments: Vec<(SyntectStyle, String)>,
    start_column: usize,
    remaining: &mut usize,
    syntax_theme_id: EditorThemeId,
    search_ranges: &[std::ops::Range<usize>],
    frame: RenderFrame,
) -> io::Result<()> {
    let mut skipped_columns = 0usize;
    let mut source_column = 0usize;
    for (style, segment) in segments {
        if *remaining == 0 {
            break;
        }
        let segment_columns = line_segment_display_width(&segment, skipped_columns);
        if skipped_columns + segment_columns <= start_column {
            skipped_columns += segment_columns;
            source_column += segment.chars().count();
            continue;
        }
        queue!(
            writer,
            SetForegroundColor(syntect_color_to_terminal(style.foreground, syntax_theme_id,)),
        )?;
        print_line_window_with_search(
            writer,
            LineWindowSearchView {
                text: &segment,
                start_column,
                display_column: &mut skipped_columns,
                source_column: &mut source_column,
                remaining_columns: remaining,
                search_ranges,
                base_fg: Some(syntect_color_to_terminal(style.foreground, syntax_theme_id)),
                frame,
            },
        )?;
    }
    queue!(writer, ResetColor)
}

struct StatusLineView<'a> {
    document: &'a TextDocument,
    cursor: Cursor,
    status: &'a str,
    settings: EditorSettings,
    screen_row: u16,
    horizontal_offset: usize,
}

fn write_status_line(
    writer: &mut impl Write,
    view: StatusLineView<'_>,
    frame: RenderFrame,
) -> io::Result<Option<u16>> {
    let line_numbers = if view.settings.show_line_numbers {
        "num:on"
    } else {
        "num:off"
    };
    let wrap = if view.settings.wrap_lines {
        "wrap:on"
    } else {
        "wrap:off"
    };
    let scroll = if view.settings.wrap_lines || view.horizontal_offset == 0 {
        "x:0".to_string()
    } else {
        format!("x:{}", view.horizontal_offset + 1)
    };
    let dirty = if view.document.buffer.is_dirty() {
        "modified"
    } else {
        "saved"
    };
    let right = format!(
        " Ln {}, Col {} | {} | {} | {} | {} | {} ",
        view.cursor.row + 1,
        view.cursor.column + 1,
        line_numbers,
        wrap,
        scroll,
        view.settings.theme_id.label(),
        dirty
    );
    let left = format!(" {} ", view.status);
    let mut remaining = frame.terminal_width;
    queue!(
        writer,
        frame.move_to(0, view.screen_row),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(frame.theme.status_fg),
        SetBackgroundColor(frame.theme.status_bg),
        SetAttribute(Attribute::Bold),
    )?;
    let rendered = if let Some(query) = view.status.strip_prefix("Search: ") {
        compose_prompt_status_line("Search: ", query, &right, frame.terminal_width)
    } else if let Some(query) = view.status.strip_prefix("Go to line: ") {
        compose_prompt_status_line("Go to line: ", query, &right, frame.terminal_width)
    } else {
        StatusLineRender {
            text: compose_status_line(&left, &right, frame.terminal_width),
            cursor_column: None,
        }
    };
    print_truncated(writer, &rendered.text, &mut remaining)?;
    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)?;
    Ok(rendered.cursor_column)
}

fn write_cursor_cell_highlight(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let character = document
        .buffer
        .lines()
        .get(view.cursor.row)
        .and_then(|line| line.chars().nth(view.cursor.column))
        .map(cursor_cell_character)
        .unwrap_or(' ');
    queue!(
        writer,
        frame.move_to(
            cursor_screen_column(document, view.cursor, view, frame),
            cursor_screen_row(document, view, frame)
        ),
        SetAttribute(Attribute::Reverse),
        Print(character),
        SetAttribute(Attribute::NoReverse),
        ResetColor,
    )
}

fn cursor_row_is_visible(
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> bool {
    if view.settings.wrap_lines {
        return cursor_visual_row_offset(document, view, frame)
            .is_some_and(|offset| offset < view.visible_rows);
    }

    let visible_end = view.viewport_start.saturating_add(view.visible_rows.max(1));
    view.cursor.row >= view.viewport_start && view.cursor.row < visible_end
}

fn write_help_line(writer: &mut impl Write, screen_row: u16, frame: RenderFrame) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    let help = compose_help_line(frame.terminal_width);
    queue!(
        writer,
        frame.move_to(0, screen_row),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(frame.theme.help_fg),
        SetBackgroundColor(frame.theme.help_bg),
    )?;
    print_truncated(writer, &help, &mut remaining)?;
    queue!(writer, ResetColor)
}

fn compose_help_line(width: usize) -> String {
    let controls = [
        "F2 Command",
        "F10 Menu/Help",
        "Ctrl-S Save",
        "Ctrl-B Files",
        "Ctrl-Q Quit",
    ];
    let width = width.max(1);
    let mut line = String::new();
    for control in controls {
        let candidate = if line.is_empty() {
            format!(" {control}")
        } else {
            format!("{line} | {control}")
        };
        if text_display_width(&candidate) > width {
            break;
        }
        line = candidate;
    }
    if text_display_width(&line) < width {
        line.push(' ');
    }
    line
}

fn line_number_width(document: &TextDocument) -> usize {
    document.buffer.line_count().to_string().len().max(2)
}

fn cursor_screen_column(
    document: &TextDocument,
    cursor: Cursor,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> u16 {
    let gutter_columns = if view.settings.show_line_numbers {
        frame.gutter_width + 1
    } else {
        0
    };
    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let body_display_column = document
        .buffer
        .lines()
        .get(cursor.row)
        .map(|line| line_display_width_until(line, cursor.column))
        .unwrap_or(0);
    let body_column = if view.settings.wrap_lines {
        body_display_column % body_width
    } else {
        body_display_column.saturating_sub(view.horizontal_offset)
    };
    let max_column = frame.terminal_width.saturating_sub(1);
    body_column
        .saturating_add(gutter_columns)
        .saturating_add(EDITOR_BODY_PADDING)
        .min(max_column) as u16
}

fn cursor_screen_row(document: &TextDocument, view: EditorView<'_>, frame: RenderFrame) -> u16 {
    if !view.settings.wrap_lines {
        return frame
            .body_top
            .saturating_add(view.cursor.row.saturating_sub(view.viewport_start) as u16);
    }

    let visual_offset = cursor_visual_row_offset(document, view, frame).unwrap_or(0);
    let max_row = frame
        .body_top
        .saturating_add(view.visible_rows.saturating_sub(1) as u16);
    frame
        .body_top
        .saturating_add(visual_offset as u16)
        .min(max_row)
}

fn cursor_visual_row_offset(
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> Option<usize> {
    if view.cursor.row < view.viewport_start {
        return None;
    }

    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let mut visual_offset = 0usize;
    for line in document
        .buffer
        .lines()
        .iter()
        .skip(view.viewport_start)
        .take(view.cursor.row.saturating_sub(view.viewport_start))
    {
        visual_offset =
            visual_offset.saturating_add(wrapped_line_chunks(line, body_width).len().max(1));
        if visual_offset >= view.visible_rows {
            return Some(visual_offset);
        }
    }

    let body_display_column = document
        .buffer
        .lines()
        .get(view.cursor.row)
        .map(|line| line_display_width_until(line, view.cursor.column))
        .unwrap_or(0);
    Some(visual_offset.saturating_add(body_display_column / body_width))
}

fn print_truncated(
    writer: &mut impl Write,
    text: &str,
    remaining_columns: &mut usize,
) -> io::Result<()> {
    if *remaining_columns == 0 {
        return Ok(());
    }

    let mut visible = String::new();
    let mut used_columns = 0;
    for character in text.chars() {
        let width = character_display_width(character, used_columns);
        if width > 0 && used_columns + width > *remaining_columns {
            break;
        }
        visible.push(character);
        used_columns += width;
    }
    *remaining_columns = remaining_columns.saturating_sub(used_columns);
    queue!(writer, Print(visible))
}

fn fit_text_end(text: &str, width: usize) -> String {
    let width = width.max(1);
    if text_display_width(text) <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }

    let mut suffix = String::new();
    let mut used_columns = 1;
    for character in text.chars().rev() {
        let character_width = character_display_width(character, used_columns);
        if character_width > 0 && used_columns + character_width > width {
            break;
        }
        suffix.insert(0, character);
        used_columns += character_width;
    }
    format!("…{suffix}")
}

fn compose_status_line(left: &str, right: &str, width: usize) -> String {
    let width = width.max(1);
    let right_count = text_display_width(right);
    if right_count >= width {
        return fit_text_start(right, width);
    }

    let left_width = width - right_count;
    let left = fit_text_start(left, left_width);
    let padding = width.saturating_sub(text_display_width(&left) + right_count);
    format!("{left}{}{right}", " ".repeat(padding))
}

struct StatusLineRender {
    text: String,
    cursor_column: Option<u16>,
}

fn compose_prompt_status_line(
    prompt: &str,
    query: &str,
    right: &str,
    width: usize,
) -> StatusLineRender {
    let width = width.max(1);
    let right_count = text_display_width(right);
    let left_width = if right_count < width {
        width - right_count
    } else {
        width
    };
    let prefix = format!(" {prompt}");
    let prefix_width = text_display_width(&prefix);

    if left_width <= prefix_width {
        let text = fit_text_start(&prefix, left_width);
        let cursor_column = text_display_width(&text).saturating_sub(1);
        return StatusLineRender {
            text,
            cursor_column: Some(cursor_column as u16),
        };
    }

    let query_width = left_width.saturating_sub(prefix_width + 1).max(1);
    let visible_query = fit_text_end(query, query_width);
    let left = format!("{prefix}{visible_query} ");
    let cursor_column = text_display_width(&left).saturating_sub(1);
    let text = if right_count < width {
        let padding = width.saturating_sub(text_display_width(&left) + right_count);
        format!("{left}{}{right}", " ".repeat(padding))
    } else {
        fit_text_start(&left, width)
    };

    StatusLineRender {
        text,
        cursor_column: Some(cursor_column.min(width.saturating_sub(1)) as u16),
    }
}

fn fit_text_start(text: &str, width: usize) -> String {
    let width = width.max(1);
    if text_display_width(text) <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }

    let mut prefix = String::new();
    let mut used_columns = 1;
    for character in text.chars() {
        let character_width = character_display_width(character, used_columns);
        if character_width > 0 && used_columns + character_width > width {
            break;
        }
        prefix.push(character);
        used_columns += character_width;
    }
    format!("{prefix}…")
}

struct LineWindowSearchView<'a> {
    text: &'a str,
    start_column: usize,
    display_column: &'a mut usize,
    source_column: &'a mut usize,
    remaining_columns: &'a mut usize,
    search_ranges: &'a [std::ops::Range<usize>],
    base_fg: Option<Color>,
    frame: RenderFrame,
}

fn print_line_window_with_search(
    writer: &mut impl Write,
    view: LineWindowSearchView<'_>,
) -> io::Result<()> {
    let mut search_paint_active = false;
    for character in view.text.chars() {
        if *view.remaining_columns == 0 {
            break;
        }

        let character_width = character_display_width(character, *view.display_column);
        let character_start = *view.display_column;
        let character_end = character_start + character_width;
        *view.display_column = character_end;
        let current_source_column = *view.source_column;
        *view.source_column += 1;

        if character_width > 0 && character_end <= view.start_column {
            continue;
        }
        if character_width > 0 && character_start < view.start_column {
            continue;
        }
        if character_width > *view.remaining_columns {
            break;
        }

        let in_match = view
            .search_ranges
            .iter()
            .any(|range| range.contains(&current_source_column));
        if in_match {
            queue!(
                writer,
                SetForegroundColor(view.frame.theme.search_fg),
                SetBackgroundColor(view.frame.theme.search_bg),
            )?;
            search_paint_active = true;
        } else if search_paint_active {
            queue!(writer, ResetColor)?;
            if let Some(base_fg) = view.base_fg {
                queue!(writer, SetForegroundColor(base_fg))?;
            }
            search_paint_active = false;
        }

        if character == '\t' {
            queue!(writer, Print(" ".repeat(character_width)))?;
        } else {
            queue!(writer, Print(character))?;
        }
        *view.remaining_columns = view.remaining_columns.saturating_sub(character_width);
    }
    if search_paint_active {
        queue!(writer, ResetColor)?;
        if let Some(base_fg) = view.base_fg {
            queue!(writer, SetForegroundColor(base_fg))?;
        }
    }
    Ok(())
}

fn search_match_ranges(text: &str, query: &str, mode: SearchMode) -> Vec<std::ops::Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    let query_columns = query.chars().count().max(1);
    let haystack = if mode.case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    };
    let needle = if mode.case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };
    let mut ranges = Vec::new();
    let mut search_byte = 0usize;
    while search_byte <= haystack.len() {
        let Some(relative_match) = haystack[search_byte..].find(&needle) else {
            break;
        };
        let match_byte = search_byte + relative_match;
        let start_column = haystack[..match_byte].chars().count();
        ranges.push(start_column..start_column + query_columns);
        search_byte = match_byte + needle.len().max(1);
    }
    ranges
}

fn text_display_width(text: &str) -> usize {
    let mut display_column = 0;
    for character in text.chars() {
        display_column += character_display_width(character, display_column);
    }
    display_column
}

fn line_segment_display_width(text: &str, start_column: usize) -> usize {
    let mut display_column = start_column;
    for character in text.chars() {
        display_column += character_display_width(character, display_column);
    }
    display_column - start_column
}

fn line_display_width_until(line: &str, character_column: usize) -> usize {
    let mut display_column = 0;
    for character in line.chars().take(character_column) {
        display_column += character_display_width(character, display_column);
    }
    display_column
}

fn char_column_for_display_column(line: &str, target_display_column: usize) -> usize {
    let mut display_column = 0;
    for (character_column, character) in line.chars().enumerate() {
        let width = character_display_width(character, display_column);
        if width > 0 && display_column + width > target_display_column {
            return character_column;
        }
        display_column += width;
    }
    line.chars().count()
}

fn character_display_width(character: char, display_column: usize) -> usize {
    if character == '\t' {
        let remainder = display_column % TAB_WIDTH;
        if remainder == 0 {
            TAB_WIDTH
        } else {
            TAB_WIDTH - remainder
        }
    } else {
        UnicodeWidthChar::width(character).unwrap_or(0)
    }
}

fn cursor_cell_character(character: char) -> char {
    if character == '\t' || character_display_width(character, 0) == 0 {
        ' '
    } else {
        character
    }
}

fn syntect_color_to_terminal(
    color: syntect::highlighting::Color,
    theme_id: EditorThemeId,
) -> Color {
    let (r, g, b) = terminal_syntax_role_rgb(
        theme_id,
        terminal_syntax_color_role(color.r, color.g, color.b),
    );
    Color::Rgb { r, g, b }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalSyntaxColorRole {
    Text,
    Comment,
    Rose,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}

fn terminal_syntax_color_role(red: u8, green: u8, blue: u8) -> TerminalSyntaxColorRole {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let chroma = max.saturating_sub(min);
    let luminance = 0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue);

    if chroma < 24 {
        return if luminance < 150.0 {
            TerminalSyntaxColorRole::Comment
        } else {
            TerminalSyntaxColorRole::Text
        };
    }

    let hue = terminal_rgb_hue_degrees(red, green, blue);
    if !(25.0..345.0).contains(&hue) {
        TerminalSyntaxColorRole::Rose
    } else if hue < 55.0 {
        TerminalSyntaxColorRole::Orange
    } else if hue < 78.0 {
        TerminalSyntaxColorRole::Yellow
    } else if hue < 160.0 {
        TerminalSyntaxColorRole::Green
    } else if hue < 200.0 {
        TerminalSyntaxColorRole::Cyan
    } else if hue < 255.0 {
        TerminalSyntaxColorRole::Blue
    } else if hue < 315.0 {
        TerminalSyntaxColorRole::Purple
    } else {
        TerminalSyntaxColorRole::Rose
    }
}

fn terminal_rgb_hue_degrees(red: u8, green: u8, blue: u8) -> f32 {
    let red = f32::from(red) / 255.0;
    let green = f32::from(green) / 255.0;
    let blue = f32::from(blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;

    if delta == 0.0 {
        return 0.0;
    }

    let hue = if max == red {
        60.0 * (((green - blue) / delta) % 6.0)
    } else if max == green {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    if hue < 0.0 {
        hue + 360.0
    } else {
        hue
    }
}

fn terminal_syntax_role_rgb(
    theme_id: EditorThemeId,
    role: TerminalSyntaxColorRole,
) -> (u8, u8, u8) {
    match theme_id {
        EditorThemeId::Nocturne => match role {
            TerminalSyntaxColorRole::Text => (213, 224, 246),
            TerminalSyntaxColorRole::Comment => (126, 141, 170),
            TerminalSyntaxColorRole::Rose => (255, 126, 154),
            TerminalSyntaxColorRole::Orange => (246, 177, 116),
            TerminalSyntaxColorRole::Yellow => (238, 213, 122),
            TerminalSyntaxColorRole::Green => (139, 222, 160),
            TerminalSyntaxColorRole::Cyan => (99, 221, 224),
            TerminalSyntaxColorRole::Blue => (132, 172, 255),
            TerminalSyntaxColorRole::Purple => (202, 158, 255),
        },
        EditorThemeId::Aurora => match role {
            TerminalSyntaxColorRole::Text => (218, 255, 241),
            TerminalSyntaxColorRole::Comment => (112, 162, 156),
            TerminalSyntaxColorRole::Rose => (255, 129, 162),
            TerminalSyntaxColorRole::Orange => (255, 183, 112),
            TerminalSyntaxColorRole::Yellow => (245, 224, 128),
            TerminalSyntaxColorRole::Green => (104, 241, 151),
            TerminalSyntaxColorRole::Cyan => (65, 234, 217),
            TerminalSyntaxColorRole::Blue => (119, 198, 255),
            TerminalSyntaxColorRole::Purple => (208, 151, 255),
        },
        EditorThemeId::Paper => match role {
            TerminalSyntaxColorRole::Text => (80, 67, 91),
            TerminalSyntaxColorRole::Comment => (119, 105, 130),
            TerminalSyntaxColorRole::Rose => (154, 62, 100),
            TerminalSyntaxColorRole::Orange => (158, 87, 48),
            TerminalSyntaxColorRole::Yellow => (125, 94, 20),
            TerminalSyntaxColorRole::Green => (45, 116, 93),
            TerminalSyntaxColorRole::Cyan => (37, 111, 126),
            TerminalSyntaxColorRole::Blue => (67, 89, 153),
            TerminalSyntaxColorRole::Purple => (118, 72, 156),
        },
        EditorThemeId::Terminal => match role {
            TerminalSyntaxColorRole::Text => (168, 255, 176),
            TerminalSyntaxColorRole::Comment => (83, 165, 95),
            TerminalSyntaxColorRole::Rose => (255, 126, 126),
            TerminalSyntaxColorRole::Orange => (247, 186, 96),
            TerminalSyntaxColorRole::Yellow => (240, 250, 127),
            TerminalSyntaxColorRole::Green => (80, 255, 119),
            TerminalSyntaxColorRole::Cyan => (113, 255, 207),
            TerminalSyntaxColorRole::Blue => (142, 215, 255),
            TerminalSyntaxColorRole::Purple => (205, 168, 255),
        },
        EditorThemeId::Abyss => match role {
            TerminalSyntaxColorRole::Text => (214, 244, 255),
            TerminalSyntaxColorRole::Comment => (100, 132, 158),
            TerminalSyntaxColorRole::Rose => (255, 97, 137),
            TerminalSyntaxColorRole::Orange => (255, 169, 111),
            TerminalSyntaxColorRole::Yellow => (241, 218, 111),
            TerminalSyntaxColorRole::Green => (111, 230, 172),
            TerminalSyntaxColorRole::Cyan => (93, 239, 255),
            TerminalSyntaxColorRole::Blue => (126, 174, 255),
            TerminalSyntaxColorRole::Purple => (196, 145, 255),
        },
        EditorThemeId::Terror => match role {
            TerminalSyntaxColorRole::Text => (255, 193, 238),
            TerminalSyntaxColorRole::Comment => (157, 103, 148),
            TerminalSyntaxColorRole::Rose => (255, 62, 166),
            TerminalSyntaxColorRole::Orange => (255, 120, 75),
            TerminalSyntaxColorRole::Yellow => (255, 226, 82),
            TerminalSyntaxColorRole::Green => (91, 255, 141),
            TerminalSyntaxColorRole::Cyan => (90, 238, 230),
            TerminalSyntaxColorRole::Blue => (136, 172, 255),
            TerminalSyntaxColorRole::Purple => (221, 97, 255),
        },
    }
}

fn visible_editor_rows(extra_reserved_rows: usize) -> usize {
    size()
        .map(|(_, rows)| rows.saturating_sub(3 + extra_reserved_rows as u16).max(1) as usize)
        .unwrap_or(20)
}

fn terminal_width() -> usize {
    size()
        .map(|(columns, _)| columns.max(1) as usize)
        .unwrap_or(80)
}

fn visible_text_columns(
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> usize {
    let gutter_columns = if settings.show_line_numbers {
        gutter_width + 1
    } else {
        0
    };
    terminal_width
        .saturating_sub(gutter_columns)
        .saturating_sub(EDITOR_BODY_PADDING)
        .max(1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WrappedLineChunk {
    start_column: usize,
    text: String,
}

fn wrapped_line_chunks(line: &str, width: usize) -> Vec<WrappedLineChunk> {
    let width = width.max(1);
    if line.is_empty() {
        return vec![WrappedLineChunk {
            start_column: 0,
            text: String::new(),
        }];
    }

    let mut chunks = Vec::new();
    let mut remaining_start = 0usize;
    let mut first_chunk = true;

    while remaining_start < line.len() {
        let mut remaining = &line[remaining_start..];
        if !first_chunk {
            let trimmed = remaining.trim_start_matches(char::is_whitespace);
            remaining_start += remaining.len().saturating_sub(trimmed.len());
            remaining = trimmed;
            if remaining.is_empty() {
                break;
            }
        }
        let start_column = text_display_width(&line[..remaining_start]);
        let (chunk, rest) = take_wrapped_line_chunk(remaining, width);
        remaining_start = line.len().saturating_sub(rest.len());
        chunks.push(WrappedLineChunk {
            start_column,
            text: chunk,
        });
        first_chunk = false;
    }

    chunks
}

fn take_wrapped_line_chunk(line: &str, width: usize) -> (String, &str) {
    let mut display_column = 0;
    let mut last_break_byte = None;

    for (byte_index, character) in line.char_indices() {
        let character_width = character_display_width(character, display_column);
        if character_width > 0 && display_column + character_width > width {
            if character.is_whitespace() {
                let chunk = line[..byte_index].trim_end_matches(char::is_whitespace);
                return (
                    chunk.to_string(),
                    &line[byte_index + character.len_utf8()..],
                );
            }
            if let Some(break_byte) = last_break_byte {
                let chunk = line[..break_byte].trim_end_matches(char::is_whitespace);
                return (chunk.to_string(), &line[break_byte..]);
            }
            if byte_index == 0 {
                let next_byte = byte_index + character.len_utf8();
                return (line[..next_byte].to_string(), &line[next_byte..]);
            }
            return (line[..byte_index].to_string(), &line[byte_index..]);
        }

        display_column += character_width;
        if character.is_whitespace() {
            last_break_byte = Some(byte_index + character.len_utf8());
        }
    }

    (line.to_string(), "")
}

fn clamp_horizontal_viewport(
    document: &TextDocument,
    cursor: Cursor,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
    horizontal_offset: usize,
) -> usize {
    let visible_columns = visible_text_columns(settings, gutter_width, terminal_width);
    let cursor_display_column = document
        .buffer
        .lines()
        .get(cursor.row)
        .map(|line| line_display_width_until(line, cursor.column))
        .unwrap_or(0);
    if cursor_display_column < horizontal_offset {
        cursor_display_column
    } else if cursor_display_column >= horizontal_offset + visible_columns {
        cursor_display_column + 1 - visible_columns
    } else {
        horizontal_offset
    }
}

fn clamp_viewport(
    document: &TextDocument,
    cursor: Cursor,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> usize {
    let visible_rows = visible_rows.max(1);
    let max_start = max_viewport_start(document, visible_rows, settings);
    let mut start = viewport_start.min(max_start);

    if cursor.row < start {
        start = cursor.row;
    } else if cursor.row >= start + visible_rows {
        start = cursor.row + 1 - visible_rows;
    }

    start = start.min(max_start);
    if settings.wrap_lines {
        while start < max_start
            && !cursor_is_visible_from_viewport(
                document,
                cursor,
                start,
                visible_rows,
                settings,
                gutter_width,
                terminal_width,
            )
        {
            start += 1;
        }
    }

    start
}

fn clamp_passive_viewport(
    document: &TextDocument,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
) -> usize {
    let visible_rows = visible_rows.max(1);
    let max_start = max_viewport_start(document, visible_rows, settings);
    viewport_start.min(max_start)
}

fn max_viewport_start(
    document: &TextDocument,
    visible_rows: usize,
    settings: EditorSettings,
) -> usize {
    if settings.wrap_lines {
        document.buffer.line_count().saturating_sub(1)
    } else {
        document
            .buffer
            .line_count()
            .saturating_sub(visible_rows.max(1))
    }
}

fn cursor_is_visible_from_viewport(
    document: &TextDocument,
    cursor: Cursor,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> bool {
    let frame = RenderFrame {
        theme: EditorTheme::for_id(settings.theme_id),
        gutter_width,
        terminal_width,
        origin_column: 0,
        body_top: 0,
    };
    let view = EditorView {
        cursor,
        viewport_start,
        horizontal_offset: 0,
        visible_rows,
        status: "",
        settings,
        menu: None,
        sidebar_width: 0,
        tab_strip: &[],
        search_highlight: None,
    };
    cursor_row_is_visible(document, view, frame)
}

trait TerminalBackend {
    type Writer: Write;

    fn enter() -> io::Result<(Self::Writer, Self)>
    where
        Self: Sized;
    fn restore(&mut self);
}

struct CrosstermBackend {
    keyboard_enhancement_active: bool,
}

fn editor_keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
}

impl TerminalBackend for CrosstermBackend {
    type Writer = io::Stdout;

    fn enter() -> io::Result<(Self::Writer, Self)> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();

        let keyboard_enhancement_active = supports_keyboard_enhancement().unwrap_or(false);
        if keyboard_enhancement_active {
            if let Err(error) = execute!(
                stdout,
                PushKeyboardEnhancementFlags(editor_keyboard_enhancement_flags())
            ) {
                let _ = disable_raw_mode();
                return Err(error);
            }
        }

        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Show) {
            if keyboard_enhancement_active {
                let _ = execute!(stdout, PopKeyboardEnhancementFlags);
            }
            let _ = execute!(stdout, DisableMouseCapture, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok((
            stdout,
            Self {
                keyboard_enhancement_active,
            },
        ))
    }

    fn restore(&mut self) {
        let mut stdout = io::stdout();
        if self.keyboard_enhancement_active {
            let _ = execute!(
                stdout,
                Show,
                DisableMouseCapture,
                PopKeyboardEnhancementFlags,
                LeaveAlternateScreen
            );
        } else {
            let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
        }
        let _ = disable_raw_mode();
    }
}

struct TerminalSession<B: TerminalBackend = CrosstermBackend> {
    stdout: B::Writer,
    backend: B,
}

impl TerminalSession<CrosstermBackend> {
    fn enter() -> io::Result<Self> {
        let (stdout, backend) = CrosstermBackend::enter()?;
        Ok(Self { stdout, backend })
    }
}

impl<B: TerminalBackend> Drop for TerminalSession<B> {
    fn drop(&mut self) {
        self.backend.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kfnotepad::{EditorTab, EditorTabDocument, EditorTabState};
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;

    struct TempArea {
        root: PathBuf,
    }

    impl TempArea {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("kfnotepad-main-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).expect("create temp test directory");
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
        let entries = fs::read_dir(directory)
            .expect("read directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect directory entries");
        assert!(
            entries
                .iter()
                .all(|entry| !entry.file_name().to_string_lossy().contains(".kfnotepad-")),
            "temporary file left in {}",
            directory.display()
        );
    }

    #[test]
    fn editor_config_path_prefers_xdg_config_home() {
        let temp = TempArea::new("config-path-xdg");
        let xdg = temp.path("xdg-config");
        let home = temp.path("home");

        let path = editor_config_path(Some(xdg.as_path()), Some(home.as_path()))
            .expect("resolve config path");

        assert_eq!(path, xdg.join("kfnotepad").join("config.toml"));
    }

    #[test]
    fn editor_config_path_falls_back_to_home_config() {
        let temp = TempArea::new("config-path-home");
        let home = temp.path("home");

        let path = editor_config_path(None, Some(home.as_path())).expect("resolve config path");

        assert_eq!(
            path,
            home.join(".config").join("kfnotepad").join("config.toml")
        );
    }

    #[test]
    fn editor_config_path_requires_a_base_directory() {
        assert!(editor_config_path(None, None).is_none());
    }

    #[test]
    fn load_editor_settings_uses_defaults_for_missing_config() {
        let temp = TempArea::new("config-missing");
        let path = temp.path("missing").join("config.toml");

        let settings = load_editor_settings(&path).expect("missing config should use defaults");

        assert_eq!(settings, EditorSettings::default());
    }

    #[test]
    fn parse_editor_settings_config_reads_known_keys_and_ignores_bad_values() {
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
gui_restore_last_workspace = "true"
gui_font_family = "papyrus"
gui_font_size = 500
gui_ui_font_size = 500
"#,
        );

        assert_eq!(fallback, EditorSettings::default());
    }

    #[test]
    fn save_editor_settings_writes_atomic_private_config() {
        let temp = TempArea::new("config-save");
        let path = temp.path("xdg").join("kfnotepad").join("config.toml");
        let settings = EditorSettings {
            show_line_numbers: false,
            theme_id: EditorThemeId::Abyss,
            wrap_lines: true,
            gui_restore_last_workspace: true,
            gui_font_family: GuiFontFamily::JetBrainsMono,
            gui_font_size: 18,
            gui_ui_font_size: 15,
            ..EditorSettings::default()
        };

        save_editor_settings(&path, settings).expect("save editor config");

        assert_eq!(
            fs::read_to_string(&path).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"jetbrains-mono\"\ngui_font_size = 18\ngui_ui_font_size = 15\n"
        );
        assert_no_temp_files(path.parent().expect("config parent"));

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
    fn preference_controls_persist_runtime_settings_when_configured() {
        let temp = TempArea::new("config-runtime");
        let config_path = temp.path("config").join("kfnotepad").join("config.toml");
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            config_path: Some(config_path.clone()),
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)
        ));
        assert_eq!(runtime.settings.theme_id, EditorThemeId::Aurora);
        assert_eq!(runtime.status, "Theme: aurora");
        assert_eq!(
            fs::read_to_string(&config_path).expect("read config"),
            "theme = \"aurora\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
        ));
        assert!(!runtime.settings.show_line_numbers);
        assert!(fs::read_to_string(&config_path)
            .expect("read updated config")
            .contains("line_numbers = false"));

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
        ));
        assert!(runtime.settings.wrap_lines);
        assert!(fs::read_to_string(&config_path)
            .expect("read updated config")
            .contains("wrap = true"));
    }

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

    #[test]
    fn ctrl_q_works_while_sidebar_is_open() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState {
                current_dir: PathBuf::from("."),
                entries: Vec::new(),
                selected: 0,
                scroll: 0,
            }),
            ..EditorRuntime::default()
        };

        assert!(handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)
        ));
    }

    #[test]
    fn dirty_ctrl_q_confirmation_works_while_sidebar_is_open() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        document
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty document");
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState {
                current_dir: PathBuf::from("."),
                entries: Vec::new(),
                selected: 0,
                scroll: 0,
            }),
            ..EditorRuntime::default()
        };
        let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            quit
        ));
        assert!(runtime.quit_confirmation_pending);
        assert!(runtime.status.contains("Unsaved changes"));
        assert!(handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            quit
        ));
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
    fn editor_tab_state_keeps_cursor_and_viewport_independent() {
        let first = EditorTabState {
            cursor: Cursor { row: 3, column: 7 },
            viewport_start: 2,
            horizontal_offset: 5,
        };
        let second = EditorTabState::default();

        assert_eq!(first.cursor, Cursor { row: 3, column: 7 });
        assert_eq!(first.viewport_start, 2);
        assert_eq!(first.horizontal_offset, 5);
        assert_eq!(second.cursor, Cursor { row: 0, column: 0 });
        assert_eq!(second.viewport_start, 0);
        assert_eq!(second.horizontal_offset, 0);
    }

    #[test]
    fn editor_workspace_starts_with_one_active_tab() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let workspace = EditorWorkspace::from_document(&mut document);

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active, 0);
        assert_eq!(
            workspace.active_tab().document.as_ref().path,
            PathBuf::from("note.txt")
        );
        assert_eq!(workspace.active_tab().state, EditorTabState::default());
    }

    #[test]
    fn editor_workspace_active_tab_mutates_original_document() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
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
                .expect("insert into active tab");

            assert_eq!(active_tab.state.cursor, Cursor { row: 0, column: 5 });
            assert!(active_tab.document.as_ref().buffer.is_dirty());
        }

        assert_eq!(document.buffer.to_text(), "alpha!\n");
        assert!(document.buffer.is_dirty());
    }

    #[test]
    fn workspace_tab_switch_reports_single_tab_without_moving() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        select_next_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.active, 0);
        assert_eq!(runtime.status, "Only one tab open");
        assert!(!runtime.quit_confirmation_pending);

        select_previous_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.active, 0);
        assert_eq!(runtime.status, "Only one tab open");
    }

    #[test]
    fn workspace_tab_switch_cycles_between_tabs() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState {
                        cursor: Cursor { row: 0, column: 1 },
                        viewport_start: 0,
                        horizontal_offset: 0,
                    },
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState {
                        cursor: Cursor { row: 0, column: 2 },
                        viewport_start: 0,
                        horizontal_offset: 0,
                    },
                },
            ],
            active: 0,
        };
        let mut runtime = EditorRuntime::default();

        select_next_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.active, 1);
        assert_eq!(
            workspace.active_tab().state.cursor,
            Cursor { row: 0, column: 2 }
        );
        assert_eq!(runtime.status, "Tab 2/2: second.txt");

        select_next_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.active, 0);
        assert_eq!(
            workspace.active_tab().state.cursor,
            Cursor { row: 0, column: 1 }
        );
        assert_eq!(runtime.status, "Tab 1/2: first.txt");

        select_previous_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Tab 2/2: second.txt");
    }

    #[test]
    fn workspace_tab_keybindings_switch_only_when_editor_body_is_active() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState::default(),
                },
            ],
            active: 0,
        };
        let next_tab = KeyEvent::new(KeyCode::PageDown, KeyModifiers::CONTROL);
        let mut runtime = EditorRuntime {
            menu: Some(MenuState::default()),
            ..EditorRuntime::default()
        };

        assert!(!handle_workspace_key_event(
            &mut workspace,
            &mut runtime,
            next_tab
        ));
        assert_eq!(workspace.active, 0);

        runtime.menu = None;
        assert!(handle_workspace_key_event(
            &mut workspace,
            &mut runtime,
            next_tab
        ));
        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Tab 2/2: second.txt");
    }

    #[test]
    fn workspace_close_tab_refuses_only_tab() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            close_tab_confirmation_pending: true,
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        close_active_tab(&mut workspace, &mut runtime);

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active, 0);
        assert_eq!(runtime.status, "Cannot close the only tab");
        assert!(!runtime.close_tab_confirmation_pending);
        assert!(!runtime.quit_confirmation_pending);
    }

    #[test]
    fn workspace_close_tab_removes_clean_active_tab_and_clamps_selection() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState {
                        cursor: Cursor { row: 0, column: 2 },
                        viewport_start: 0,
                        horizontal_offset: 0,
                    },
                },
            ],
            active: 1,
        };
        let mut runtime = EditorRuntime::default();

        close_active_tab(&mut workspace, &mut runtime);

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active, 0);
        assert_eq!(
            workspace.active_tab().document.as_ref().path,
            PathBuf::from("first.txt")
        );
        assert_eq!(runtime.status, "Closed tab: second.txt");
        assert!(!runtime.close_tab_confirmation_pending);
    }

    #[test]
    fn workspace_close_dirty_tab_requires_confirmation() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        second
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty second tab");
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState::default(),
                },
            ],
            active: 1,
        };
        let mut runtime = EditorRuntime::default();

        close_active_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert!(runtime.close_tab_confirmation_pending);
        assert_eq!(
            runtime.status,
            "Unsaved changes. Press Ctrl-F4 again to close tab."
        );

        close_active_tab(&mut workspace, &mut runtime);
        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active, 0);
        assert!(!runtime.close_tab_confirmation_pending);
        assert_eq!(runtime.status, "Closed tab: second.txt");
    }

    #[test]
    fn workspace_close_tab_keybinding_works_only_when_editor_body_is_active() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState::default(),
                },
            ],
            active: 1,
        };
        let close_tab = KeyEvent::new(KeyCode::F(4), KeyModifiers::CONTROL);
        let mut runtime = EditorRuntime {
            search_active: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_workspace_key_event(
            &mut workspace,
            &mut runtime,
            close_tab
        ));
        assert_eq!(workspace.tabs.len(), 2);

        runtime.search_active = false;
        assert!(handle_workspace_key_event(
            &mut workspace,
            &mut runtime,
            close_tab
        ));
        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(runtime.status, "Closed tab: second.txt");
    }

    #[test]
    fn f10_tabs_menu_switches_and_closes_tabs() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Borrowed(&mut second),
                    state: EditorTabState::default(),
                },
            ],
            active: 0,
        };
        let mut runtime = EditorRuntime {
            menu: Some(MenuState {
                group: MenuGroup::Tabs,
                selected: 1,
            }),
            ..EditorRuntime::default()
        };

        assert!(!handle_workspace_menu_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
        ));
        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Tab 2/2: second.txt");

        runtime.menu = Some(MenuState {
            group: MenuGroup::Tabs,
            selected: 2,
        });
        assert!(!handle_workspace_menu_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
        ));
        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(runtime.status, "Closed tab: second.txt");
    }

    #[test]
    fn file_sidebar_lists_parent_dirs_and_files_in_order() {
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
    }

    #[test]
    fn file_sidebar_navigates_into_subdirectories_and_parent() {
        let temp = TempArea::new("sidebar-nav");
        fs::create_dir(temp.path("sub")).expect("create sub dir");
        fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write sub file");
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
        assert_eq!(
            runtime.sidebar.as_ref().expect("sidebar").current_dir,
            temp.path("sub")
                .canonicalize()
                .expect("canonicalize subdirectory")
        );

        runtime.sidebar.as_mut().expect("sidebar").selected = 0;
        activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
        assert_eq!(
            runtime.sidebar.as_ref().expect("sidebar").current_dir,
            temp.root.canonicalize().expect("canonicalize root")
        );
    }

    #[test]
    fn file_sidebar_reopens_in_last_visited_directory() {
        let temp = TempArea::new("sidebar-last-dir");
        fs::create_dir(temp.path("sub")).expect("create sub");
        fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write inside");
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        let sub_dir = temp.path("sub").canonicalize().expect("canonicalize sub");
        assert_eq!(
            runtime.sidebar.as_ref().expect("sidebar").current_dir,
            sub_dir
        );

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert_eq!(runtime.sidebar, None);
        assert_eq!(runtime.last_sidebar_dir, Some(sub_dir.clone()));

        toggle_file_sidebar(&mut runtime);
        assert_eq!(
            runtime.sidebar.as_ref().expect("sidebar").current_dir,
            sub_dir
        );
    }

    #[test]
    fn file_sidebar_opens_clean_file_and_blocks_dirty_switch() {
        let temp = TempArea::new("sidebar-open");
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut cursor = Cursor { row: 0, column: 4 };
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
        assert_eq!(document.path, next_path);
        assert_eq!(document.buffer.lines(), &["next".to_string()]);
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(runtime.sidebar, None);

        let mut dirty_document = TextDocument {
            path: PathBuf::from("dirty.txt"),
            buffer: kfnotepad::TextBuffer::from_text("dirty\n"),
        };
        dirty_document
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty document");
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;
        activate_selected_sidebar_entry(&mut dirty_document, &mut cursor, &mut runtime);

        assert_eq!(dirty_document.path, PathBuf::from("dirty.txt"));
        assert_eq!(runtime.status, "Save before opening another file");
        assert!(runtime.sidebar.is_some());
    }

    #[test]
    fn file_sidebar_opens_selected_file_in_new_tab_without_replacing_dirty_current() {
        let temp = TempArea::new("sidebar-open-tab");
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        document
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty current document");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            quit_confirmation_pending: true,
            close_tab_confirmation_pending: true,
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        open_selected_sidebar_entry_in_new_tab(&mut workspace, &mut runtime);

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(
            workspace.tabs[0].document.as_ref().path,
            PathBuf::from("current.txt")
        );
        assert!(workspace.tabs[0].document.as_ref().buffer.is_dirty());
        assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
        assert_eq!(
            workspace.active_tab().document.as_ref().buffer.lines(),
            &["next".to_string()]
        );
        assert_eq!(workspace.active_tab().state, EditorTabState::default());
        assert_eq!(runtime.sidebar, None);
        assert_eq!(runtime.status, "Opened tab next.txt");
        assert!(!runtime.quit_confirmation_pending);
        assert!(!runtime.close_tab_confirmation_pending);
    }

    #[test]
    fn sidebar_ctrl_enter_opens_selected_file_in_new_tab() {
        let temp = TempArea::new("sidebar-ctrl-enter-tab");
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        assert!(handle_workspace_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
        ));

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
        assert_eq!(runtime.status, "Opened tab next.txt");
    }

    #[test]
    fn workspace_sidebar_enter_opens_selected_file_as_visible_tab() {
        let temp = TempArea::new("sidebar-enter-tab");
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        document
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty current document");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(
            workspace.tabs[0].document.as_ref().path,
            PathBuf::from("current.txt")
        );
        assert!(workspace.tabs[0].document.as_ref().buffer.is_dirty());
        assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
        assert_eq!(runtime.sidebar, None);
        assert_eq!(runtime.status, "Opened tab next.txt");
    }

    #[test]
    fn workspace_sidebar_open_focuses_existing_file_tab_without_duplicate() {
        let temp = TempArea::new("sidebar-focus-existing-tab");
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let current = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let next = open_text_file(&next_path).expect("open next");
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Owned(current),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Owned(next),
                    state: EditorTabState::default(),
                },
            ],
            active: 0,
        };
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Focused tab next.txt");
    }

    #[test]
    fn help_menu_opens_and_focuses_maintained_help_document() {
        let mut document = TextDocument {
            path: PathBuf::from("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime::default();

        run_workspace_menu_command(MenuCommand::OpenHelp, &mut workspace, &mut runtime);

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        let active = workspace.active_tab().document.as_ref();
        assert_eq!(active.path, PathBuf::from(TUI_HELP_DOCUMENT_PATH));
        assert!(active.buffer.to_text().contains("# kfnotepad help"));
        assert!(active.buffer.to_text().contains("## File sidebar"));
        assert!(active
            .buffer
            .to_text()
            .contains("Ctrl-R toggles reader mode."));
        assert!(active
            .buffer
            .to_text()
            .contains("Ctrl-Shift-T cycles the syntax highlighting theme independently."));
        assert!(active
            .buffer
            .to_text()
            .contains("Managed notes are normal Markdown files."));
        assert!(!active.buffer.is_dirty());
        assert_eq!(runtime.status, "Opened help");

        workspace.active = 0;
        run_workspace_menu_command(MenuCommand::OpenHelp, &mut workspace, &mut runtime);

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Focused help");
    }

    #[test]
    fn tui_workspace_project_save_current_writes_path_only_project() {
        let temp = TempArea::new("tui-workspace-save-current");
        let first_path = temp.path("first.txt");
        let second_path = temp.path("second.txt");
        fs::write(&first_path, "first\n").expect("write first");
        fs::write(&second_path, "second\n").expect("write second");
        let first = open_text_file(&first_path).expect("open first");
        let second = open_text_file(&second_path).expect("open second");
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Owned(first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Owned(second),
                    state: EditorTabState::default(),
                },
            ],
            active: 1,
        };
        let projects_dir = temp.path("workspaces");
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir.clone()),
            ..EditorRuntime::default()
        };

        run_workspace_menu_command(
            MenuCommand::SaveCurrentWorkspace,
            &mut workspace,
            &mut runtime,
        );

        let project_path = gui_workspace_project_path(&projects_dir, "current workspace")
            .expect("current project path");
        let project = parse_gui_workspace_project(
            &fs::read_to_string(project_path).expect("read saved project"),
        )
        .expect("parse project");
        assert_eq!(project.name, "current workspace");
        assert_eq!(project.files, vec![first_path, second_path]);
        assert_eq!(project.active_ordinal, 1);
        assert_eq!(project.layout, None);
        assert_eq!(runtime.status, "Workspace saved: current workspace");
    }

    #[test]
    fn tui_workspace_project_save_named_prompt_and_list_projects() {
        let temp = TempArea::new("tui-workspace-save-named");
        let file_path = temp.path("note.txt");
        fs::write(&file_path, "note\n").expect("write note");
        let mut document = open_text_file(&file_path).expect("open note");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let projects_dir = temp.path("workspaces");
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir.clone()),
            ..EditorRuntime::default()
        };

        start_workspace_save_prompt(&mut runtime);
        for character in "Project One".chars() {
            handle_workspace_prompt_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let project_path =
            gui_workspace_project_path(&projects_dir, "Project One").expect("named project path");
        assert!(project_path.exists());
        assert_eq!(runtime.status, "Workspace saved: Project One");

        open_workspace_manager(&mut runtime);
        let manager = runtime
            .workspace_manager
            .as_ref()
            .expect("workspace manager");
        assert_eq!(manager.entries.len(), 1);
        assert_eq!(manager.entries[0].name, "Project One");
        assert_eq!(
            runtime.status,
            "Workspace manager: Enter open | S save over | D delete | N new | Esc"
        );
    }

    #[test]
    fn tui_new_file_command_creates_clean_untitled_tab_without_writing_file() {
        let temp = TempArea::new("tui-new-file-tab");
        let existing_untitled = temp.path("untitled.txt");
        fs::write(&existing_untitled, "already exists\n").expect("write existing untitled");
        let current_path = temp.path("current.txt");
        fs::write(&current_path, "current\n").expect("write current");
        let mut document = open_text_file(&current_path).expect("open current");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };

        run_workspace_menu_command(MenuCommand::NewFile, &mut workspace, &mut runtime);

        let expected = temp.path("untitled-2.txt");
        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(workspace.active_tab().document.as_ref().path, expected);
        assert_eq!(
            workspace.active_tab().document.as_ref().buffer.to_text(),
            ""
        );
        assert!(!workspace.active_tab().document.as_ref().buffer.is_dirty());
        assert!(!expected.exists());
        assert_eq!(runtime.status, "New file tab: untitled-2.txt");
    }

    #[test]
    fn tui_workspace_prompt_cycles_existing_names_for_save_overwrite() {
        let temp = TempArea::new("tui-workspace-save-cycle");
        let old_path = temp.path("old.txt");
        let new_path = temp.path("new.txt");
        fs::write(&old_path, "old\n").expect("write old");
        fs::write(&new_path, "new\n").expect("write new");
        let projects_dir = temp.path("workspaces");
        save_gui_workspace_project(
            &gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
            &GuiWorkspaceProject {
                name: "Alpha".to_string(),
                files: vec![old_path.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save alpha");
        save_gui_workspace_project(
            &gui_workspace_project_path(&projects_dir, "Beta").expect("beta path"),
            &GuiWorkspaceProject {
                name: "Beta".to_string(),
                files: vec![old_path],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save beta");
        let mut document = open_text_file(&new_path).expect("open new");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir.clone()),
            ..EditorRuntime::default()
        };

        start_workspace_save_prompt(&mut runtime);
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert_eq!(runtime.workspace_query, "Beta");
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let project = parse_gui_workspace_project(
            &fs::read_to_string(
                gui_workspace_project_path(&projects_dir, "Beta").expect("beta path"),
            )
            .expect("read beta"),
        )
        .expect("parse beta");
        assert_eq!(project.files, vec![new_path]);
        assert_eq!(runtime.status, "Workspace saved: Beta");
    }

    #[test]
    fn tui_workspace_open_prompt_cycles_saved_projects() {
        let temp = TempArea::new("tui-workspace-open-cycle");
        let alpha_path = temp.path("alpha.txt");
        let beta_path = temp.path("beta.txt");
        let original_path = temp.path("original.txt");
        fs::write(&alpha_path, "alpha\n").expect("write alpha");
        fs::write(&beta_path, "beta\n").expect("write beta");
        fs::write(&original_path, "original\n").expect("write original");
        let projects_dir = temp.path("workspaces");
        for (name, path) in [("Alpha", alpha_path.clone()), ("Beta", beta_path.clone())] {
            save_gui_workspace_project(
                &gui_workspace_project_path(&projects_dir, name).expect("project path"),
                &GuiWorkspaceProject {
                    name: name.to_string(),
                    files: vec![path],
                    active_ordinal: 0,
                    layout: None,
                },
            )
            .expect("save project");
        }
        let mut original = open_text_file(&original_path).expect("open original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        start_workspace_open_prompt(&mut runtime);
        assert_eq!(runtime.workspace_query, "Alpha");
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert_eq!(runtime.workspace_query, "Beta");
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active_tab().document.as_ref().path, beta_path);
        assert_eq!(runtime.status, "Opened workspace: Beta");
    }

    #[test]
    fn tui_workspace_delete_prompt_requires_confirmation_and_removes_project() {
        let temp = TempArea::new("tui-workspace-delete");
        let file_path = temp.path("note.txt");
        fs::write(&file_path, "note\n").expect("write note");
        let projects_dir = temp.path("workspaces");
        let project_path = gui_workspace_project_path(&projects_dir, "Project").expect("path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "Project".to_string(),
                files: vec![file_path.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");
        let mut document = open_text_file(&file_path).expect("open note");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        start_workspace_delete_prompt(&mut runtime);
        assert_eq!(runtime.workspace_query, "Project");
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert_eq!(
            runtime.workspace_prompt,
            Some(WorkspacePrompt::ConfirmDelete)
        );
        assert!(project_path.exists());
        for character in "yes".chars() {
            handle_workspace_prompt_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(!project_path.exists());
        assert_eq!(runtime.workspace_prompt, None);
        assert_eq!(runtime.status, "Deleted workspace: Project");
    }

    #[test]
    fn tui_workspace_manager_opens_for_many_projects() {
        let temp = TempArea::new("tui-workspace-manager-many");
        let file_path = temp.path("note.txt");
        fs::write(&file_path, "note\n").expect("write note");
        let projects_dir = temp.path("workspaces");
        for name in ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"] {
            let path = gui_workspace_project_path(&projects_dir, name).expect("project path");
            save_gui_workspace_project(
                &path,
                &GuiWorkspaceProject {
                    name: name.to_string(),
                    files: vec![file_path.clone()],
                    active_ordinal: 0,
                    layout: None,
                },
            )
            .expect("save project");
        }
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_manager(&mut runtime);

        let manager = runtime
            .workspace_manager
            .as_ref()
            .expect("workspace manager");
        assert_eq!(manager.entries.len(), 5);
        assert_eq!(manager.entries[0].name, "Alpha");
        assert!(runtime.status.contains("Workspace manager"));
    }

    #[test]
    fn tui_workspace_manager_enter_opens_selected_project() {
        let temp = TempArea::new("tui-workspace-manager-open");
        let alpha_path = temp.path("alpha.txt");
        let beta_path = temp.path("beta.txt");
        let original_path = temp.path("original.txt");
        fs::write(&alpha_path, "alpha\n").expect("write alpha");
        fs::write(&beta_path, "beta\n").expect("write beta");
        fs::write(&original_path, "original\n").expect("write original");
        let projects_dir = temp.path("workspaces");
        for (name, path) in [("Alpha", alpha_path), ("Beta", beta_path.clone())] {
            save_gui_workspace_project(
                &gui_workspace_project_path(&projects_dir, name).expect("project path"),
                &GuiWorkspaceProject {
                    name: name.to_string(),
                    files: vec![path],
                    active_ordinal: 0,
                    layout: None,
                },
            )
            .expect("save project");
        }
        let mut original = open_text_file(&original_path).expect("open original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_manager(&mut runtime);
        handle_workspace_manager_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        handle_workspace_manager_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(runtime.workspace_manager, None);
        assert_eq!(workspace.active_tab().document.as_ref().path, beta_path);
        assert_eq!(runtime.status, "Opened workspace: Beta");
    }

    #[test]
    fn tui_workspace_manager_s_saves_over_selected_project() {
        let temp = TempArea::new("tui-workspace-manager-save-over");
        let old_path = temp.path("old.txt");
        let new_path = temp.path("new.txt");
        fs::write(&old_path, "old\n").expect("write old");
        fs::write(&new_path, "new\n").expect("write new");
        let projects_dir = temp.path("workspaces");
        save_gui_workspace_project(
            &gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
            &GuiWorkspaceProject {
                name: "Alpha".to_string(),
                files: vec![old_path],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save alpha");
        let mut document = open_text_file(&new_path).expect("open new");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir.clone()),
            ..EditorRuntime::default()
        };

        open_workspace_manager(&mut runtime);
        handle_workspace_manager_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
        );

        let project = parse_gui_workspace_project(
            &fs::read_to_string(
                gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
            )
            .expect("read alpha"),
        )
        .expect("parse alpha");
        assert_eq!(project.files, vec![new_path]);
        assert_eq!(runtime.workspace_manager, None);
        assert_eq!(runtime.status, "Workspace saved: Alpha");
    }

    #[test]
    fn tui_workspace_manager_d_starts_delete_confirmation() {
        let temp = TempArea::new("tui-workspace-manager-delete");
        let file_path = temp.path("note.txt");
        fs::write(&file_path, "note\n").expect("write note");
        let projects_dir = temp.path("workspaces");
        let project_path = gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "Alpha".to_string(),
                files: vec![file_path.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save alpha");
        let mut document = open_text_file(&file_path).expect("open note");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_manager(&mut runtime);
        handle_workspace_manager_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        );

        assert_eq!(runtime.workspace_manager, None);
        assert_eq!(
            runtime.workspace_pending_delete,
            Some(("Alpha".to_string(), project_path))
        );
        assert_eq!(
            runtime.workspace_prompt,
            Some(WorkspacePrompt::ConfirmDelete)
        );
    }

    #[test]
    fn tui_restore_autosave_refreshes_current_workspace_after_tab_open() {
        let temp = TempArea::new("tui-autosave-current");
        let first_path = temp.path("first.txt");
        let second_path = temp.path("second.txt");
        fs::write(&first_path, "first\n").expect("write first");
        fs::write(&second_path, "second\n").expect("write second");
        let mut first = open_text_file(&first_path).expect("open first");
        let mut workspace = EditorWorkspace::from_document(&mut first);
        let projects_dir = temp.path("workspaces");
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir.clone()),
            settings: EditorSettings {
                gui_restore_last_workspace: true,
                ..EditorSettings::default()
            },
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 2;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let project_path =
            gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME).expect("path");
        let project = parse_gui_workspace_project(
            &fs::read_to_string(project_path).expect("read saved current workspace"),
        )
        .expect("parse project");
        assert_eq!(project.name, TUI_CURRENT_WORKSPACE_NAME);
        assert_eq!(project.files, vec![first_path, second_path]);
        assert_eq!(project.active_ordinal, 1);
    }

    #[test]
    fn tui_workspace_project_open_ignores_gui_layout_and_replaces_clean_tabs() {
        let temp = TempArea::new("tui-workspace-open-project");
        let first_path = temp.path("first.txt");
        let second_path = temp.path("second.txt");
        let original_path = temp.path("original.txt");
        fs::write(&first_path, "first\n").expect("write first");
        fs::write(&second_path, "second\n").expect("write second");
        fs::write(&original_path, "original\n").expect("write original");
        let projects_dir = temp.path("workspaces");
        let project_path =
            gui_workspace_project_path(&projects_dir, "GUI Project").expect("project path");
        let project = GuiWorkspaceProject {
            name: "GUI Project".to_string(),
            files: vec![first_path.clone(), second_path.clone()],
            active_ordinal: 1,
            layout: Some(kfnotepad::GuiLayout {
                browser_visible: false,
                browser_width_px: Some(220),
                root: kfnotepad::GuiLayoutNode::Split {
                    axis: kfnotepad::GuiLayoutAxis::Horizontal,
                    ratio_per_mille: 500,
                    first: Box::new(kfnotepad::GuiLayoutNode::Leaf { ordinal: 0 }),
                    second: Box::new(kfnotepad::GuiLayoutNode::Leaf { ordinal: 1 }),
                },
                minimized_ordinals: vec![0],
            }),
        };
        save_gui_workspace_project(&project_path, &project).expect("save project");
        let mut original = open_text_file(&original_path).expect("open original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_project_named(&mut workspace, &mut runtime, "GUI Project");

        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active, 1);
        assert_eq!(workspace.tabs[0].document.as_ref().path, first_path);
        assert_eq!(workspace.active_tab().document.as_ref().path, second_path);
        assert_eq!(runtime.status, "Opened workspace: GUI Project");
    }

    #[test]
    fn tui_workspace_project_open_requires_confirmation_for_dirty_tabs() {
        let temp = TempArea::new("tui-workspace-open-dirty");
        let original_path = temp.path("original.txt");
        let project_file = temp.path("project.txt");
        fs::write(&original_path, "original\n").expect("write original");
        fs::write(&project_file, "project\n").expect("write project");
        let projects_dir = temp.path("workspaces");
        let project_path =
            gui_workspace_project_path(&projects_dir, "Project").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "Project".to_string(),
                files: vec![project_file.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");
        let mut original = open_text_file(&original_path).expect("open original");
        original
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_project_named(&mut workspace, &mut runtime, "Project");

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active_tab().document.as_ref().path, original_path);
        assert_eq!(runtime.workspace_prompt, Some(WorkspacePrompt::ConfirmOpen));
        assert!(runtime.workspace_pending_open.is_some());
        assert!(runtime.workspace_open_confirmation_pending);

        for character in "yes".chars() {
            handle_workspace_prompt_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active_tab().document.as_ref().path, project_file);
        assert_eq!(runtime.workspace_prompt, None);
        assert_eq!(runtime.status, "Opened workspace: Project");
    }

    #[test]
    fn tui_workspace_project_open_skips_missing_files_and_loads_available_tabs() {
        let temp = TempArea::new("tui-workspace-open-partial");
        let original_path = temp.path("original.txt");
        let available_path = temp.path("available.txt");
        let missing_path = temp.path("missing.txt");
        fs::write(&original_path, "original\n").expect("write original");
        fs::write(&available_path, "available\n").expect("write available");
        let projects_dir = temp.path("workspaces");
        let project_path =
            gui_workspace_project_path(&projects_dir, "Partial").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "Partial".to_string(),
                files: vec![missing_path.clone(), available_path.clone()],
                active_ordinal: 1,
                layout: None,
            },
        )
        .expect("save broken project");
        let mut original = open_text_file(&original_path).expect("open original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_project_named(&mut workspace, &mut runtime, "Partial");

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(
            workspace.active_tab().document.as_ref().path,
            available_path
        );
        assert!(runtime.status.contains("Opened workspace: Partial"));
        assert!(runtime
            .status
            .contains("skipped 1 missing/unavailable file(s)"));
        assert!(runtime.status.contains(&missing_path.display().to_string()));
        assert_ne!(workspace.active_tab().document.as_ref().path, original_path);
    }

    #[test]
    fn tui_workspace_project_open_uses_blank_tab_when_no_files_load() {
        let temp = TempArea::new("tui-workspace-open-all-missing");
        let original_path = temp.path("original.txt");
        let missing_path = temp.path("missing.txt");
        fs::write(&original_path, "original\n").expect("write original");
        let projects_dir = temp.path("workspaces");
        let project_path =
            gui_workspace_project_path(&projects_dir, "Missing").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "Missing".to_string(),
                files: vec![missing_path],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save broken project");
        let mut original = open_text_file(&original_path).expect("open original");
        let mut workspace = EditorWorkspace::from_document(&mut original);
        let mut runtime = EditorRuntime {
            workspace_projects_dir: Some(projects_dir),
            ..EditorRuntime::default()
        };

        open_workspace_project_named(&mut workspace, &mut runtime, "Missing");

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(
            workspace.active_tab().document.as_ref().path.file_name(),
            Some(std::ffi::OsStr::new("untitled.txt"))
        );
        assert_eq!(
            workspace.active_tab().document.as_ref().buffer.to_text(),
            ""
        );
        assert!(runtime.status.contains("opened blank tab"));
    }

    #[test]
    fn file_sidebar_creates_file_in_selected_directory() {
        let temp = TempArea::new("sidebar-create-file");
        fs::create_dir(temp.path("sub")).expect("create sub dir");
        let mut document = TextDocument {
            path: temp.path("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            page_rows: 20,
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
        );
        for value in "note.txt".chars() {
            handle_workspace_sidebar_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
            );
        }
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let created = temp.path("sub").join("note.txt");
        assert!(created.exists());
        assert_eq!(fs::read_to_string(&created).expect("read created file"), "");
        assert_eq!(runtime.status, "Created file note.txt");
        let selected = runtime
            .sidebar
            .as_ref()
            .and_then(FileSidebarState::selected_entry)
            .expect("selected entry");
        assert_eq!(selected.path, created);
    }

    #[test]
    fn file_sidebar_creates_directory_and_rejects_path_names() {
        let temp = TempArea::new("sidebar-create-dir");
        let mut document = TextDocument {
            path: temp.path("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            page_rows: 20,
            ..EditorRuntime::default()
        };

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
        );
        for value in "bad/name".chars() {
            handle_workspace_sidebar_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
            );
        }
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert_eq!(runtime.status, "Name must be local, not a path");
        assert!(!temp.path("bad").exists());

        runtime.sidebar_query.clear();
        for value in "notes".chars() {
            handle_workspace_sidebar_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
            );
        }
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let created = temp.path("notes");
        assert!(created.is_dir());
        assert_eq!(runtime.status, "Created directory notes/");
        let selected = runtime
            .sidebar
            .as_ref()
            .and_then(FileSidebarState::selected_entry)
            .expect("selected entry");
        assert_eq!(selected.path, created);
    }

    #[test]
    fn file_sidebar_delete_requires_confirmation_and_removes_file() {
        let temp = TempArea::new("sidebar-delete-file");
        let delete_path = temp.path("delete.txt");
        fs::write(&delete_path, "remove\n").expect("write file");
        let mut document = TextDocument {
            path: temp.path("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            page_rows: 20,
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
        );
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(delete_path.exists());
        assert_eq!(runtime.status, "Delete cancelled; type yes to confirm");

        for value in "yes".chars() {
            handle_workspace_sidebar_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
            );
        }
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(!delete_path.exists());
        assert_eq!(runtime.status, "Deleted delete.txt");
    }

    #[test]
    fn file_sidebar_delete_directory_warns_and_removes_children_after_confirmation() {
        let temp = TempArea::new("sidebar-delete-dir");
        let delete_dir = temp.path("delete-dir");
        fs::create_dir(&delete_dir).expect("create dir");
        fs::write(delete_dir.join("child.txt"), "child\n").expect("write child");
        let mut document = TextDocument {
            path: temp.path("current.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
            page_rows: 20,
            ..EditorRuntime::default()
        };
        runtime.sidebar.as_mut().expect("sidebar").selected = 1;

        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
        );
        assert_eq!(
            runtime.status,
            "Delete directory and all contents? type yes: "
        );

        for value in "yes".chars() {
            handle_workspace_sidebar_key_event(
                &mut workspace,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
            );
        }
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(!delete_dir.exists());
        assert_eq!(runtime.status, "Deleted delete-dir/");
    }

    #[test]
    fn file_sidebar_delete_refuses_open_dirty_file_and_symlink() {
        #[cfg(unix)]
        use std::os::unix::fs::symlink;

        let temp = TempArea::new("sidebar-delete-refuse");
        let dirty_path = temp.path("dirty.txt");
        let target_path = temp.path("target.txt");
        let link_path = temp.path("link.txt");
        fs::write(&dirty_path, "dirty\n").expect("write dirty file");
        fs::write(&target_path, "target\n").expect("write target file");
        #[cfg(unix)]
        symlink(&target_path, &link_path).expect("create symlink");

        let mut document = TextDocument {
            path: dirty_path.clone(),
            buffer: kfnotepad::TextBuffer::from_text("dirty\n"),
        };
        document.buffer.insert_char(0, 0, '!').expect("mark dirty");
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let mut runtime = EditorRuntime {
            sidebar: Some(FileSidebarState {
                current_dir: temp.root.clone(),
                entries: vec![FileSidebarEntry {
                    label: String::from("dirty.txt"),
                    path: dirty_path.clone(),
                    kind: FileSidebarEntryKind::File,
                }],
                selected: 0,
                scroll: 0,
            }),
            sidebar_prompt: Some(SidebarPrompt::DeleteConfirm {
                entry: FileSidebarEntry {
                    label: String::from("dirty.txt"),
                    path: dirty_path.clone(),
                    kind: FileSidebarEntryKind::File,
                },
                recursive: false,
            }),
            sidebar_query: String::from("yes"),
            ..EditorRuntime::default()
        };

        apply_sidebar_prompt(&mut workspace, &mut runtime);
        assert!(dirty_path.exists());
        assert_eq!(runtime.status, "Cannot delete an open modified file");

        #[cfg(unix)]
        {
            runtime.sidebar_prompt = Some(SidebarPrompt::DeleteConfirm {
                entry: FileSidebarEntry {
                    label: String::from("link.txt"),
                    path: link_path.clone(),
                    kind: FileSidebarEntryKind::File,
                },
                recursive: false,
            });
            runtime.sidebar_query = String::from("yes");
            apply_sidebar_prompt(&mut workspace, &mut runtime);
            assert!(link_path.exists());
            assert_eq!(runtime.status, "Refusing to delete symlink");
        }
    }

    #[test]
    fn mouse_click_on_tab_strip_switches_workspace_tab() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: kfnotepad::TextBuffer::from_text("first\n"),
        };
        let second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("second\n"),
        };
        let mut workspace = EditorWorkspace {
            tabs: vec![
                EditorTab {
                    document: EditorTabDocument::Owned(first),
                    state: EditorTabState::default(),
                },
                EditorTab {
                    document: EditorTabDocument::Owned(second),
                    state: EditorTabState::default(),
                },
            ],
            active: 0,
        };
        let mut runtime = EditorRuntime::default();
        let first_label_width = text_display_width(" 1:first.txt ");

        assert_eq!(
            handle_workspace_mouse_event(
                &mut workspace,
                &mut runtime,
                left_click(first_label_width as u16 + 1, 1),
                MouseContext {
                    viewport_start: 0,
                    horizontal_offset: 0,
                    visible_rows: 10,
                    gutter_width: 4,
                    terminal_width: 80,
                    sidebar_width: 0,
                    body_top: 2,
                }
            ),
            InputResult::Handled
        );

        assert_eq!(workspace.active, 1);
        assert_eq!(runtime.status, "Tab 2/2: second.txt");
    }

    #[test]
    fn sidebar_mouse_wheel_moves_selection_without_wrapping() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("current\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            sidebar: Some(sidebar_fixture(12)),
            ..EditorRuntime::default()
        };
        let context = MouseContext {
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            gutter_width: 4,
            terminal_width: 80,
            sidebar_width: SIDEBAR_WIDTH,
            body_top: 1,
        };

        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollDown, 2, 2),
                context
            ),
            InputResult::Handled
        );
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 1);
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);

        for _ in 0..3 {
            assert_eq!(
                handle_mouse_event(
                    &mut document,
                    &mut cursor,
                    &mut runtime,
                    mouse_event(MouseEventKind::ScrollDown, 2, 2),
                    context
                ),
                InputResult::Handled
            );
        }
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 4);
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 2);

        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollUp, 2, 2),
                context
            ),
            InputResult::Handled
        );
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 3);
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 2);

        runtime.sidebar.as_mut().expect("sidebar").selected = 0;
        runtime.sidebar.as_mut().expect("sidebar").scroll = 0;
        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollUp, 2, 2),
                context
            ),
            InputResult::Ignored
        );
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 0);
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);
    }

    #[test]
    fn editor_body_mouse_wheel_moves_cursor_by_rows() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\nfive\nsix\nseven\n"),
        };
        let mut cursor = Cursor { row: 1, column: 2 };
        let mut runtime = EditorRuntime {
            sidebar: Some(sidebar_fixture(4)),
            ..EditorRuntime::default()
        };

        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollDown, (SIDEBAR_WIDTH + 2) as u16, 2),
                MouseContext {
                    viewport_start: 0,
                    horizontal_offset: 0,
                    visible_rows: 3,
                    gutter_width: 4,
                    terminal_width: 80,
                    sidebar_width: SIDEBAR_WIDTH,
                    body_top: 1,
                }
            ),
            InputResult::Handled
        );
        assert_eq!(cursor, Cursor { row: 4, column: 2 });
        assert_eq!(runtime.status, "Scroll down");
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 0);
        assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);

        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollUp, (SIDEBAR_WIDTH + 2) as u16, 2),
                MouseContext {
                    viewport_start: 0,
                    horizontal_offset: 0,
                    visible_rows: 3,
                    gutter_width: 4,
                    terminal_width: 80,
                    sidebar_width: SIDEBAR_WIDTH,
                    body_top: 1,
                }
            ),
            InputResult::Handled
        );
        assert_eq!(cursor, Cursor { row: 1, column: 2 });
        assert_eq!(runtime.status, "Scroll up");
    }

    #[test]
    fn editor_body_mouse_wheel_ignores_header_and_active_menu() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\n"),
        };
        let mut cursor = Cursor { row: 1, column: 0 };
        let mut runtime = EditorRuntime::default();
        let context = MouseContext {
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            gutter_width: 4,
            terminal_width: 80,
            sidebar_width: 0,
            body_top: 1,
        };

        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollDown, 5, 0),
                context
            ),
            InputResult::Ignored
        );
        assert_eq!(cursor, Cursor { row: 1, column: 0 });

        runtime.menu = Some(MenuState {
            group: MenuGroup::File,
            selected: 0,
        });
        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollDown, 5, 2),
                context
            ),
            InputResult::Ignored
        );
        assert_eq!(cursor, Cursor { row: 1, column: 0 });
    }

    fn left_click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn sidebar_fixture(count: usize) -> FileSidebarState {
        FileSidebarState {
            current_dir: PathBuf::from("."),
            entries: (0..count)
                .map(|index| FileSidebarEntry {
                    label: format!("file-{index}.txt"),
                    path: PathBuf::from(format!("file-{index}.txt")),
                    kind: FileSidebarEntryKind::File,
                })
                .collect(),
            selected: 0,
            scroll: 0,
        }
    }

    struct FakeBackend {
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl TerminalBackend for FakeBackend {
        type Writer = Vec<u8>;

        fn enter() -> io::Result<(Self::Writer, Self)> {
            let events = Rc::new(RefCell::new(vec!["enter"]));
            Ok((Vec::new(), Self { events }))
        }

        fn restore(&mut self) {
            self.events.borrow_mut().push("restore");
        }
    }

    impl TerminalSession<FakeBackend> {
        fn enter_fake() -> io::Result<(Self, Rc<RefCell<Vec<&'static str>>>)> {
            let (stdout, backend) = FakeBackend::enter()?;
            let events = Rc::clone(&backend.events);
            Ok((Self { stdout, backend }, events))
        }
    }

    #[test]
    fn terminal_session_restores_backend_on_drop() {
        let (session, events) = TerminalSession::enter_fake().expect("enter fake terminal");

        drop(session);

        assert_eq!(&*events.borrow(), &["enter", "restore"]);
    }

    #[test]
    fn keyboard_enhancement_flags_disambiguate_modified_keys_only() {
        let flags = editor_keyboard_enhancement_flags();

        assert!(flags.contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
        assert!(!flags.contains(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
        assert!(!flags.contains(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES));
    }

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
    fn render_tab_strip_wraps_long_tab_labels_before_editor_body() {
        let document = TextDocument {
            path: PathBuf::from("active.txt"),
            buffer: kfnotepad::TextBuffer::from_text("body\n"),
        };
        let tabs = vec![
            TabStripItem {
                label: String::from("getting-started-guide.md"),
                active: false,
                dirty: false,
            },
            TabStripItem {
                label: String::from("release-notes-draft.md"),
                active: false,
                dirty: false,
            },
            TabStripItem {
                label: String::from("keyboard-shortcuts-reference.md"),
                active: true,
                dirty: false,
            },
        ];
        let mut output = Vec::new();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 0 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 2,
                status: "ready",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &tabs,
                search_highlight: None,
            },
            &SyntaxHighlighter::default(),
            42,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert_eq!(tab_strip_height_for_width(&tabs, 42), 4);
        assert!(output.contains("\x1b[2;1H"));
        assert!(output.contains("\x1b[3;1H"));
        assert!(output.contains("\x1b[4;1H"));
        assert!(output.contains("\x1b[5;1H\x1b[2K"));
        assert!(output.contains("body"));
    }

    #[test]
    fn render_does_not_clear_entire_screen_between_frames() {
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
                visible_rows: 3,
                status: "ready",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(!output.contains("\x1b[2J"));
        assert!(output.contains("\x1b[2K"));
    }

    #[test]
    fn render_clears_empty_body_rows_without_full_screen_clear() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("only line\n"),
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
                visible_rows: 3,
                status: "ready",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\x1b[2;1H\x1b[2K"));
        assert!(output.contains("\x1b[3;1H\x1b[2K"));
        assert!(output.contains("\x1b[4;1H\x1b[2K"));
        assert!(!output.contains("\x1b[2J"));
    }

    #[test]
    fn render_tab_strip_shows_active_and_dirty_tabs_above_body() {
        let document = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: kfnotepad::TextBuffer::from_text("body\n"),
        };
        let tabs = vec![
            TabStripItem {
                label: String::from("first.txt"),
                active: false,
                dirty: true,
            },
            TabStripItem {
                label: String::from("second.txt"),
                active: true,
                dirty: false,
            },
        ];
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
                status: "ready",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &tabs,
                search_highlight: None,
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\x1b[2;1H\x1b[2K"));
        assert!(output.contains(" 1:first.txt* "));
        assert!(output.contains(" 2:second.txt "));
        assert!(output.contains("\x1b[3;1H\x1b[2K"));
        assert!(output.contains("body"));
        assert!(output.contains("\x1b[5;1H"));
    }

    #[test]
    fn render_preserves_header_state_when_path_is_long() {
        let document = TextDocument {
            path: PathBuf::from("/very/long/path/that/would/otherwise/hide/the/state/note.txt"),
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
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            32,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("kfnotepad"));
        assert!(output.contains(" saved "));
        assert!(output.contains("…"));
    }

    #[test]
    fn status_line_preserves_cursor_and_mode_metadata() {
        let status = compose_status_line(
            " very long transient status text that can be shortened ",
            " Ln 12, Col 80 | num:on | wrap:off | x:42 | nocturne | modified ",
            64,
        );

        assert_eq!(status.chars().count(), 64);
        assert!(status.contains("Col 80"));
        assert!(status.contains("wrap:off"));
        assert!(status.contains("x:42"));
        assert!(status.contains("modified"));
    }

    #[test]
    fn search_status_preserves_query_tail_and_cursor() {
        let status = compose_prompt_status_line(
            "Search: ",
            "a very long search query",
            " Ln 1, Col 1 | num:on | wrap:off | x:0 | nocturne | saved ",
            72,
        );

        assert_eq!(status.text.chars().count(), 72);
        assert!(status.text.contains("Search:"));
        assert!(status.text.contains("…"));
        assert!(status.text.contains("ry"));
        assert!(status.text.contains("Col 1"));
        assert!(status.cursor_column.is_some());
    }

    #[test]
    fn render_starts_at_viewport_offset() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 1, column: 0 },
                viewport_start: 1,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
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
        assert!(!output.contains(" 1 one"));
        assert!(output.contains(" 2"));
        assert!(output.contains("two"));
        assert!(!output.contains(" 3 three"));
    }

    #[test]
    fn render_can_hide_line_number_gutter() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 1 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 2,
                status: "status",
                settings: EditorSettings {
                    show_line_numbers: false,
                    ..EditorSettings::default()
                },
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(!output.contains(" 1 one"));
        assert!(output.contains("one"));
        assert!(output.contains("num:off"));
    }

    #[test]
    fn render_positions_rows_and_truncates_long_lines() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\nsecond\n"),
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
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            10,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\u{1b}[2;1H"));
        assert!(output.contains("\u{1b}[3;1H"));
        assert!(output.contains("\u{1b}[4;1H"));
        assert!(output.contains(" 1 "));
        assert!(output.contains("abcde"));
        assert!(output.contains(" 2 "));
        assert!(output.contains("secon"));
        assert!(!output.contains("fgh"));
    }

    #[test]
    fn render_reserves_columns_for_sidebar() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\n"),
        };
        let mut output = Vec::new();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 0 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 2,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 10,
                tab_strip: &[],
                search_highlight: None,
            },
            &SyntaxHighlighter::default(),
            40,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\u{1b}[1;11H"));
        assert!(output.contains("\u{1b}[2;11H"));
        assert!(!output.contains("\u{1b}[1;1H"));
    }

    #[test]
    fn render_keeps_terminal_cursor_visible_at_editor_cursor() {
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
                cursor: Cursor { row: 0, column: 2 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            20,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\u{1b}[?25h"));
        assert!(output.ends_with("\u{1b}[2;7H"));
    }

    #[test]
    fn render_paints_active_cursor_cell() {
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
                cursor: Cursor { row: 0, column: 1 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            20,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("\u{1b}[2;6H\u{1b}[7me\u{1b}[27m"));
        assert!(output.ends_with("\u{1b}[2;6H"));
    }

    #[test]
    fn render_highlights_visible_search_matches() {
        assert_eq!(
            search_match_ranges(
                "Alpha beta alpha",
                "alpha",
                SearchMode {
                    case_sensitive: false,
                },
            ),
            vec![0..5, 11..16]
        );
        let mut direct = Vec::new();
        let mut display_column = 0;
        let mut source_column = 0;
        let mut remaining = 10;
        assert_eq!(
            EditorTheme::default().search_bg,
            Color::Rgb {
                r: 90,
                g: 230,
                b: 245
            }
        );
        let direct_range = 0..5;
        print_line_window_with_search(
            &mut direct,
            LineWindowSearchView {
                text: "Alpha",
                start_column: 0,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: std::slice::from_ref(&direct_range),
                base_fg: None,
                frame: RenderFrame {
                    theme: EditorTheme::default(),
                    gutter_width: 1,
                    terminal_width: 20,
                    origin_column: 0,
                    body_top: 1,
                },
            },
        )
        .expect("paint direct search");
        assert!(!direct.is_empty());
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("Alpha beta alpha\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 6 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 3,
                status: "ready",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: Some(SearchHighlightView {
                    query: "alpha",
                    mode: SearchMode {
                        case_sensitive: false,
                    },
                }),
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains(" beta "));
    }

    #[test]
    fn render_moves_cursor_to_active_search_prompt() {
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
                visible_rows: 1,
                status: "Search: beta",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("Search: beta"));
        assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
        assert!(output.ends_with("\u{1b}[3;14H"));
    }

    #[test]
    fn render_moves_cursor_to_go_to_line_prompt() {
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
                visible_rows: 1,
                status: "Go to line: 42",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            80,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("Go to line: 42"));
        assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
        assert!(output.ends_with("\u{1b}[3;16H"));
    }

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

        write_command_palette_overlay(&mut output, &palette, 10, 0, 90, 1, EditorTheme::default())
            .expect("render command palette");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("Command: reader"));
        assert!(output.contains("Reader mode"));
        assert!(output.contains("Reader slower"));
        assert!(output.contains("Reader faster"));
    }

    #[test]
    fn render_expands_tabs_to_terminal_columns() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("a\tb\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 2 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            20,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("a   b"));
        assert!(output.ends_with("\u{1b}[2;9H"));
    }

    #[test]
    fn render_positions_cursor_after_wide_character() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("界x\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 1 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            20,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("界x"));
        assert!(output.ends_with("\u{1b}[2;7H"));
    }

    #[test]
    fn render_keeps_combining_mark_at_zero_width() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("e\u{301}x\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 2 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            20,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("e\u{301}x"));
        assert!(output.ends_with("\u{1b}[2;6H"));
    }

    #[test]
    fn render_uses_horizontal_offset_for_long_lines() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 10 },
                viewport_start: 0,
                horizontal_offset: 6,
                visible_rows: 1,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            10,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("ghijk"));
        assert!(!output.contains("abcde"));
        assert!(output.ends_with("\u{1b}[2;9H"));
    }

    #[test]
    fn render_wraps_long_lines_when_enabled() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 8 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 3,
                status: "status",
                settings: EditorSettings {
                    wrap_lines: true,
                    ..EditorSettings::default()
                },
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            10,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("rendered output is UTF-8");
        assert!(output.contains("abcdef"));
        assert!(output.contains("ghijkl"));
        assert!(output.contains("mnop"));
        assert!(output.ends_with("\u{1b}[3;7H"));
    }

    #[test]
    fn wrap_prefers_word_boundaries() {
        assert_eq!(
            wrapped_chunk_texts("alpha beta gamma", 10),
            vec!["alpha beta".to_string(), "gamma".to_string()]
        );
    }

    #[test]
    fn wrap_falls_back_to_character_chunks_for_long_words() {
        assert_eq!(
            wrapped_chunk_texts("superlongword", 5),
            vec!["super".to_string(), "longw".to_string(), "ord".to_string()]
        );
    }

    #[test]
    fn wrap_preserves_leading_indentation_on_first_visual_row() {
        assert_eq!(
            wrapped_chunk_texts("    let value = call();", 12),
            vec![
                "    let".to_string(),
                "value =".to_string(),
                "call();".to_string()
            ]
        );
        assert_eq!(
            wrapped_line_chunks("    let value = call();", 12)[0].start_column,
            0
        );
        assert_eq!(
            wrapped_line_chunks("    let value = call();", 12)[1].start_column,
            8
        );
    }

    fn wrapped_chunk_texts(line: &str, width: usize) -> Vec<String> {
        wrapped_line_chunks(line, width)
            .into_iter()
            .map(|chunk| chunk.text)
            .collect()
    }

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

    #[test]
    fn page_up_and_down_move_by_visible_page() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\nfive\n"),
        };
        let mut cursor = Cursor { row: 1, column: 3 };
        let mut runtime = EditorRuntime {
            page_rows: 2,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 3, column: 3 });
        assert_eq!(runtime.status, "Page down");

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 1, column: 3 });
        assert_eq!(runtime.status, "Page up");

        cursor = Cursor { row: 3, column: 4 };
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 4, column: 4 });
    }

    #[test]
    fn syntax_highlighter_detects_rust_and_falls_back_to_plain_text() {
        let highlighter = SyntaxHighlighter::default();
        let rust_document = TextDocument {
            path: PathBuf::from("main.rs"),
            buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
        };
        let text_document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("plain note\n"),
        };

        assert_eq!(highlighter.syntax_name_for_document(&rust_document), "Rust");
        assert_eq!(
            highlighter.syntax_name_for_document(&text_document),
            "Plain Text"
        );
        assert!(highlighter
            .highlight_line(&rust_document, "fn main() {}")
            .is_some());
        assert!(highlighter
            .highlight_line(&text_document, "plain note")
            .is_none());
    }

    #[test]
    fn syntax_highlighter_keeps_state_before_viewport() {
        let highlighter = SyntaxHighlighter::default();
        let document = TextDocument {
            path: PathBuf::from("main.rs"),
            buffer: kfnotepad::TextBuffer::from_text("/* start\ninside\n*/\nfn main() {}\n"),
        };

        let stateful = highlighter.highlight_visible_lines(&document, 1, 1);
        let reset = highlighter
            .highlight_line(&document, "inside")
            .expect("standalone Rust line highlights");
        let stateful_line = stateful
            .first()
            .and_then(Option::as_ref)
            .expect("stateful Rust line highlights");

        assert_ne!(stateful_line[0].0.foreground, reset[0].0.foreground);
        assert_eq!(stateful_line[0].1, "inside");
    }

    #[test]
    fn viewport_follows_cursor_down_and_up() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\n"),
        };

        assert_eq!(
            clamp_viewport(
                &document,
                Cursor { row: 3, column: 0 },
                0,
                2,
                EditorSettings::default(),
                2,
                80
            ),
            2
        );
        assert_eq!(
            clamp_viewport(
                &document,
                Cursor { row: 0, column: 0 },
                2,
                2,
                EditorSettings::default(),
                2,
                80
            ),
            0
        );
    }

    #[test]
    fn wrapped_viewport_can_scroll_to_last_source_line() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one two three four five\nlast\n"),
        };
        let settings = EditorSettings {
            wrap_lines: true,
            ..EditorSettings::default()
        };

        assert_eq!(
            clamp_passive_viewport(&document, 99, 10, settings),
            1,
            "wrapped mode must allow the final source line to become visible even when source line count is smaller than visible rows"
        );
        assert_eq!(
            clamp_viewport(
                &document,
                Cursor { row: 1, column: 0 },
                0,
                3,
                settings,
                2,
                10
            ),
            1,
            "cursor-following clamp must account for visual rows in wrapped mode"
        );
    }

    #[test]
    fn dirty_quit_requires_second_ctrl_q() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\n"),
        };
        document.buffer.insert_char(0, 0, '!').expect("edit buffer");
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();
        let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            quit
        ));
        assert!(runtime.quit_confirmation_pending);
        assert!(runtime.status.contains("Unsaved changes"));

        assert!(handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            quit
        ));
    }

    #[test]
    fn backspace_key_updates_buffer_and_cursor() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\n"),
        };
        let mut cursor = Cursor { row: 0, column: 5 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
        ));

        assert_eq!(document.buffer.line(0), Some("hell"));
        assert_eq!(cursor, Cursor { row: 0, column: 4 });
        assert!(document.buffer.is_dirty());
    }

    #[test]
    fn ctrl_backspace_and_ctrl_delete_update_buffer_by_word() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\n"),
        };
        let mut cursor = Cursor { row: 0, column: 11 };
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL)
        ));

        assert_eq!(document.buffer.line(0), Some("alpha gamma"));
        assert_eq!(cursor, Cursor { row: 0, column: 6 });
        assert_eq!(runtime.status, "Modified");
        assert!(!runtime.quit_confirmation_pending);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::CONTROL)
        ));

        assert_eq!(document.buffer.line(0), Some("alpha "));
        assert_eq!(cursor, Cursor { row: 0, column: 6 });
        assert_eq!(runtime.status, "Modified");
    }

    #[test]
    fn ctrl_k_and_edit_menu_delete_to_line_end() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\nnext\n"),
        };
        let mut cursor = Cursor { row: 0, column: 6 };
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)
        ));
        assert_eq!(document.buffer.line(0), Some("alpha "));
        assert_eq!(cursor, Cursor { row: 0, column: 6 });
        assert_eq!(runtime.status, "Modified");
        assert!(!runtime.quit_confirmation_pending);

        assert!(document.buffer.undo_last_edit());
        cursor = Cursor { row: 0, column: 6 };
        runtime.menu = Some(MenuState {
            group: MenuGroup::Edit,
            selected: 6,
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
    fn home_and_end_keys_move_within_current_line() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
        };
        let mut cursor = Cursor { row: 0, column: 2 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::End, KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 5 });

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
    }

    #[test]
    fn insert_key_toggles_overwrite_mode_for_typed_characters() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("abcd\n"),
        };
        let mut cursor = Cursor { row: 0, column: 1 };
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
        ));
        assert!(runtime.overwrite_mode);
        assert_eq!(runtime.status, "Overwrite on");
        assert!(!runtime.quit_confirmation_pending);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT)
        ));
        assert_eq!(document.buffer.line(0), Some("aXcd"));
        assert_eq!(cursor, Cursor { row: 0, column: 2 });
        assert_eq!(runtime.status, "Modified overwrite");

        assert!(document.buffer.undo_last_edit());
        assert_eq!(document.buffer.line(0), Some("abcd"));
        assert!(!document.buffer.undo_last_edit());

        cursor.column = 4;
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT)
        ));
        assert_eq!(document.buffer.line(0), Some("abcd!"));
        assert_eq!(cursor, Cursor { row: 0, column: 5 });

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
        ));
        assert!(!runtime.overwrite_mode);
        assert_eq!(runtime.status, "Insert mode");
        cursor.column = 1;
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT)
        ));
        assert_eq!(document.buffer.line(0), Some("aYbcd!"));
        assert_eq!(runtime.status, "Modified");
    }

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

    #[test]
    fn ctrl_a_and_ctrl_e_move_within_current_line() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
        };
        let mut cursor = Cursor { row: 0, column: 2 };
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 5 });
        assert!(!runtime.quit_confirmation_pending);

        runtime.quit_confirmation_pending = true;
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert!(!runtime.quit_confirmation_pending);
    }

    #[test]
    fn ctrl_home_and_end_move_to_document_edges() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
        };
        let mut cursor = Cursor { row: 1, column: 2 };
        let mut runtime = EditorRuntime {
            quit_confirmation_pending: true,
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 2, column: 5 });
        assert_eq!(runtime.status, "Bottom");
        assert!(!runtime.quit_confirmation_pending);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(runtime.status, "Top");
    }

    #[test]
    fn ctrl_left_and_right_move_by_words() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha, beta\n  gamma\n"),
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
            KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 7 });
        assert!(!runtime.quit_confirmation_pending);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 1, column: 2 });

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)
        ));
        assert_eq!(cursor, Cursor { row: 0, column: 7 });
    }

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
    fn tab_and_backtab_indent_and_unindent_current_line() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)
        ));
        assert_eq!(document.buffer.line(0), Some("    alpha"));
        assert_eq!(cursor.column, 4);
        assert_eq!(runtime.status, "Indented");

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE)
        ));
        assert_eq!(document.buffer.line(0), Some("alpha"));
        assert_eq!(cursor.column, 0);
        assert_eq!(runtime.status, "Unindented");
    }

    #[test]
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
        let sample = syntect::highlighting::Color {
            r: 120,
            g: 140,
            b: 230,
            a: 255,
        };

        assert_eq!(
            syntect_color_to_terminal(sample, EditorThemeId::Nocturne),
            Color::Rgb {
                r: 132,
                g: 172,
                b: 255,
            }
        );
        assert_eq!(
            syntect_color_to_terminal(sample, EditorThemeId::Terror),
            Color::Rgb {
                r: 136,
                g: 172,
                b: 255,
            }
        );
        assert_ne!(
            syntect_color_to_terminal(sample, EditorThemeId::Nocturne),
            syntect_color_to_terminal(sample, EditorThemeId::Paper)
        );
    }

    #[test]
    fn ctrl_z_undo_restores_buffer_and_clamps_cursor() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
        };
        document
            .buffer
            .delete_char(0, 5)
            .expect("join lines for setup");
        let mut cursor = Cursor { row: 1, column: 99 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL)
        ));

        assert_eq!(
            document.buffer.lines(),
            &["hello".to_string(), "world".to_string()]
        );
        assert_eq!(cursor, Cursor { row: 1, column: 5 });
        assert_eq!(runtime.status, "Undone");
    }

    #[test]
    fn ctrl_y_redo_restores_buffer_and_clamps_cursor() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
        };
        document
            .buffer
            .delete_char(0, 5)
            .expect("join lines for setup");
        assert!(document.buffer.undo_last_edit());
        let mut cursor = Cursor { row: 1, column: 99 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL)
        ));

        assert_eq!(document.buffer.lines(), &["helloworld".to_string()]);
        assert_eq!(cursor, Cursor { row: 0, column: 10 });
        assert_eq!(runtime.status, "Redone");
    }

    #[test]
    fn f3_repeats_last_search() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
        ));
        for key in [
            KeyCode::Char('a'),
            KeyCode::Char('l'),
            KeyCode::Char('p'),
            KeyCode::Char('h'),
            KeyCode::Char('a'),
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
        assert_eq!(runtime.last_search_query, "alpha");

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
        ));

        assert_eq!(cursor, Cursor { row: 1, column: 5 });
        assert_eq!(runtime.status, "Found: alpha");
    }

    #[test]
    fn f3_repeats_last_search_case_insensitive_by_default() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("Alpha\nbeta\n"),
        };
        let mut cursor = Cursor { row: 1, column: 4 };
        let mut runtime = EditorRuntime {
            last_search_query: String::from("alpha"),
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
        ));

        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(runtime.status, "Found: alpha");
    }

    #[test]
    fn f3_without_search_reports_missing_query() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
        ));

        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(runtime.status, "No previous search");
    }

    #[test]
    fn shift_f3_repeats_last_search_backwards() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\ngamma alpha\n"),
        };
        let mut cursor = Cursor { row: 2, column: 7 };
        let mut runtime = EditorRuntime {
            last_search_query: String::from("alpha"),
            ..EditorRuntime::default()
        };

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
        ));

        assert_eq!(cursor, Cursor { row: 1, column: 5 });
        assert_eq!(runtime.status, "Found: alpha");
    }

    #[test]
    fn shift_f3_without_search_reports_missing_query() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
        ));

        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(runtime.status, "No previous search");
    }

    #[test]
    fn ctrl_g_goes_to_line_and_clamps_column() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
        };
        let mut cursor = Cursor { row: 0, column: 99 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
        ));
        assert!(runtime.goto_line_active);
        assert_eq!(runtime.status, "Go to line: ");

        for key in [KeyCode::Char('3'), KeyCode::Enter] {
            assert!(!handle_key_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                KeyEvent::new(key, KeyModifiers::NONE)
            ));
        }

        assert!(!runtime.goto_line_active);
        assert_eq!(cursor, Cursor { row: 2, column: 5 });
        assert_eq!(runtime.status, "Line 3");
    }

    #[test]
    fn go_to_line_rejects_out_of_range_line() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
        };
        let mut cursor = Cursor { row: 1, column: 1 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
        ));
        for key in [KeyCode::Char('9'), KeyCode::Char('9'), KeyCode::Enter] {
            assert!(!handle_key_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                KeyEvent::new(key, KeyModifiers::NONE)
            ));
        }

        assert!(!runtime.goto_line_active);
        assert_eq!(cursor, Cursor { row: 1, column: 1 });
        assert_eq!(runtime.status, "Line out of range: 99");
    }

    #[test]
    fn ctrl_f_search_moves_cursor_to_match() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
        ));
        assert!(runtime.search_active);

        for value in ['b', 'e', 't', 'a'] {
            assert!(!handle_key_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE)
            ));
        }

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
        ));

        assert_eq!(cursor, Cursor { row: 1, column: 0 });
        assert!(!runtime.search_active);
        assert_eq!(runtime.status, "Found: beta");
    }

    #[test]
    fn ctrl_f_search_remembers_history_and_recalls_with_arrows() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\ngamma\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        for query in ["alpha", "beta", "alpha"] {
            start_search(&mut runtime);
            runtime.search_query = query.to_string();
            handle_search_key_event(
                &document,
                &mut cursor,
                &mut runtime,
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            );
        }

        assert_eq!(
            runtime.search_history,
            vec![String::from("alpha"), String::from("beta")]
        );

        start_search(&mut runtime);
        handle_search_key_event(
            &document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        );
        assert_eq!(runtime.search_query, "alpha");
        handle_search_key_event(
            &document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        );
        assert_eq!(runtime.search_query, "beta");
        handle_search_key_event(
            &document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert_eq!(runtime.search_query, "alpha");
    }

    #[test]
    fn search_case_toggle_persists_and_exact_case_changes_results() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text("Alpha\nalpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime {
            last_search_query: String::from("alpha"),
            ..EditorRuntime::default()
        };

        toggle_search_case(&mut runtime);
        assert!(runtime.settings.search_case_sensitive);

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
        ));
        assert_eq!(cursor, Cursor { row: 1, column: 0 });
    }

    #[test]
    fn save_failure_status_does_not_include_buffer_contents() {
        let secret = "SUPER_SECRET_TOKEN";
        let mut document = TextDocument {
            path: PathBuf::from("."),
            buffer: kfnotepad::TextBuffer::from_text(secret),
        };
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
        ));

        assert!(runtime.status.starts_with("Save failed:"));
        assert!(!runtime.status.contains(secret));
    }

    #[test]
    fn tui_save_refuses_external_modification_since_open() {
        let temp = TempArea::new("tui-save-conflict");
        let path = temp.path("note.txt");
        fs::write(&path, "original\n").expect("write original");
        let mut document = open_text_file(&path).expect("open document");
        document.buffer.insert_char(0, 0, '!').expect("edit buffer");
        fs::write(&path, "external\n").expect("external edit");
        let mut cursor = Cursor { row: 0, column: 0 };
        let mut runtime = EditorRuntime::default();

        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
        ));

        assert_eq!(fs::read_to_string(&path).expect("read file"), "external\n");
        assert!(document.buffer.is_dirty());
        assert!(runtime
            .status
            .contains("file changed on disk since open or last save"));
    }
}
