use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{env, fs, io};

use iced::advanced::{
    input_method, layout as advanced_layout, overlay as advanced_overlay,
    renderer as advanced_renderer,
    widget::{Operation as AdvancedOperation, Tree, Widget},
    Clipboard as AdvancedClipboard, Layout as AdvancedLayout, Shell as AdvancedShell,
};
use iced::keyboard::key::{Key, Named};
use iced::widget::{
    button, checkbox, column, container, mouse_area, pane_grid, responsive, rich_text, row,
    scrollable, slider, span, text, text::Wrapping, text_editor, text_input,
};
use iced::{
    clipboard, event, highlighter, keyboard, mouse, window, Alignment, Background, Border, Color,
    Element, Event, Font, Length, Pixels, Rectangle, Shadow, Size, Subscription, Task, Theme,
    Vector,
};
use iced_aw::{menu, Menu, MenuBar};
use iced_swdir_tree::{
    DirectoryFilter, DirectoryTree, DirectoryTreeEvent, IconRole, IconSpec, IconTheme,
};
use kfnotepad::{
    delete_gui_workspace_project, delete_managed_note, delete_next_word, delete_previous_word,
    delete_to_line_end, go_to_document_end, go_to_document_start, go_to_line,
    gui_workspace_project_path, list_file_sidebar_entries, list_gui_workspace_projects,
    list_managed_notes, load_editor_settings, managed_notes_dir, open_or_create_managed_note,
    open_text_file, parse_gui_layout, parse_gui_workspace_project, redo_document_edit,
    repeat_search_next, repeat_search_previous, save_editor_settings, save_gui_layout,
    save_gui_workspace_project, save_text_buffer, save_text_document, undo_document_edit,
    Cursor as DocumentCursor, EditorSettings, EditorTabState, EditorThemeId, FileSidebarEntry,
    FileSidebarEntryKind, GoToLineResult, GuiCloseTileResult, GuiFileBrowser, GuiFontFamily,
    GuiLayout, GuiLayoutAxis, GuiLayoutNode, GuiLeftPanelMode, GuiLeftPanelState, GuiTileId,
    GuiTileSaveStatus, GuiWorkspace, GuiWorkspaceProject, GuiWorkspaceProjectDeleteResult,
    GuiWorkspaceProjectEntry, ManagedNoteDeleteResult, ManagedNoteEntry, SearchRepeatResult,
    SyntaxHighlightCacheState, SyntaxHighlighter, TextBuffer, TextDocument, UndoRedoResult,
    MAX_GUI_FONT_SIZE, MAX_GUI_READER_LINES_PER_MINUTE, MIN_GUI_FONT_SIZE,
    MIN_GUI_READER_LINES_PER_MINUTE, VERSION,
};
#[cfg(not(test))]
use kfnotepad::{editor_config_path, gui_layout_path, gui_workspace_projects_dir};
use nerd_font_symbols as nf;
use syntect::highlighting::Style as SyntectStyle;
use unicode_width::UnicodeWidthChar;

fn main() -> iced::Result {
    let launch = match GuiLaunch::from_args(env::args().skip(1).collect()) {
        LaunchAction::Run(launch) => launch,
        LaunchAction::Printed => return Ok(()),
        LaunchAction::Error(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    iced::application(
        move || KfnotepadGui::new_with_task(launch.clone()),
        update,
        view,
    )
    .title(title)
    .theme(theme)
    .subscription(subscription)
    .window_size(Size::new(1100.0, 720.0))
    .exit_on_close_request(false)
    .centered()
    .run()
}

#[derive(Clone)]
struct GuiLaunch {
    requested_paths: Vec<PathBuf>,
}

enum LaunchAction {
    Run(GuiLaunch),
    Printed,
    Error(String),
}

impl GuiLaunch {
    fn from_args(args: Vec<String>) -> LaunchAction {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            print_gui_help();
            return LaunchAction::Printed;
        }
        if args.iter().any(|arg| arg == "--version" || arg == "-V") {
            println!("kfnotepad-gui {VERSION}");
            return LaunchAction::Printed;
        }
        if args.iter().any(|arg| arg == "--describe") {
            print_gui_describe();
            return LaunchAction::Printed;
        }
        if let Some(option) = args.iter().find(|arg| arg.starts_with('-')) {
            return LaunchAction::Error(format!("unknown option: {option}"));
        }

        let requested_paths = args.into_iter().map(PathBuf::from).collect();
        LaunchAction::Run(Self { requested_paths })
    }
}

fn print_gui_help() {
    println!("Usage:");
    println!("  kfnotepad-gui [FILE ...]");
    println!("  kfnotepad-gui --describe");
    println!("  kfnotepad-gui --version");
    println!();
    println!("Opens local UTF-8 files as tiled GUI document panes.");
    println!("File open/save validation matches the terminal editor: regular UTF-8 files only.");
    println!(
        "Current controls: Ctrl-N {}, Ctrl-O {}, Ctrl-S {}, Ctrl-Shift-S {}, Ctrl-B {},",
        LABEL_NEW_TILE.to_ascii_lowercase(),
        LABEL_OPEN.to_ascii_lowercase(),
        LABEL_SAVE.to_ascii_lowercase(),
        LABEL_SAVE_AS.to_ascii_lowercase(),
        LABEL_FILES.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-F search, F3/Shift-F3 next/previous, Ctrl-G {}, Ctrl-T app theme,",
        LABEL_GO_TO_LINE.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-Shift-T syntax theme, Ctrl-R reader mode, Ctrl-M {}, Ctrl-Shift-M {},",
        LABEL_MINIMIZE.to_ascii_lowercase(),
        LABEL_MAXIMIZE.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-F4 {}, Ctrl-Q quit, Ctrl-Shift-arrow move tile.",
        LABEL_CLOSE_TILE.to_ascii_lowercase()
    );
    println!("Search is case-insensitive by default; toggle exact-case search in the Find row.");
    println!(
        "Reader mode auto-scrolls the active document at the speed configured in Preferences."
    );
    println!(
        "Preferences also configure app theme, syntax theme, wrapping, line numbers, and fonts."
    );
    println!("Path prompts resolve relative paths from the current file-browser directory.");
    println!(
        "Current browser control: Ctrl-B {}.",
        LABEL_FILES.to_ascii_lowercase(),
    );
}

fn print_gui_describe() {
    println!("kfnotepad-gui tiled Iced editor is available.");
    println!("Run `kfnotepad-gui FILE [FILE...]` to open editable GUI document panes.");
    println!(
        "Safe file behavior: UTF-8 regular files only, symlink/non-regular rejection, atomic saves, save-time external-change conflict checks."
    );
    println!("Layout: resizable tiled panes, compact icon chrome, collapsible left panel.");
    println!("Left panel: Files, Workspaces, and Preferences modes.");
    println!("Persistence: XDG config preferences, workspace projects, and geometry-only layout.");
    println!("Smoke: ./scripts/gui-visual-smoke.sh captures a nonblank local screenshot.");
    println!(
        "Current review gaps: manual desktop dialog coverage, live accessibility review, rich visual regression."
    );
}

struct KfnotepadGui {
    workspace: GuiWorkspace,
    panes: pane_grid::State<GuiPane>,
    active_pane: pane_grid::Pane,
    minimized_panes: Vec<GuiPane>,
    browser: Option<GuiFileBrowser>,
    browser_tree: Option<DirectoryTree>,
    browser_expanded_paths: HashSet<PathBuf>,
    browser_selected_path: Option<PathBuf>,
    browser_visible: bool,
    browser_width: f32,
    left_panel: GuiLeftPanelState,
    current_dir: PathBuf,
    notes_dir: Option<PathBuf>,
    workspace_projects_dir: Option<PathBuf>,
    workspace_projects: Vec<GuiWorkspaceProjectEntry>,
    workspace_project_name: String,
    pending_project_delete: Option<usize>,
    pending_browser_delete: Option<PathBuf>,
    #[cfg(test)]
    spawned_workspace_project_paths: Vec<PathBuf>,
    path_prompt: Option<GuiPathPrompt>,
    path_prompt_value: String,
    notes_panel: Option<Vec<ManagedNoteEntry>>,
    pending_managed_note_delete: Option<PathBuf>,
    file_snapshots: HashMap<GuiTileId, GuiFileSnapshot>,
    external_edit_locks: HashSet<GuiTileId>,
    syntax_caches: HashMap<GuiTileId, GuiSyntaxCache>,
    replacement_pointer_point: Option<(pane_grid::Pane, GuiEditorReplacementMousePoint)>,
    replacement_drag: Option<GuiEditorDragState>,
    replacement_drag_edge: Option<GuiEditorDragEdge>,
    replacement_scrollbar_drag: Option<GuiEditorScrollbarDrag>,
    replacement_scrollbar_pointer: Option<(pane_grid::Pane, f32, GuiEditorScrollbarModel)>,
    replacement_ime_preedit: Option<GuiImePreedit>,
    replacement_overwrite_mode: bool,
    pending_project_open: Option<usize>,
    pending_close_tile: Option<GuiTileId>,
    pending_app_quit: bool,
    search_query: String,
    search_history: Vec<String>,
    search_history_open: bool,
    search_highlight: Option<GuiSearchHighlight>,
    reader_scroll_accumulator: f32,
    go_to_line_query: String,
    syntax_highlighter: SyntaxHighlighter,
    settings: EditorSettings,
    config_path: Option<PathBuf>,
    layout_path: Option<PathBuf>,
    status_message: String,
}

struct GuiPane {
    tile_id: GuiTileId,
    editor: GuiEditorAdapter,
}

impl GuiPane {
    fn new(tile_id: GuiTileId, editor: text_editor::Content) -> Self {
        Self {
            tile_id,
            editor: GuiEditorAdapter::new(editor),
        }
    }
}

struct GuiMinimizedTrayItem {
    tile_id: GuiTileId,
    title: String,
    tooltip: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GuiFileSnapshot {
    len: u64,
    modified: Option<SystemTime>,
}

struct GuiSyntaxCache {
    path: PathBuf,
    line_count: usize,
    highlighted_until: usize,
    lines: Vec<Option<Vec<GuiEditorSyntaxSegment>>>,
    state: Option<SyntaxHighlightCacheState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiSearchHighlight {
    tile_id: GuiTileId,
    query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiImePreedit {
    tile_id: GuiTileId,
    content: String,
    selection: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiImeInputMethodRequest {
    visual_row: usize,
    cursor_column: usize,
    gutter_width: f32,
    character_width: f32,
    row_height: f32,
    preedit: Option<input_method::Preedit<String>>,
}

impl GuiImeInputMethodRequest {
    fn cursor_rect(&self, bounds: Rectangle) -> Rectangle {
        Rectangle::new(
            iced::Point::new(
                bounds.x + self.gutter_width + self.cursor_column as f32 * self.character_width,
                bounds.y + self.visual_row as f32 * self.row_height,
            ),
            Size::new(1.0, self.row_height),
        )
    }
}

struct GuiInputMethodArea<'a> {
    content: Element<'a, Message>,
    request: Option<GuiImeInputMethodRequest>,
}

impl<'a> GuiInputMethodArea<'a> {
    fn new(content: Element<'a, Message>, request: Option<GuiImeInputMethodRequest>) -> Self {
        Self { content, request }
    }
}

impl Widget<Message, Theme, iced::Renderer> for GuiInputMethodArea<'_> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &advanced_layout::Limits,
    ) -> advanced_layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: AdvancedLayout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn AdvancedOperation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: AdvancedLayout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn AdvancedClipboard,
        shell: &mut AdvancedShell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if let (Event::Window(window::Event::RedrawRequested(_)), Some(request)) =
            (event, &self.request)
        {
            shell.request_input_method(&input_method::InputMethod::Enabled {
                cursor: request.cursor_rect(layout.bounds()),
                purpose: input_method::Purpose::Normal,
                preedit: request.preedit.as_ref().map(input_method::Preedit::as_ref),
            });
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &advanced_renderer::Style,
        layout: AdvancedLayout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: AdvancedLayout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: AdvancedLayout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced_overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

struct GuiEditorAdapter {
    content: text_editor::Content,
    viewport: GuiEditorViewportState,
    viewport_tracks_cursor: bool,
    replacement_selection: Option<GuiEditorReplacementSelection>,
}

struct GuiEditorRenderState<'a> {
    content: &'a text_editor::Content,
    line_numbers: GuiEditorLineNumberSnapshot,
    #[cfg(test)]
    viewport_slice: GuiEditorViewportSlice,
}

struct GuiEditorSurfaceModel<'a> {
    content: &'a text_editor::Content,
    editor_font: Font,
    editor_size: u32,
    wrapping: Wrapping,
    syntax_token: String,
    highlighter_theme: highlighter::Theme,
    line_numbers: Option<GuiEditorLineNumberSnapshot>,
    viewport_slice: GuiEditorViewportSlice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorViewportState {
    first_line: usize,
    visible_lines: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorViewportLine {
    number: usize,
    text: String,
    cursor_column: Option<usize>,
    selection: Option<GuiEditorSelectionSpan>,
    syntax_segments: Option<Vec<GuiEditorSyntaxSegment>>,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorViewportSlice {
    line_count: usize,
    first_line: usize,
    lines: Vec<GuiEditorViewportLine>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiEditorReadOnlyRenderModel {
    line_count: usize,
    first_line: usize,
    gutter_text: String,
    body_text: String,
    cursor_row_in_view: Option<usize>,
    cursor_column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorReadOnlyLineSegment {
    text: String,
    selected: bool,
    syntax_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorReadOnlyVisualRow {
    line: GuiEditorViewportLine,
    viewport_row: usize,
    source_column_start: usize,
    show_line_number: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GuiEditorScrollbarModel {
    visible: bool,
    track_height: f32,
    thumb_top: f32,
    thumb_height: f32,
    page_delta: i32,
    visible_lines: usize,
    line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorDragState {
    pane: pane_grid::Pane,
    anchor: DocumentCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorDragEdge {
    pane: pane_grid::Pane,
    direction: i32,
    column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GuiEditorScrollbarDrag {
    pane: pane_grid::Pane,
    thumb_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GuiEditorBodyHitTest {
    columns: usize,
    visible_rows: usize,
    text_origin_x: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorSyntaxSegment {
    text: String,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorSelectionSpan {
    start_column: usize,
    end_column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorReplacementSelection {
    anchor: DocumentCursor,
    focus: DocumentCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuiEditorReplacementMousePoint {
    viewport_row: usize,
    column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiEditorReplacementInput {
    InsertChar(char),
    InsertNewline,
    DeleteBackward,
    DeleteForward,
    DeletePreviousWord,
    DeleteNextWord,
    DeleteToLineEnd,
    Move(kfnotepad::CursorMove),
    MoveLineStart,
    MoveLineEnd,
    ScrollViewportLines(i32),
    SelectAll,
    #[allow(dead_code)]
    SelectRange {
        anchor: DocumentCursor,
        focus: DocumentCursor,
    },
    ClearSelection,
}

enum GuiEditorCommand {
    IcedAction(text_editor::Action),
    Delete,
    MoveTo(DocumentCursor),
    Paste(String),
    ScrollViewportLines(i32),
    SelectAll,
    SelectRightChars(usize),
}

fn gui_editor_command_invalidates_syntax(command: &GuiEditorCommand) -> bool {
    match command {
        GuiEditorCommand::IcedAction(action) => action.is_edit(),
        GuiEditorCommand::Delete | GuiEditorCommand::Paste(_) => true,
        GuiEditorCommand::MoveTo(_)
        | GuiEditorCommand::ScrollViewportLines(_)
        | GuiEditorCommand::SelectAll
        | GuiEditorCommand::SelectRightChars(_) => false,
    }
}

fn gui_editor_command_may_extend_syntax_cache(command: &GuiEditorCommand) -> bool {
    matches!(
        command,
        GuiEditorCommand::MoveTo(_)
            | GuiEditorCommand::ScrollViewportLines(_)
            | GuiEditorCommand::IcedAction(text_editor::Action::Scroll { .. })
    )
}

fn gui_replacement_inputs_invalidate_syntax(inputs: &[GuiEditorReplacementInput]) -> bool {
    inputs.iter().any(|input| {
        matches!(
            input,
            GuiEditorReplacementInput::InsertChar(_)
                | GuiEditorReplacementInput::InsertNewline
                | GuiEditorReplacementInput::DeleteBackward
                | GuiEditorReplacementInput::DeleteForward
                | GuiEditorReplacementInput::DeletePreviousWord
                | GuiEditorReplacementInput::DeleteNextWord
                | GuiEditorReplacementInput::DeleteToLineEnd
        )
    })
}

impl GuiEditorAdapter {
    fn new(content: text_editor::Content) -> Self {
        let mut adapter = Self {
            content,
            viewport: GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            viewport_tracks_cursor: true,
            replacement_selection: None,
        };
        adapter.sync_viewport_to_cursor();
        adapter
    }

    #[cfg(test)]
    fn from_text(text: &str) -> Self {
        Self::new(text_editor::Content::with_text(text))
    }

    fn text(&self) -> String {
        self.content.text()
    }

    fn clone_for_relayout(&self) -> Self {
        let mut adapter = Self::new(text_editor::Content::with_text(&self.text()));
        adapter.content.move_to(self.cursor());
        adapter.viewport = self.viewport;
        adapter.viewport_tracks_cursor = self.viewport_tracks_cursor;
        adapter.replacement_selection = self.replacement_selection;
        adapter
    }

    fn cursor(&self) -> text_editor::Cursor {
        self.content.cursor()
    }

    fn document_cursor(&self) -> DocumentCursor {
        document_cursor_from_editor(self.cursor())
    }

    fn selection(&self) -> Option<String> {
        if let Some(selection) = self.replacement_selection {
            let document = TextDocument {
                path: PathBuf::from("replacement-selection.txt"),
                buffer: TextBuffer::from_text(&self.text()),
            };
            return gui_editor_replacement_copy_selection(&document, Some(selection));
        }
        self.content.selection()
    }

    fn line_count(&self) -> usize {
        self.content.line_count()
    }

    fn apply(&mut self, command: GuiEditorCommand) {
        if GUI_USE_READ_ONLY_EDITOR_RENDERER {
            self.apply_replacement_command(command);
            return;
        }

        let sync_viewport_to_cursor = match command {
            GuiEditorCommand::IcedAction(action) => {
                let scroll_lines = match action {
                    text_editor::Action::Scroll { lines } => Some(lines),
                    _ => None,
                };
                self.content.perform(action);
                if let Some(lines) = scroll_lines {
                    self.scroll_viewport_by_lines(lines);
                    false
                } else {
                    true
                }
            }
            GuiEditorCommand::Delete => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                true
            }
            GuiEditorCommand::MoveTo(cursor) => {
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::Right));
                self.content.move_to(editor_cursor_from_document(cursor));
                true
            }
            GuiEditorCommand::Paste(contents) => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(contents),
                    )));
                true
            }
            GuiEditorCommand::ScrollViewportLines(delta) => {
                self.scroll_viewport_by_lines(delta);
                false
            }
            GuiEditorCommand::SelectAll => {
                self.content.perform(text_editor::Action::SelectAll);
                true
            }
            GuiEditorCommand::SelectRightChars(count) => {
                for _ in 0..count {
                    self.content
                        .perform(text_editor::Action::Select(text_editor::Motion::Right));
                }
                true
            }
        };
        if sync_viewport_to_cursor {
            self.sync_viewport_to_cursor();
        }
    }

    fn apply_replacement_command(&mut self, command: GuiEditorCommand) {
        match command {
            GuiEditorCommand::IcedAction(action) => match action {
                text_editor::Action::Scroll { lines } => {
                    self.scroll_viewport_by_lines(lines);
                }
                text_editor::Action::Move(motion) => {
                    self.apply_text_editor_motion_to_replacement(motion);
                }
                _ => {
                    self.content.perform(action);
                    self.sync_viewport_to_cursor();
                }
            },
            GuiEditorCommand::Delete => {
                let mut document = TextDocument {
                    path: PathBuf::from("replacement-delete.txt"),
                    buffer: TextBuffer::from_text(&self.text()),
                };
                let mut cursor = self.document_cursor();
                let mut selection = self.replacement_selection;
                let mut viewport = self.viewport;
                apply_gui_editor_replacement_input(
                    &mut document,
                    &mut cursor,
                    &mut viewport,
                    &mut selection,
                    GuiEditorReplacementInput::DeleteForward,
                );
                self.content = text_editor::Content::with_text(&document.buffer.to_text());
                self.content.move_to(editor_cursor_from_document(cursor));
                self.viewport = viewport;
                self.viewport_tracks_cursor = true;
                self.replacement_selection = selection;
            }
            GuiEditorCommand::MoveTo(cursor) => {
                self.content.move_to(editor_cursor_from_document(cursor));
                self.replacement_selection = None;
                self.sync_viewport_to_cursor();
            }
            GuiEditorCommand::Paste(contents) => {
                let mut document = TextDocument {
                    path: PathBuf::from("replacement-paste.txt"),
                    buffer: TextBuffer::from_text(&self.text()),
                };
                let mut cursor = self.document_cursor();
                let mut selection = self.replacement_selection;
                let mut viewport = self.viewport;
                gui_editor_replacement_paste_text(
                    &mut document,
                    &mut cursor,
                    &mut viewport,
                    &mut selection,
                    &contents,
                );
                self.content = text_editor::Content::with_text(&document.buffer.to_text());
                self.content.move_to(editor_cursor_from_document(cursor));
                self.viewport = viewport;
                self.viewport_tracks_cursor = true;
                self.replacement_selection = selection;
            }
            GuiEditorCommand::ScrollViewportLines(delta) => {
                self.scroll_viewport_by_lines(delta);
            }
            GuiEditorCommand::SelectAll => {
                let document = TextDocument {
                    path: PathBuf::from("replacement-select-all.txt"),
                    buffer: TextBuffer::from_text(&self.text()),
                };
                let start = DocumentCursor { row: 0, column: 0 };
                let end = gui_editor_replacement_document_end_cursor(&document.buffer);
                self.content.move_to(editor_cursor_from_document(end));
                self.replacement_selection = GuiEditorReplacementSelection::new(start, end);
                self.sync_viewport_to_cursor();
            }
            GuiEditorCommand::SelectRightChars(count) => {
                let document = TextDocument {
                    path: PathBuf::from("replacement-select-right.txt"),
                    buffer: TextBuffer::from_text(&self.text()),
                };
                let start = self.document_cursor();
                let max_columns = document
                    .buffer
                    .line_char_count(start.row)
                    .unwrap_or(start.column);
                let focus = DocumentCursor {
                    row: start.row,
                    column: start.column.saturating_add(count).min(max_columns),
                };
                self.content.move_to(editor_cursor_from_document(start));
                self.replacement_selection = GuiEditorReplacementSelection::new(start, focus);
                self.sync_viewport_to_cursor();
            }
        }
    }

    fn apply_text_editor_motion_to_replacement(&mut self, motion: text_editor::Motion) {
        let mut document = TextDocument {
            path: PathBuf::from("replacement-motion.txt"),
            buffer: TextBuffer::from_text(&self.text()),
        };
        let mut cursor = self.document_cursor();
        let mut selection = self.replacement_selection;
        let mut viewport = self.viewport;

        match motion {
            text_editor::Motion::Left => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Left),
            ),
            text_editor::Motion::Right => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
            ),
            text_editor::Motion::Up => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Up),
            ),
            text_editor::Motion::Down => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down),
            ),
            text_editor::Motion::WordLeft => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::WordLeft),
            ),
            text_editor::Motion::WordRight => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::Move(kfnotepad::CursorMove::WordRight),
            ),
            text_editor::Motion::Home => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::MoveLineStart,
            ),
            text_editor::Motion::End => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::MoveLineEnd,
            ),
            text_editor::Motion::PageUp => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::ScrollViewportLines(
                    -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
                ),
            ),
            text_editor::Motion::PageDown => apply_gui_editor_replacement_input(
                &mut document,
                &mut cursor,
                &mut viewport,
                &mut selection,
                GuiEditorReplacementInput::ScrollViewportLines(
                    GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
                ),
            ),
            text_editor::Motion::DocumentStart => {
                selection = None;
                go_to_document_start(&mut cursor);
                viewport.keep_cursor_visible(cursor, document.buffer.line_count());
            }
            text_editor::Motion::DocumentEnd => {
                selection = None;
                go_to_document_end(&document, &mut cursor);
                viewport.keep_cursor_visible(cursor, document.buffer.line_count());
            }
        }

        self.content.move_to(editor_cursor_from_document(cursor));
        self.viewport = viewport;
        self.viewport_tracks_cursor = true;
        self.replacement_selection = selection;
    }

    fn move_to(&mut self, cursor: DocumentCursor) {
        self.apply(GuiEditorCommand::MoveTo(cursor));
    }

    fn select_right_chars(&mut self, count: usize) {
        self.apply(GuiEditorCommand::SelectRightChars(count));
    }

    fn scroll_viewport_by_lines(&mut self, delta: i32) {
        let line_count = self.line_count();
        self.viewport.scroll_by(delta, line_count);
        let cursor = self.document_cursor();
        let visible_cursor = self.viewport.clamp_cursor_to_visible(cursor, line_count);
        if visible_cursor != cursor {
            self.content
                .move_to(editor_cursor_from_document(visible_cursor));
        }
        self.viewport_tracks_cursor = true;
    }

    fn scroll_viewport_by_lines_preserving_cursor(&mut self, delta: i32) {
        let line_count = self.line_count();
        self.viewport.scroll_by(delta, line_count);
        self.viewport_tracks_cursor = false;
    }

    fn render_state(
        &self,
        visible_line_numbers: usize,
        editor_font_size: u16,
    ) -> GuiEditorRenderState<'_> {
        GuiEditorRenderState {
            content: &self.content,
            line_numbers: self.line_number_snapshot(visible_line_numbers, editor_font_size),
            #[cfg(test)]
            viewport_slice: self.viewport_slice(visible_line_numbers),
        }
    }

    fn line_number_snapshot(
        &self,
        visible_lines: usize,
        editor_font_size: u16,
    ) -> GuiEditorLineNumberSnapshot {
        let line_count = self.line_count();
        let viewport = self.viewport.with_visible_lines(visible_lines, line_count);
        let viewport = if self.viewport_tracks_cursor {
            viewport.with_cursor_visible_for_render(self.document_cursor(), line_count)
        } else {
            viewport
        };
        GuiEditorLineNumberSnapshot {
            line_count,
            gutter_start: viewport.first_line,
            text: gui_line_number_gutter_text(viewport.first_line, line_count, visible_lines),
            width: gui_line_number_gutter_width(line_count, editor_font_size),
        }
    }

    fn sync_viewport_to_cursor(&mut self) {
        self.viewport
            .keep_cursor_visible(self.document_cursor(), self.line_count());
        self.viewport_tracks_cursor = true;
    }

    #[cfg(test)]
    fn viewport_slice(&self, visible_lines: usize) -> GuiEditorViewportSlice {
        let line_count = self.line_count();
        let cursor = self.document_cursor();
        let viewport = self.viewport.with_visible_lines(visible_lines, line_count);
        let viewport = if self.viewport_tracks_cursor {
            viewport.with_cursor_visible_for_render(cursor, line_count)
        } else {
            viewport
        };
        gui_editor_viewport_slice(
            &self.text(),
            line_count,
            viewport,
            cursor,
            self.replacement_selection,
        )
    }

    fn render_viewport_slice_from_lines(
        &self,
        document_lines: &[String],
        visible_lines: usize,
    ) -> GuiEditorViewportSlice {
        let line_count = self.line_count();
        let total = line_count.max(1);
        let mut viewport = self.viewport;
        viewport.visible_lines = visible_lines.max(1);
        viewport.first_line = viewport.first_line.clamp(1, total);
        if self.viewport_tracks_cursor {
            viewport = viewport.with_cursor_visible_for_render(self.document_cursor(), line_count);
        }
        gui_editor_viewport_slice_from_lines(
            document_lines,
            line_count,
            viewport,
            self.document_cursor(),
            self.replacement_selection,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GuiEditorLineNumberSnapshot {
    line_count: usize,
    gutter_start: usize,
    text: String,
    width: f32,
}

impl GuiEditorViewportState {
    fn new(visible_lines: usize) -> Self {
        Self {
            first_line: 1,
            visible_lines: visible_lines.max(1),
        }
    }

    fn with_visible_lines(mut self, visible_lines: usize, line_count: usize) -> Self {
        self.visible_lines = visible_lines.max(1);
        self.clamp_to_line_count(line_count);
        self
    }

    fn with_cursor_visible_for_render(mut self, cursor: DocumentCursor, line_count: usize) -> Self {
        let total = line_count.max(1);
        self.first_line = self.first_line.clamp(1, total);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let last_visible = self
            .first_line
            .saturating_add(self.visible_lines.saturating_sub(1));
        if cursor_line < self.first_line {
            self.first_line = cursor_line;
        } else if cursor_line > last_visible {
            self.first_line = cursor_line.saturating_sub(self.visible_lines.saturating_sub(1));
        }
        self.first_line = self.first_line.clamp(1, total);
        self
    }

    fn scroll_by(&mut self, delta: i32, line_count: usize) {
        let current = self.first_line as i64;
        let next = current + i64::from(delta);
        self.first_line = next.max(1) as usize;
        self.clamp_to_line_count(line_count);
    }

    fn keep_cursor_visible(&mut self, cursor: DocumentCursor, line_count: usize) {
        self.clamp_to_line_count(line_count);
        let total = line_count.max(1);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let last_visible = self
            .first_line
            .saturating_add(self.visible_lines.saturating_sub(1));

        if cursor_line < self.first_line {
            self.first_line = cursor_line;
        } else if cursor_line > last_visible {
            self.first_line = cursor_line.saturating_sub(self.visible_lines.saturating_sub(1));
        }

        self.clamp_to_line_count(line_count);
    }

    fn clamp_cursor_to_visible(&self, cursor: DocumentCursor, line_count: usize) -> DocumentCursor {
        let total = line_count.max(1);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let first = self.first_line.clamp(1, total);
        let last = self.last_visible_line(line_count);
        let clamped_line = cursor_line.clamp(first, last);

        DocumentCursor {
            row: clamped_line.saturating_sub(1),
            column: cursor.column,
        }
    }

    fn last_visible_line(&self, line_count: usize) -> usize {
        let total = line_count.max(1);
        self.first_line
            .saturating_add(self.visible_lines.saturating_sub(1))
            .min(total)
    }

    fn clamp_to_line_count(&mut self, line_count: usize) {
        let total = line_count.max(1);
        let max_first = total
            .saturating_sub(self.visible_lines.saturating_sub(1))
            .max(1);
        self.first_line = self.first_line.clamp(1, max_first);
    }
}

fn gui_editor_scrollbar_model(
    line_count: usize,
    first_line: usize,
    visible_lines: usize,
    track_height: f32,
) -> GuiEditorScrollbarModel {
    let visible_lines = visible_lines.max(1);
    let line_count = line_count.max(1);
    let track_height = track_height.max(1.0);
    let page_delta = visible_lines.min(i32::MAX as usize) as i32;

    if line_count <= visible_lines {
        return GuiEditorScrollbarModel {
            visible: false,
            track_height,
            thumb_top: 0.0,
            thumb_height: track_height,
            page_delta,
            visible_lines,
            line_count,
        };
    }

    let max_first = line_count
        .saturating_sub(visible_lines.saturating_sub(1))
        .max(1);
    let clamped_first = first_line.clamp(1, max_first);
    let proportional_thumb = track_height * (visible_lines as f32 / line_count as f32);
    let thumb_height = proportional_thumb
        .max(GUI_EDITOR_SCROLLBAR_THUMB_MIN_HEIGHT)
        .min(track_height);
    let travel = (track_height - thumb_height).max(0.0);
    let progress = if max_first <= 1 {
        0.0
    } else {
        (clamped_first.saturating_sub(1) as f32 / max_first.saturating_sub(1) as f32)
            .clamp(0.0, 1.0)
    };

    GuiEditorScrollbarModel {
        visible: true,
        track_height,
        thumb_top: travel * progress,
        thumb_height,
        page_delta,
        visible_lines,
        line_count,
    }
}

fn gui_editor_scrollbar_first_line_from_thumb_y(
    model: GuiEditorScrollbarModel,
    y: f32,
    thumb_offset: f32,
) -> usize {
    let line_count = model.line_count.max(1);
    let visible_lines = model.visible_lines.max(1);
    let max_first = line_count
        .saturating_sub(visible_lines.saturating_sub(1))
        .max(1);
    if !model.visible || max_first <= 1 {
        return 1;
    }

    let travel = (model.track_height - model.thumb_height).max(1.0);
    let thumb_top = (y - thumb_offset).clamp(0.0, travel);
    let progress = (thumb_top / travel).clamp(0.0, 1.0);
    1usize
        .saturating_add((progress * max_first.saturating_sub(1) as f32).round() as usize)
        .clamp(1, max_first)
}

fn gui_editor_scrollbar_press_target(
    model: GuiEditorScrollbarModel,
    y: f32,
) -> GuiEditorScrollbarPressTarget {
    if !model.visible {
        return GuiEditorScrollbarPressTarget::None;
    }
    if y < model.thumb_top {
        GuiEditorScrollbarPressTarget::Page(-(model.page_delta.max(1)))
    } else if y <= model.thumb_top + model.thumb_height {
        GuiEditorScrollbarPressTarget::Thumb {
            offset: (y - model.thumb_top).clamp(0.0, model.thumb_height.max(1.0)),
        }
    } else {
        GuiEditorScrollbarPressTarget::Page(model.page_delta.max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GuiEditorScrollbarPressTarget {
    None,
    Page(i32),
    Thumb { offset: f32 },
}

impl GuiEditorReplacementSelection {
    fn new(anchor: DocumentCursor, focus: DocumentCursor) -> Option<Self> {
        (anchor != focus).then_some(Self { anchor, focus })
    }

    fn normalized(self) -> (DocumentCursor, DocumentCursor) {
        if document_cursor_is_before_or_equal(self.anchor, self.focus) {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiMenuGroup {
    File,
    Edit,
    View,
    Go,
    Notes,
    Tile,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiPathPrompt {
    Open,
    SaveAs,
    ManagedNote,
    BrowserCreateFile,
    BrowserCreateDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiHeaderLayoutMode {
    SingleRow,
    SplitActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiSearchLayoutMode {
    SingleRow,
    SplitRows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiMenuCommand {
    NewTile,
    Open,
    OpenPath,
    Save,
    SaveAs,
    SaveAsPath,
    ClosePane,
    Quit,
    OpenManagedNote,
    ListManagedNotes,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    SelectAll,
    FindNext,
    FindPrevious,
    ToggleBrowser,
    CycleTheme,
    CycleSyntaxTheme,
    ToggleReaderMode,
    GoDocumentStart,
    GoDocumentEnd,
    GoToLine,
    ToggleMinimize,
    ToggleMaximize,
    EqualizeTiles,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    OpenHelp,
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct GuiActionDescriptor {
    label: &'static str,
    shortcut: Option<&'static str>,
    menu_group: Option<GuiMenuGroup>,
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct GuiFocusStep {
    area: &'static str,
    label: &'static str,
    keyboard: Option<&'static str>,
}

struct GuiMenuItem {
    label: &'static str,
    command: GuiMenuCommand,
}

const LABEL_SAVE: &str = "Save";
const LABEL_SAVE_AS: &str = "Save as";
const LABEL_SAVE_AS_PATH: &str = "Save as path";
const LABEL_OPEN: &str = "Open";
const LABEL_OPEN_PATH: &str = "Open path";
const LABEL_NEW_TILE: &str = "New tile";
const LABEL_CLOSE_TILE: &str = "Close tile";
const LABEL_QUIT: &str = "Quit";
const LABEL_OPEN_NOTE: &str = "Open note";
const LABEL_LIST_NOTES: &str = "List notes";
const LABEL_COPY: &str = "Copy";
const LABEL_CUT: &str = "Cut";
const LABEL_PASTE: &str = "Paste";
const LABEL_UNDO: &str = "Undo";
const LABEL_REDO: &str = "Redo";
const LABEL_SELECT_ALL: &str = "Select all";
const LABEL_FIND_NEXT: &str = "Find next";
const LABEL_FIND_PREVIOUS: &str = "Find previous";
const LABEL_FILES: &str = "Files";
const LABEL_WORKSPACES: &str = "Workspaces";
const LABEL_PREFERENCES: &str = "Preferences";
#[cfg(test)]
const LABEL_REFRESH: &str = "Refresh";
const LABEL_CREATE_FILE: &str = "Create file";
const LABEL_THEME: &str = "Theme";
const LABEL_SYNTAX_THEME: &str = "Syntax theme";
const LABEL_READER_MODE: &str = "Reader mode";
const LABEL_GO_TO_LINE: &str = "Go to line";
const LABEL_DOCUMENT_START: &str = "Document start";
const LABEL_DOCUMENT_END: &str = "Document end";
const LABEL_MINIMIZE: &str = "Minimize";
const LABEL_RESTORE: &str = "Restore";
const LABEL_MAXIMIZE: &str = "Maximize";
const LABEL_UNLOCK_EXTERNAL_EDIT: &str = "Unlock external edit lock";
const LABEL_EQUALIZE_TILES: &str = "Equalize tiles";
const LABEL_MOVE_LEFT: &str = "Move left";
const LABEL_MOVE_RIGHT: &str = "Move right";
const LABEL_MOVE_UP: &str = "Move up";
const LABEL_MOVE_DOWN: &str = "Move down";
const LABEL_OPEN_HELP: &str = "Open help";
const LABEL_GO: &str = "Apply";
const LABEL_CANCEL: &str = "Cancel";
#[cfg(test)]
const ICON_VIEW_MENU: &str = nf::cod::COD_EYE;
const ICON_NEW_TILE: &str = nf::fa::FA_PLUS;
const ICON_SAVE: &str = nf::cod::COD_SAVE;
const ICON_FILES: &str = nf::fa::FA_FOLDER;
const ICON_WORKSPACES: &str = nf::cod::COD_MULTIPLE_WINDOWS;
const ICON_PREFERENCES: &str = nf::cod::COD_SETTINGS_GEAR;
const ICON_THEME: &str = nf::cod::COD_SYMBOL_COLOR;
const ICON_SYNTAX_THEME: &str = nf::cod::COD_BRACKET_DOT;
const ICON_CASE_SENSITIVE: &str = nf::cod::COD_CASE_SENSITIVE;
const ICON_READER_MODE_PLAY: &str = nf::fa::FA_BOOK_OPEN_READER;
const ICON_READER_MODE_PAUSE: &str = nf::fa::FA_PAUSE;
const ICON_NEW_WINDOW: &str = nf::md::MD_WINDOW_MAXIMIZE;
const ICON_DELETE: &str = nf::fa::FA_TRASH;
const ICON_REFRESH: &str = nf::cod::COD_REFRESH;
const ICON_CREATE_FILE: &str = nf::cod::COD_NEW_FILE;
const ICON_CREATE_DIRECTORY: &str = nf::cod::COD_NEW_FOLDER;
const ICON_PARENT_DIR: &str = nf::cod::COD_ARROW_UP;
const ICON_FIND_PREVIOUS: &str = nf::cod::COD_ARROW_LEFT;
const ICON_FIND_NEXT: &str = nf::cod::COD_ARROW_RIGHT;
const ICON_GO_TO_LINE: &str = nf::cod::COD_DEBUG_LINE_BY_LINE;
const ICON_DOCUMENT_START: &str = nf::oct::OCT_HOME;
const ICON_DOCUMENT_END: &str = nf::oct::OCT_MOVE_TO_END;
const ICON_MOVE_LEFT: &str = nf::cod::COD_ARROW_SMALL_LEFT;
const ICON_MOVE_RIGHT: &str = nf::cod::COD_ARROW_SMALL_RIGHT;
const ICON_MOVE_UP: &str = nf::cod::COD_ARROW_SMALL_UP;
const ICON_MOVE_DOWN: &str = nf::cod::COD_ARROW_SMALL_DOWN;
const ICON_MINIMIZE: &str = nf::fa::FA_WINDOW_MINIMIZE;
const ICON_RESTORE: &str = nf::fa::FA_WINDOW_RESTORE;
const ICON_MAXIMIZE: &str = nf::fa::FA_WINDOW_MAXIMIZE;
const ICON_CLOSE: &str = nf::cod::COD_CHROME_CLOSE;
const ICON_UNLOCK: &str = nf::cod::COD_UNLOCK;
const WORKSPACE_PROJECT_ENV: &str = "KFNOTEPAD_GUI_WORKSPACE_PROJECT";
const GUI_BROWSER_WIDTH_DEFAULT: f32 = 220.0;
const GUI_BROWSER_WIDTH_MIN: f32 = 160.0;
const GUI_BROWSER_WIDTH_MAX: f32 = 360.0;
const GUI_PANE_GRID_SPACING: f32 = 5.0;
const GUI_PANE_GRID_MIN_SIZE: f32 = 120.0;
const GUI_PANE_GRID_REFERENCE_SIZE: Size = Size {
    width: 1000.0,
    height: 800.0,
};
const GUI_MENU_DROPDOWN_WIDTH: f32 = 190.0;
const GUI_MENU_DROPDOWN_RADIUS: f32 = 6.0;
const GUI_MENU_ITEM_RADIUS: f32 = 4.0;
const GUI_MENU_ROOT_HORIZONTAL_PADDING: f32 = 3.0;
const GUI_MENU_ROOT_VERTICAL_PADDING: f32 = 1.0;
const GUI_MENU_ROOT_HEIGHT: f32 = 24.0;
const GUI_MENU_BAR_SPACING: u32 = 1;
const GUI_HEADER_ACTION_SPACING: u32 = 3;
const GUI_HEADER_GROUP_SPACING: u32 = 6;
const GUI_HEADER_SPLIT_SPACING: u32 = 3;
const GUI_MENU_ITEM_PADDING: [u16; 2] = [3, 5];
const GUI_CHROME_PADDING: [u16; 2] = [2, 3];
const GUI_ICON_BUTTON_SIDE: f32 = 22.0;
const GUI_TILE_CONTROL_BUTTON_SIDE: f32 = 24.0;
const GUI_ICON_FONT_NAME: &str = "Symbols Nerd Font Mono";
const GUI_ICON_LINE_HEIGHT: f32 = 1.0;
const GUI_FIND_HISTORY_LIMIT: usize = 10;
const GUI_READER_TICK_MS: u64 = 500;
const GUI_HELP_DOCUMENT_PATH: &str = "kfnotepad-help.md";
const GUI_ROOT_PADDING: u16 = 8;
const GUI_PANEL_PADDING_LEFT: f32 = 2.0;
const GUI_PANEL_PADDING_RIGHT: f32 = 4.0;
const GUI_PANEL_PADDING_VERTICAL: f32 = 6.0;
const GUI_PANEL_CONTROL_SPACING: u32 = 5;
const GUI_PANEL_SECTION_SPACING: u32 = 6;
const GUI_PANEL_PATH_MAX_CHARS: usize = 34;
const GUI_PANEL_TREE_TOP_PADDING: f32 = 4.0;
const GUI_FILE_TREE_INDENT: f32 = 14.0;
const GUI_FILE_TREE_ROW_SPACING: u32 = 2;
const GUI_FILE_TREE_MAX_DEPTH: usize = 8;
const GUI_TILE_BODY_PADDING: u16 = 2;
const GUI_TILE_TITLE_PADDING: u16 = 3;
const GUI_TILE_CONTROL_SPACING: u32 = 1;
const GUI_EDITOR_PADDING: u16 = 2;
const GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES: usize = 32;
const GUI_EDITOR_RENDER_LINE_BUDGET: usize = 512;
const GUI_LINE_NUMBER_GUTTER_HORIZONTAL_PADDING: f32 = 6.0;
const GUI_EDITOR_LINE_HEIGHT: f32 = 1.3;
const GUI_LINE_NUMBER_SEPARATOR_WIDTH: f32 = 1.0;
const GUI_TAB_WIDTH: usize = 4;
const GUI_EDITOR_SCROLLBAR_WIDTH: f32 = 6.0;
const GUI_EDITOR_SCROLLBAR_THUMB_MIN_HEIGHT: f32 = 18.0;
const GUI_REPLACEMENT_DRAG_TICK_MS: u64 = 40;
const GUI_USE_READ_ONLY_EDITOR_RENDERER: bool = true;
const GUI_TILE_RADIUS: f32 = 3.0;
const GUI_HEADER_SPLIT_WIDTH: f32 = 1180.0;
const GUI_SEARCH_SPLIT_WIDTH: f32 = 760.0;
const GUI_HELP_DOCUMENT_TEXT: &str = r#"# kfnotepad help

kfnotepad is a local UTF-8 text-file editor. The terminal editor and Iced GUI both edit normal files on disk; there is no database, account, sync service, or autosave.

## Files

- Use the Files panel to browse from the current working directory.
- Single-click a file or directory to select it.
- Double-click a file to open it in a tile.
- Double-click a directory to make it the active tree location.
- Use the parent and refresh buttons to move up or reload the tree.
- Create file and create folder use the selected directory when a directory is selected; otherwise they use the current Files root.
- Delete selected requires confirmation. Directory deletion warns because nested files and folders are removed too.
- Opening a file that is already open focuses or restores the existing tile instead of opening a duplicate.

## Tiles

- File > New tile or Ctrl-N creates an untitled document tile.
- File > Open or Ctrl-O opens local files.
- File > Save or Ctrl-S saves the active tile.
- File > Close tile or Ctrl-F4 closes the active tile. Unsaved tiles ask for confirmation.
- Tile > Equalize tiles arranges open tiles into an even grid.
- Tile controls can move, minimize, maximize, restore, or close a tile. Hover a tile titlebar to show those controls.
- Minimized tiles move to the tray below the menu and can be restored from there.
- Ctrl-Shift-Arrow moves the active tile. Ctrl-M minimizes/restores it. Ctrl-Shift-M maximizes/restores it.

## Editing

- Type normally in the active tile.
- Ctrl-Z and Ctrl-Y undo and redo.
- Ctrl-C, Ctrl-X, Ctrl-V copy, cut, and paste selected text.
- Ctrl-A selects all text.
- Insert toggles overwrite mode.
- Home and End move within a line; Ctrl-Home and Ctrl-End move to the document edges.
- Ctrl-Left and Ctrl-Right move by word.
- Ctrl-Backspace and Ctrl-Delete delete by word.
- Ctrl-K deletes to the end of the current line.
- PageUp and PageDown move the active document viewport.
- Mouse click moves the cursor; drag selects text; mouse wheel scrolls the hovered tile.
- IME committed text is supported, and preedit text is shown transiently at the cursor.

## Search and navigation

- Ctrl-F focuses Find.
- Search is case-insensitive by default.
- Use the exact-case toggle in the Find row or Preferences to make search case-sensitive.
- F3 finds the next match; Shift-F3 finds the previous match.
- Recent Find queries are kept in a session-only history dropdown when the Find field is empty.
- Ctrl-G focuses Go to line.
- The Nav menu can jump to the top or bottom of the active document.
- The active search match is selected and highlighted in the editor.

## Reader mode

- Ctrl-R, View > Reader mode, the header reader button, or Preferences toggles reader mode.
- Reader mode auto-scrolls the active document down without editing or saving it.
- Reader speed is configured in Preferences as lines per minute.
- Reader mode stops at the end of the document.

## Themes and syntax

- Ctrl-T cycles the app theme.
- Ctrl-Shift-T cycles the syntax highlighting theme independently from the app theme.
- Preferences can also cycle syntax theme and configure line numbers, wrapping, editor font, editor size, UI size, exact-case search, reader mode, reader speed, and restore-last-workspace.
- Built-in app and syntax theme names are Nocturne, Aurora, Pastel, Terminal, Abyss, and Terror.
- Syntax colors are adjusted for readable contrast against the selected app theme.

## Workspaces and preferences

- The Workspaces panel can save and reopen a group of open tiles and layout.
- Save current updates the deterministic current workspace used by restore-last-workspace.
- Save named stores a project under the entered name.
- Saved projects can open in the current window or in a new kfnotepad GUI process.
- Deleting a saved project requires confirmation.
- Restore last workspace is opt-in from Preferences. When enabled, argument-free startup reopens the latest saved current workspace when it is still valid.

## Managed notes

- Notes > Open note creates or opens a managed Markdown note.
- Notes > List notes shows existing managed notes.
- Notes can be deleted from the list after confirmation.
- Managed notes are normal `.md` files under the local XDG data directory.

## External changes

If an already-open clean file changes on disk, kfnotepad refreshes the tile and locks editing to avoid overwriting the outside change. Locked tiles still allow scrolling and further external refreshes. Use the unlock button in the tile titlebar when you are ready to edit locally again. Dirty local buffers are not overwritten by external refresh.

## Saving and safety

- Save uses the same atomic local-file adapter as the terminal editor.
- Open rejects missing files, directories, symlinks, non-UTF-8 files, and files larger than 8 MiB.
- Save rejects symlink targets and preserves existing file permissions where possible.
- Unsaved buffers remain in memory until saved.
"#;

#[derive(Debug, Clone)]
enum Message {
    Edit(pane_grid::Pane, text_editor::Action),
    BrowserTreeEvent(DirectoryTreeEvent),
    BrowserLocalTreeToggle(PathBuf),
    BrowserLocalTreeSelected(PathBuf, bool),
    BrowserLocalTreeActivated(PathBuf, bool),
    BrowserParentRequested,
    BrowserRefreshRequested,
    BrowserCreateFileRequested,
    BrowserCreateDirectoryRequested,
    BrowserDeleteSelectedRequested,
    BrowserWidthChanged(f32),
    SelectLeftPanelMode(GuiLeftPanelMode),
    PaneClicked(pane_grid::Pane),
    PaneResized(pane_grid::ResizeEvent),
    PaneDragged(pane_grid::DragEvent),
    NewTileRequested,
    OpenPromptRequested,
    OpenDialogSelected(Option<PathBuf>),
    SaveAsPromptRequested,
    SaveAsDialogSelected(Option<PathBuf>),
    ManagedNoteClicked(usize),
    ManagedNoteDeleteClicked(usize),
    ExternalFileCheckTick,
    ReaderScrollTick,
    UnlockExternalEdit(GuiTileId),
    WorkspaceProjectClicked(usize),
    WorkspaceProjectNewWindowClicked(usize),
    WorkspaceProjectDeleteClicked(usize),
    SaveCurrentWorkspaceProject,
    WorkspaceProjectNameChanged(String),
    SaveNamedWorkspaceProject,
    RestoreLastWorkspaceChanged(bool),
    ShowLineNumbersChanged(bool),
    WrapLinesChanged(bool),
    SearchCaseSensitiveChanged(bool),
    ReaderModeChanged(bool),
    ReaderSpeedChanged(u16),
    CycleGuiFontFamily,
    GuiFontSizeChanged(u16),
    GuiUiFontSizeChanged(u16),
    RefreshWorkspaceProjects,
    PathPromptChanged(String),
    SubmitPathPrompt,
    CancelPathPrompt,
    SaveRequested,
    ToggleBrowser,
    ClosePane(pane_grid::Pane),
    CloseActivePane,
    QuitRequested(window::Id),
    ToggleMinimizePane(pane_grid::Pane),
    RestoreMinimizedTile(GuiTileId),
    ToggleActiveMinimize,
    ToggleActiveMaximize,
    ToggleMaximizePane(pane_grid::Pane),
    MoveActivePane(pane_grid::Direction),
    MovePane(pane_grid::Pane, pane_grid::Direction),
    CycleTheme,
    CycleSyntaxTheme,
    SearchQueryChanged(String),
    SearchHistorySelected(String),
    GoToLineQueryChanged(String),
    SearchNext,
    SearchPrevious,
    GoDocumentStart,
    GoDocumentEnd,
    GoToLineRequested,
    ScrollActiveEditorViewport(i32),
    ReplacementEditorInputs(Vec<GuiEditorReplacementInput>),
    ReplacementEditorIme(input_method::Event),
    ToggleReplacementOverwriteMode,
    ReplacementEditorWheelScrolled(pane_grid::Pane, i32),
    ReplacementEditorPointerMoved(pane_grid::Pane, GuiEditorReplacementMousePoint),
    ReplacementEditorBodyPointerMoved(
        pane_grid::Pane,
        GuiEditorReplacementMousePoint,
        GuiEditorDragEdge,
    ),
    ReplacementEditorPointerPressed(pane_grid::Pane),
    ReplacementEditorPointerReleased(pane_grid::Pane),
    ReplacementEditorDragTick,
    ReplacementEditorGlobalPointerReleased,
    ReplacementEditorScrollbarMoved(pane_grid::Pane, f32, GuiEditorScrollbarModel),
    ReplacementEditorScrollbarPressed(pane_grid::Pane),
    ReplacementEditorScrollbarReleased(pane_grid::Pane),
    MenuCommand(GuiMenuCommand),
    ClipboardPasted(Option<String>),
    QuitLatestWindow(Option<window::Id>),
    WindowCloseRequested(window::Id),
}

impl KfnotepadGui {
    fn new_with_task(launch: GuiLaunch) -> (Self, Task<Message>) {
        let mut state = Self::new(launch);
        let task = state.expand_browser_tree_root();
        (state, task)
    }

    #[cfg(not(test))]
    fn new(launch: GuiLaunch) -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_with_current_dir(launch, current_dir)
    }

    #[cfg(test)]
    fn new(launch: GuiLaunch) -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_with_paths(launch, current_dir, None, None, None, None)
    }

    #[cfg(not(test))]
    fn new_with_current_dir(launch: GuiLaunch, current_dir: PathBuf) -> Self {
        Self::new_with_paths(
            launch,
            current_dir,
            current_editor_config_path(),
            current_gui_layout_path(),
            current_gui_workspace_project_launch_path(),
            current_gui_workspace_projects_dir(),
        )
    }

    #[cfg(test)]
    fn new_with_current_dir(launch: GuiLaunch, current_dir: PathBuf) -> Self {
        Self::new_with_paths(launch, current_dir, None, None, None, None)
    }

    fn new_with_paths(
        launch: GuiLaunch,
        current_dir: PathBuf,
        config_path: Option<PathBuf>,
        layout_path: Option<PathBuf>,
        workspace_project_path: Option<PathBuf>,
        workspace_projects_dir: Option<PathBuf>,
    ) -> Self {
        let mut status_messages = Vec::new();
        let mut documents = Vec::new();
        let settings = config_path
            .as_deref()
            .map(load_editor_settings)
            .transpose()
            .map(|settings| settings.unwrap_or_default())
            .unwrap_or_else(|error| {
                status_messages.push(format!("settings unavailable: {error}"));
                EditorSettings::default()
            });

        let mut launch_paths = launch.requested_paths;
        let mut project_layout = None;
        let mut project_active_ordinal = None;
        let project_restore_path = workspace_project_path
            .map(|path| (path, false))
            .or_else(|| {
                if !launch_paths.is_empty() || !settings.gui_restore_last_workspace {
                    return None;
                }
                let projects_dir = workspace_projects_dir.as_deref()?;
                gui_workspace_project_path(projects_dir, "current workspace")
                    .map(|path| (path, true))
            });
        if let Some((path, is_auto_restore)) = project_restore_path {
            match load_workspace_project_launch_documents(&path) {
                Ok((project, project_documents)) => {
                    if is_auto_restore {
                        status_messages
                            .push(format!("restored last workspace project {}", project.name));
                    } else {
                        status_messages.push(format!("opened workspace project {}", project.name));
                    }
                    documents = project_documents;
                    launch_paths.clear();
                    project_layout = project.layout.clone();
                    project_active_ordinal = Some(project.active_ordinal);
                }
                Err(error) => {
                    let action = if is_auto_restore {
                        "workspace auto-restore failed"
                    } else {
                        "workspace project open failed"
                    };
                    status_messages.push(format!("{action}: {}: {error}", path.display()));
                    launch_paths.clear();
                }
            }
        }

        for path in launch_paths {
            match open_text_file(&path) {
                Ok(document) => {
                    status_messages.push(format!("opened {}", document.path.display()));
                    documents.push(document);
                }
                Err(error) => {
                    status_messages.push(format!("cannot open {}: {error}", path.display()));
                }
            }
        }

        if documents.is_empty() {
            documents.push(empty_document(current_dir.clone()));
            if status_messages.is_empty() {
                status_messages.push("started empty GUI document tile".to_string());
            }
        }

        let first_document = documents.remove(0);
        let mut workspace = GuiWorkspace::from_document(first_document);
        let mut pane_states = vec![GuiPane::new(
            workspace.active,
            text_editor::Content::with_text(&workspace.active_tile().document.buffer.to_text()),
        )];
        for document in documents {
            let editor = text_editor::Content::with_text(&document.buffer.to_text());
            let tile_id = workspace.open_tile(document);
            pane_states.push(GuiPane::new(tile_id, editor));
        }

        let restored_layout = project_layout.or_else(|| {
            layout_path
                .as_deref()
                .and_then(|path| load_gui_layout(path, pane_states.len()))
        });
        let browser_visible = restored_layout
            .as_ref()
            .is_none_or(|layout| layout.browser_visible);
        let browser_width = restored_layout
            .as_ref()
            .and_then(|layout| layout.browser_width_px)
            .map(f32::from)
            .map(clamp_browser_width)
            .unwrap_or(GUI_BROWSER_WIDTH_DEFAULT);
        let (panes, mut active_pane) = if let Some(layout) = restored_layout.as_ref() {
            let (panes, pane) = panes_from_gui_layout(layout.clone(), pane_states);
            for ordinal in &layout.minimized_ordinals {
                if let Some(tile_id) = workspace.tiles.get(*ordinal).map(|tile| tile.id) {
                    workspace.set_tile_minimized(tile_id, true);
                }
            }
            status_messages.push("restored GUI layout".to_string());
            (panes, pane)
        } else {
            default_panes(pane_states)
        };
        let (panes, minimized_panes, active_pane_after_minimize) =
            close_minimized_panes_into_tray(panes, &workspace, active_pane);
        active_pane = active_pane_after_minimize;
        if let Some(active) = pane_for_tile_id(&panes, workspace.active) {
            active_pane = active;
        }
        if let Some(active_tile_id) = panes.get(active_pane).map(|pane| pane.tile_id) {
            workspace.focus_tile(active_tile_id);
        }
        if let Some(active_ordinal) = project_active_ordinal {
            if let Some(active_tile_id) = workspace.tiles.get(active_ordinal).map(|tile| tile.id) {
                workspace.focus_tile(active_tile_id);
                if let Some(pane) = pane_for_tile_id(&panes, active_tile_id) {
                    active_pane = pane;
                }
            }
        }

        let (browser, browser_tree, browser_expanded_paths) =
            match GuiFileBrowser::load(current_dir.clone()) {
                Ok(browser) => {
                    let root = browser.sidebar.current_dir.clone();
                    let mut expanded = HashSet::new();
                    expanded.insert(root.clone());
                    (Some(browser), Some(gui_directory_tree(root)), expanded)
                }
                Err(error) => {
                    status_messages.push(format!(
                        "file browser unavailable for {}: {error}",
                        current_dir.display()
                    ));
                    (None, None, HashSet::new())
                }
            };
        let left_panel = GuiLeftPanelState {
            visible: browser_visible,
            mode: GuiLeftPanelMode::Files,
        };
        let notes_dir = current_managed_notes_dir().ok();
        let workspace_projects = workspace_projects_dir
            .as_deref()
            .and_then(|path| match list_gui_workspace_projects(path) {
                Ok(projects) => Some(projects),
                Err(error) => {
                    status_messages.push(format!("workspace projects unavailable: {error}"));
                    None
                }
            })
            .unwrap_or_default();

        let mut state = Self {
            workspace,
            panes,
            active_pane,
            minimized_panes,
            browser,
            browser_tree,
            browser_expanded_paths,
            browser_selected_path: None,
            browser_visible,
            browser_width,
            left_panel,
            current_dir,
            notes_dir,
            workspace_projects_dir,
            workspace_projects,
            workspace_project_name: String::new(),
            pending_project_delete: None,
            pending_browser_delete: None,
            #[cfg(test)]
            spawned_workspace_project_paths: Vec::new(),
            path_prompt: None,
            path_prompt_value: String::new(),
            notes_panel: None,
            pending_managed_note_delete: None,
            file_snapshots: HashMap::new(),
            external_edit_locks: HashSet::new(),
            syntax_caches: HashMap::new(),
            replacement_pointer_point: None,
            replacement_drag: None,
            replacement_drag_edge: None,
            replacement_scrollbar_drag: None,
            replacement_scrollbar_pointer: None,
            replacement_ime_preedit: None,
            replacement_overwrite_mode: false,
            pending_project_open: None,
            pending_close_tile: None,
            pending_app_quit: false,
            search_query: String::new(),
            search_history: Vec::new(),
            search_history_open: false,
            search_highlight: None,
            reader_scroll_accumulator: 0.0,
            go_to_line_query: String::new(),
            syntax_highlighter: SyntaxHighlighter::default(),
            settings,
            config_path,
            layout_path,
            status_message: status_messages.join(" | "),
        };
        state.refresh_all_file_snapshots();
        state.refresh_visible_syntax_caches();
        state.persist_last_workspace_if_enabled();
        state
    }

    #[cfg(test)]
    fn active_editor(&self) -> &GuiEditorAdapter {
        &self
            .panes
            .get(self.active_pane)
            .expect("active GUI pane must exist")
            .editor
    }

    fn refresh_all_file_snapshots(&mut self) {
        self.file_snapshots.clear();
        for tile in &self.workspace.tiles {
            if let Ok(Some(snapshot)) = gui_file_snapshot(&tile.document.path) {
                self.file_snapshots.insert(tile.id, snapshot);
            }
        }
    }

    fn refresh_file_snapshot_for_tile(&mut self, tile_id: GuiTileId) {
        let Some(path) = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.path.clone())
        else {
            self.file_snapshots.remove(&tile_id);
            return;
        };

        match gui_file_snapshot(&path) {
            Ok(Some(snapshot)) => {
                self.file_snapshots.insert(tile_id, snapshot);
            }
            Ok(None) | Err(_) => {
                self.file_snapshots.remove(&tile_id);
            }
        }
    }

    fn refresh_visible_syntax_caches(&mut self) {
        let tile_ids = self
            .panes
            .iter()
            .map(|(_pane, pane_state)| pane_state.tile_id)
            .collect::<Vec<_>>();
        for tile_id in tile_ids {
            self.ensure_visible_syntax_cache_for_tile(tile_id);
        }
    }

    fn invalidate_syntax_cache(&mut self, tile_id: GuiTileId) {
        self.syntax_caches.remove(&tile_id);
    }

    fn invalidate_all_syntax_caches(&mut self) {
        self.syntax_caches.clear();
    }

    fn syntax_cache_target_end_for_tile(&self, tile_id: GuiTileId) -> Option<usize> {
        let pane = pane_for_tile_id(&self.panes, tile_id)?;
        let pane_state = self.panes.get(pane)?;
        let tile = self.workspace.tile(tile_id)?;
        let line_count = tile.document.buffer.line_count().max(1);
        Some(
            pane_state
                .editor
                .viewport
                .first_line
                .saturating_sub(1)
                .saturating_add(GUI_EDITOR_RENDER_LINE_BUDGET)
                .min(line_count),
        )
    }

    fn ensure_visible_syntax_cache_for_tile(&mut self, tile_id: GuiTileId) {
        let Some(target_end) = self.syntax_cache_target_end_for_tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let path = tile.document.path.clone();
        let line_count = tile.document.buffer.line_count().max(1);

        let reset_cache = self.syntax_caches.get(&tile_id).is_none_or(|cache| {
            cache.path != path
                || cache.line_count != line_count
                || cache.highlighted_until > line_count
        });
        if reset_cache {
            self.syntax_caches.insert(
                tile_id,
                GuiSyntaxCache {
                    path: path.clone(),
                    line_count,
                    highlighted_until: 0,
                    lines: Vec::with_capacity(target_end),
                    state: None,
                },
            );
        }

        let (start_line, requested_rows, state) = {
            let Some(cache) = self.syntax_caches.get_mut(&tile_id) else {
                return;
            };
            if target_end <= cache.highlighted_until {
                return;
            }
            let start_line = cache.highlighted_until;
            let requested_rows = target_end.saturating_sub(start_line);
            let state = cache.state.take();
            (start_line, requested_rows, state)
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let (highlighted_lines, next_state) = self
            .syntax_highlighter
            .highlight_lines_incremental_for_path(
                &tile.document.path,
                tile.document.buffer.lines(),
                start_line,
                requested_rows,
                state,
            );

        let Some(cache) = self.syntax_caches.get_mut(&tile_id) else {
            return;
        };
        let theme_id = self.settings.syntax_theme_id;
        cache.lines.extend(
            highlighted_lines.into_iter().map(|line| {
                line.map(|segments| gui_syntax_segments_from_syntect(segments, theme_id))
            }),
        );
        cache.highlighted_until = cache.lines.len().min(line_count);
        cache.state = next_state;
    }

    fn is_external_edit_locked(&self, tile_id: GuiTileId) -> bool {
        self.external_edit_locks.contains(&tile_id)
    }

    fn unlock_external_edit(&mut self, tile_id: GuiTileId) {
        if self.external_edit_locks.remove(&tile_id) {
            self.status_message = "external edit lock cleared".to_string();
        }
    }

    fn poll_external_file_changes(&mut self) {
        let candidates = self
            .workspace
            .tiles
            .iter()
            .map(|tile| {
                (
                    tile.id,
                    tile.document.path.clone(),
                    tile.document.buffer.is_dirty(),
                    self.file_snapshots.get(&tile.id).copied(),
                )
            })
            .collect::<Vec<_>>();

        for (tile_id, path, dirty, previous_snapshot) in candidates {
            let Ok(Some(current_snapshot)) = gui_file_snapshot(&path) else {
                continue;
            };
            let Some(previous_snapshot) = previous_snapshot else {
                self.file_snapshots.insert(tile_id, current_snapshot);
                continue;
            };
            if current_snapshot == previous_snapshot {
                continue;
            }
            if dirty {
                self.file_snapshots.insert(tile_id, current_snapshot);
                self.external_edit_locks.insert(tile_id);
                self.status_message = format!(
                    "external change detected for {}; save or close local edits before refresh",
                    gui_file_name_label(&path)
                );
                continue;
            }

            match open_text_file(&path) {
                Ok(document) => {
                    self.replace_tile_document_from_external_change(tile_id, document);
                    self.file_snapshots.insert(tile_id, current_snapshot);
                    self.external_edit_locks.insert(tile_id);
                    self.status_message =
                        format!("external update loaded: {}", gui_file_name_label(&path));
                }
                Err(error) => {
                    self.status_message =
                        format!("external update skipped for {}: {error}", path.display());
                }
            }
        }
    }

    fn replace_tile_document_from_external_change(
        &mut self,
        tile_id: GuiTileId,
        mut document: TextDocument,
    ) {
        document.buffer.mark_clean();
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.document = document;
            tile.state.cursor = DocumentCursor { row: 0, column: 0 };
            tile.state.viewport_start = 0;
            tile.state.horizontal_offset = 0;
        }
        for (_pane, pane_state) in self.panes.iter_mut() {
            if pane_state.tile_id == tile_id {
                pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(
                    &self
                        .workspace
                        .tile(tile_id)
                        .map(|tile| tile.document.buffer.to_text())
                        .unwrap_or_default(),
                ));
            }
        }
        for pane_state in &mut self.minimized_panes {
            if pane_state.tile_id == tile_id {
                pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(
                    &self
                        .workspace
                        .tile(tile_id)
                        .map(|tile| tile.document.buffer.to_text())
                        .unwrap_or_default(),
                ));
            }
        }
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
    }

    fn focus_pane(&mut self, pane: pane_grid::Pane) -> bool {
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return false;
        };
        self.active_pane = pane;
        if self.panes.maximized().is_some() && self.panes.maximized() != Some(pane) {
            self.panes.restore();
            self.panes.maximize(pane);
        }
        self.workspace.focus_tile(tile_id)
    }

    fn sync_pane_to_document(&mut self, pane: pane_grid::Pane) {
        let Some(pane_state) = self.panes.get(pane) else {
            return;
        };
        let text = pane_state.editor.text();
        let tile_id = pane_state.tile_id;
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.document.buffer.replace_text(&text);
            tile.state.cursor = pane_state.editor.document_cursor();
        }
    }

    fn sync_active_editor_to_document(&mut self) {
        self.sync_pane_to_document(self.active_pane);
    }

    fn perform_active_editor_command(&mut self, command: GuiEditorCommand, status: &str) {
        let invalidates_syntax = gui_editor_command_invalidates_syntax(&command);
        let may_extend_syntax = gui_editor_command_may_extend_syntax_cache(&command);
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.apply(command);
        }
        self.sync_active_editor_to_document();
        if let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        {
            self.workspace.clear_tile_save_error(tile_id);
            if invalidates_syntax {
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
            } else if may_extend_syntax {
                self.ensure_visible_syntax_cache_for_tile(tile_id);
            }
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = status.to_string();
    }

    fn active_editor_selection(&self) -> Option<String> {
        self.panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.selection())
            .filter(|selection| !selection.is_empty())
    }

    fn copy_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        self.status_message = "copied selection".to_string();
        clipboard::write(selection)
    }

    fn cut_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        self.perform_active_editor_command(GuiEditorCommand::Delete, "cut selection");
        clipboard::write(selection)
    }

    fn paste_into_active_editor(&mut self, contents: Option<String>) {
        let Some(contents) = contents.filter(|contents| !contents.is_empty()) else {
            self.status_message = "clipboard is empty".to_string();
            return;
        };
        self.perform_active_editor_command(GuiEditorCommand::Paste(contents), "pasted clipboard");
    }

    fn select_all_active_editor(&mut self) {
        self.perform_active_editor_command(GuiEditorCommand::SelectAll, "selected all");
    }

    fn undo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(true);
    }

    fn redo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(false);
    }

    fn apply_undo_redo_to_active_tile(&mut self, undo: bool) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        if self.is_external_edit_locked(tile_id) {
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return;
        }

        let mut applied = false;
        let mut text = String::new();
        let mut cursor = DocumentCursor { row: 0, column: 0 };
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            let result = if undo {
                undo_document_edit(&mut tile.document, &mut tile.state.cursor)
            } else {
                redo_document_edit(&mut tile.document, &mut tile.state.cursor)
            };
            applied = result == UndoRedoResult::Applied;
            text = tile.document.buffer.to_text();
            cursor = tile.state.cursor;
        }

        if !applied {
            self.status_message = if undo {
                "nothing to undo".to_string()
            } else {
                "nothing to redo".to_string()
            };
            return;
        }

        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            let mut viewport = pane_state.editor.viewport;
            viewport.keep_cursor_visible(cursor, text.lines().count().max(1));
            pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(&text));
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.replacement_selection = None;
        }
        self.workspace.clear_tile_save_error(tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.search_highlight = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = if undo {
            "undo".to_string()
        } else {
            "redo".to_string()
        };
    }

    fn scroll_active_editor_viewport(&mut self, delta: i32) {
        self.perform_active_editor_command(
            GuiEditorCommand::ScrollViewportLines(delta),
            if delta < 0 {
                "viewport up"
            } else {
                "viewport down"
            },
        );
    }

    fn scroll_active_editor_viewport_preserving_cursor(&mut self, delta: i32, status: &str) {
        if delta == 0 {
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state
                .editor
                .scroll_viewport_by_lines_preserving_cursor(delta);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.status_message = status.to_string();
    }

    fn reader_scroll_tick(&mut self) {
        if !self.settings.gui_reader_mode_enabled {
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.set_reader_mode_enabled(false);
            return;
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.set_reader_mode_enabled(false);
            return;
        };
        let Some(pane_state) = self.panes.get(self.active_pane) else {
            self.set_reader_mode_enabled(false);
            return;
        };

        let line_count = tile.document.buffer.line_count().max(1);
        if pane_state.editor.viewport.first_line >= line_count {
            self.set_reader_mode_enabled(false);
            self.status_message = "reader mode stopped at document end".to_string();
            return;
        }

        let lines_per_tick = f32::from(self.settings.gui_reader_lines_per_minute)
            * GUI_READER_TICK_MS as f32
            / 60_000.0;
        self.reader_scroll_accumulator += lines_per_tick;
        let lines = self.reader_scroll_accumulator.floor() as i32;
        if lines <= 0 {
            return;
        }
        self.reader_scroll_accumulator -= lines as f32;
        self.scroll_active_editor_viewport_preserving_cursor(lines, "reader mode");
        self.status_message = format!(
            "reader mode: {} lines/min",
            self.settings.gui_reader_lines_per_minute
        );
    }

    fn scroll_replacement_editor_pane_viewport(&mut self, pane: pane_grid::Pane, delta: i32) {
        if delta == 0 || !self.focus_pane(pane) {
            return;
        }
        self.scroll_active_editor_viewport_preserving_cursor(
            delta,
            if delta < 0 {
                "viewport up"
            } else {
                "viewport down"
            },
        );
    }

    fn apply_replacement_editor_inputs_to_active_tile(
        &mut self,
        inputs: Vec<GuiEditorReplacementInput>,
    ) {
        if inputs.is_empty() {
            return;
        }
        let invalidates_syntax = gui_replacement_inputs_invalidate_syntax(&inputs);
        self.replacement_ime_preedit = None;

        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let mut viewport = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        for input in inputs {
            apply_gui_editor_replacement_input_with_mode(
                &mut tile.document,
                &mut tile.state.cursor,
                &mut viewport,
                &mut replacement_selection,
                self.replacement_overwrite_mode,
                input,
            );
        }
        let text = tile.document.buffer.to_text();
        let cursor = tile.state.cursor;
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(&text));
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.workspace.clear_tile_save_error(tile_id);
        if invalidates_syntax {
            self.invalidate_syntax_cache(tile_id);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = "replacement edit".to_string();
    }

    fn apply_replacement_editor_ime_event(&mut self, event: input_method::Event) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.replacement_ime_preedit = None;
            return;
        };

        if self.is_external_edit_locked(tile_id) {
            self.replacement_ime_preedit = None;
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return;
        }

        match event {
            input_method::Event::Opened => {
                self.replacement_ime_preedit = Some(GuiImePreedit {
                    tile_id,
                    content: String::new(),
                    selection: None,
                });
            }
            input_method::Event::Preedit(content, selection) => {
                self.replacement_ime_preedit = (!content.is_empty()).then_some(GuiImePreedit {
                    tile_id,
                    content,
                    selection,
                });
            }
            input_method::Event::Commit(text) => {
                self.replacement_ime_preedit = None;
                let inputs = gui_editor_replacement_inputs_from_text(&text);
                if !inputs.is_empty() {
                    self.search_highlight = None;
                    self.apply_replacement_editor_inputs_to_active_tile(inputs);
                }
            }
            input_method::Event::Closed => {
                self.replacement_ime_preedit = None;
            }
        }
    }

    fn replacement_editor_pointer_moved(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) {
        self.replacement_pointer_point = Some((pane, point));
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.apply_replacement_editor_mouse_drag_to_pane(pane, point);
        }
    }

    fn replacement_editor_body_pointer_moved(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
        edge: GuiEditorDragEdge,
    ) {
        self.replacement_pointer_point = Some((pane, point));
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.replacement_drag_edge = (edge.direction != 0).then_some(edge);
            self.apply_replacement_editor_mouse_drag_to_pane(pane, point);
        }
    }

    fn replacement_editor_pointer_pressed(&mut self, pane: pane_grid::Pane) {
        let Some((point_pane, point)) = self.replacement_pointer_point else {
            return;
        };
        if point_pane != pane {
            return;
        }

        let Some(anchor) = self.replacement_editor_cursor_for_point(pane, point) else {
            return;
        };
        self.replacement_drag = Some(GuiEditorDragState { pane, anchor });
        self.replacement_drag_edge = None;
        self.apply_replacement_editor_mouse_click_to_pane(pane, point);
    }

    fn replacement_editor_pointer_released(&mut self, pane: pane_grid::Pane) {
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.clear_replacement_drag();
        }
    }

    fn replacement_editor_global_pointer_released(&mut self) {
        self.clear_replacement_drag();
        self.replacement_scrollbar_drag = None;
    }

    fn clear_replacement_drag(&mut self) {
        self.replacement_drag = None;
        self.replacement_drag_edge = None;
    }

    fn replacement_editor_cursor_for_point(
        &self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) -> Option<DocumentCursor> {
        let tile_id = self.panes.get(pane)?.tile_id;
        let viewport = self.panes.get(pane)?.editor.viewport;
        let tile = self.workspace.tile(tile_id)?;
        Some(gui_editor_replacement_cursor_from_mouse_point(
            &tile.document.buffer,
            viewport,
            point,
        ))
    }

    fn apply_replacement_editor_mouse_click_to_pane(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) {
        if !self.focus_pane(pane) {
            return;
        }
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return;
        };
        let mut viewport = self
            .panes
            .get(pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        gui_editor_replacement_mouse_click(
            &tile.document,
            &mut tile.state.cursor,
            &mut viewport,
            &mut replacement_selection,
            point,
        );
        let cursor = tile.state.cursor;
        self.replacement_ime_preedit = None;
        self.update_replacement_editor_view_state(
            pane,
            cursor,
            viewport,
            replacement_selection,
            "cursor moved",
        );
    }

    fn apply_replacement_editor_mouse_drag_to_pane(
        &mut self,
        pane: pane_grid::Pane,
        focus: GuiEditorReplacementMousePoint,
    ) {
        if !self.focus_pane(pane) {
            return;
        }
        let Some(drag) = self.replacement_drag else {
            return;
        };
        if drag.pane != pane {
            return;
        }
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return;
        };
        let mut viewport = self
            .panes
            .get(pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        gui_editor_replacement_mouse_drag(
            &tile.document,
            &mut tile.state.cursor,
            &mut viewport,
            &mut replacement_selection,
            drag.anchor,
            focus,
        );
        let cursor = tile.state.cursor;
        let status = if replacement_selection.is_some() {
            "selected text"
        } else {
            "cursor moved"
        };
        self.replacement_ime_preedit = None;
        self.update_replacement_editor_view_state(
            pane,
            cursor,
            viewport,
            replacement_selection,
            status,
        );
    }

    fn update_replacement_editor_view_state(
        &mut self,
        pane: pane_grid::Pane,
        cursor: DocumentCursor,
        viewport: GuiEditorViewportState,
        replacement_selection: Option<GuiEditorReplacementSelection>,
        status: &str,
    ) {
        if let Some(pane_state) = self.panes.get_mut(pane) {
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.search_highlight = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = status.to_string();
    }

    fn replacement_editor_drag_tick(&mut self) {
        let Some(edge) = self.replacement_drag_edge else {
            return;
        };
        if self
            .replacement_drag
            .is_none_or(|drag| drag.pane != edge.pane)
        {
            self.replacement_drag_edge = None;
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(edge.pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.clear_replacement_drag();
            return;
        };
        let Some(pane_state) = self.panes.get(edge.pane) else {
            self.clear_replacement_drag();
            return;
        };
        let mut viewport = pane_state.editor.viewport;
        let line_count = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.buffer.line_count())
            .unwrap_or(1);
        let before = viewport.first_line;
        viewport.scroll_by(edge.direction, line_count);
        if viewport.first_line == before {
            return;
        }
        if let Some(pane_state) = self.panes.get_mut(edge.pane) {
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = false;
        }

        let focus_row = if edge.direction < 0 {
            viewport.first_line.saturating_sub(1)
        } else {
            viewport.last_visible_line(line_count).saturating_sub(1)
        };
        let focus_point = GuiEditorReplacementMousePoint {
            viewport_row: focus_row
                .saturating_add(1)
                .saturating_sub(viewport.first_line),
            column: edge.column,
        };
        self.apply_replacement_editor_mouse_drag_to_pane(edge.pane, focus_point);
    }

    fn replacement_editor_scrollbar_moved(
        &mut self,
        pane: pane_grid::Pane,
        y: f32,
        model: GuiEditorScrollbarModel,
    ) {
        self.replacement_scrollbar_pointer = Some((pane, y, model));
        let Some(drag) = self.replacement_scrollbar_drag else {
            return;
        };
        if drag.pane != pane {
            return;
        }
        self.set_replacement_editor_scrollbar_first_line(
            pane,
            gui_editor_scrollbar_first_line_from_thumb_y(model, y, drag.thumb_offset),
        );
    }

    fn replacement_editor_scrollbar_pressed(&mut self, pane: pane_grid::Pane) {
        let Some((pointer_pane, y, model)) = self.replacement_scrollbar_pointer else {
            return;
        };
        if pointer_pane != pane {
            return;
        }
        match gui_editor_scrollbar_press_target(model, y) {
            GuiEditorScrollbarPressTarget::None => {}
            GuiEditorScrollbarPressTarget::Page(delta) => {
                self.scroll_replacement_editor_pane_viewport(pane, delta);
            }
            GuiEditorScrollbarPressTarget::Thumb { offset } => {
                self.clear_replacement_drag();
                self.replacement_scrollbar_drag = Some(GuiEditorScrollbarDrag {
                    pane,
                    thumb_offset: offset,
                });
                self.set_replacement_editor_scrollbar_first_line(
                    pane,
                    gui_editor_scrollbar_first_line_from_thumb_y(model, y, offset),
                );
            }
        }
    }

    fn replacement_editor_scrollbar_released(&mut self, pane: pane_grid::Pane) {
        if self
            .replacement_scrollbar_drag
            .is_some_and(|drag| drag.pane == pane)
        {
            self.replacement_scrollbar_drag = None;
        }
    }

    fn set_replacement_editor_scrollbar_first_line(
        &mut self,
        pane: pane_grid::Pane,
        first_line: usize,
    ) {
        let Some(pane_state) = self.panes.get_mut(pane) else {
            return;
        };
        let line_count = pane_state.editor.line_count();
        pane_state.editor.viewport.first_line = first_line;
        pane_state.editor.viewport.clamp_to_line_count(line_count);
        pane_state.editor.viewport_tracks_cursor = false;
        self.status_message = "viewport scrolled".to_string();
    }

    fn open_document_in_new_pane(&mut self, document: TextDocument, opened_status: String) -> bool {
        if self.active_tile_is_replaceable_blank() {
            return self.replace_initial_blank_tile(document, opened_status);
        }
        if let Some(tile_id) = self.open_tile_id_for_path(&document.path) {
            self.focus_or_restore_existing_tile(tile_id, &document.path);
            return true;
        }

        let editor = text_editor::Content::with_text(&document.buffer.to_text());
        let tile_id = self.workspace.open_tile(document);
        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();
        if let Some((pane, _split)) =
            self.panes
                .split(split_axis, self.active_pane, GuiPane::new(tile_id, editor))
        {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.pending_project_open = None;
            self.status_message = opened_status;
            self.external_edit_locks.remove(&tile_id);
            self.refresh_file_snapshot_for_tile(tile_id);
            self.invalidate_syntax_cache(tile_id);
            self.ensure_visible_syntax_cache_for_tile(tile_id);
            self.persist_layout();
            self.persist_last_workspace_if_enabled();
            true
        } else {
            self.status_message = "cannot open document: pane split failed".to_string();
            false
        }
    }

    fn open_help_document(&mut self) {
        let path = self.current_browser_dir().join(GUI_HELP_DOCUMENT_PATH);
        let document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(GUI_HELP_DOCUMENT_TEXT),
        };
        self.open_document_in_new_pane(document, format!("opened help {}", path.display()));
    }

    fn replace_initial_blank_tile(
        &mut self,
        document: TextDocument,
        opened_status: String,
    ) -> bool {
        if !self.active_tile_is_replaceable_blank() {
            return false;
        }

        let Some(pane_state) = self.panes.get_mut(self.active_pane) else {
            return false;
        };
        let tile_id = pane_state.tile_id;
        let editor = text_editor::Content::with_text(&document.buffer.to_text());
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return false;
        };

        tile.document = document;
        tile.state = EditorTabState::default();
        tile.minimized = false;
        self.workspace.focus_tile(tile_id);
        pane_state.editor = GuiEditorAdapter::new(editor);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = opened_status;
        self.external_edit_locks.remove(&tile_id);
        self.refresh_file_snapshot_for_tile(tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.persist_last_workspace_if_enabled();
        true
    }

    fn active_tile_is_replaceable_blank(&self) -> bool {
        if self.workspace.tiles.len() != 1 || self.panes.len() != 1 {
            return false;
        }
        let Some(pane_state) = self.panes.get(self.active_pane) else {
            return false;
        };
        let Some(tile) = self.workspace.tile(pane_state.tile_id) else {
            return false;
        };
        let is_untitled = tile
            .document
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "untitled.txt");

        is_untitled && !tile.document.buffer.is_dirty() && tile.document.buffer.to_text().is_empty()
    }

    fn open_tile_id_for_path(&self, path: &Path) -> Option<GuiTileId> {
        self.workspace
            .tiles
            .iter()
            .find(|tile| gui_paths_refer_to_same_file(&tile.document.path, path))
            .map(|tile| tile.id)
    }

    fn focus_or_restore_existing_tile(&mut self, tile_id: GuiTileId, path: &Path) {
        if let Some(pane) = pane_for_tile_id(&self.panes, tile_id) {
            self.focus_pane(pane);
            self.status_message = format!("already open: {}", path.display());
            return;
        }

        if self
            .minimized_panes
            .iter()
            .any(|pane| pane.tile_id == tile_id)
        {
            self.restore_minimized_tile(tile_id);
            self.status_message = format!("restored open file: {}", path.display());
            return;
        }

        self.workspace.focus_tile(tile_id);
        self.status_message = format!("already open: {}", path.display());
    }

    fn open_path_in_new_pane(&mut self, path: PathBuf) -> bool {
        match open_text_file(&path) {
            Ok(document) => {
                let opened_path = document.path.clone();
                self.open_document_in_new_pane(
                    document,
                    format!("opened {}", opened_path.display()),
                )
            }
            Err(error) => {
                self.status_message = format!("cannot open {}: {error}", path.display());
                false
            }
        }
    }

    fn request_open_dialog(&mut self) -> Task<Message> {
        self.path_prompt = None;
        self.path_prompt_value.clear();
        let directory = self.current_browser_dir();
        self.status_message = "open dialog".to_string();

        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .set_directory(directory)
                    .pick_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Message::OpenDialogSelected,
        )
    }

    fn handle_open_dialog_selected(&mut self, path: Option<PathBuf>) {
        match path {
            Some(path) => {
                let _opened = self.open_path_in_new_pane(path);
            }
            None => {
                self.status_message = "open canceled".to_string();
            }
        }
    }

    fn create_new_tile(&mut self) {
        let path = self.next_untitled_path();
        let document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(""),
        };
        let editor = text_editor::Content::with_text("");
        let tile_id = self.workspace.open_tile(document);
        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();

        if let Some((pane, _split)) =
            self.panes
                .split(split_axis, self.active_pane, GuiPane::new(tile_id, editor))
        {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.pending_project_open = None;
            self.file_snapshots.remove(&tile_id);
            self.external_edit_locks.remove(&tile_id);
            self.invalidate_syntax_cache(tile_id);
            self.ensure_visible_syntax_cache_for_tile(tile_id);
            self.status_message = format!("new tile {}", path.display());
            self.persist_layout();
            self.persist_last_workspace_if_enabled();
        } else {
            self.status_message = format!(
                "cannot create new tile {}: pane split failed",
                path.display()
            );
        }
    }

    fn next_untitled_path(&self) -> PathBuf {
        let current_dir = self
            .browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
            .unwrap_or_else(|| self.current_dir.clone());
        self.next_untitled_path_in_dir_excluding(current_dir, None)
    }

    fn next_untitled_path_in_dir_excluding(
        &self,
        directory: PathBuf,
        excluded_tile_id: Option<GuiTileId>,
    ) -> PathBuf {
        for index in 1.. {
            let file_name = if index == 1 {
                "untitled.txt".to_string()
            } else {
                format!("untitled-{index}.txt")
            };
            let candidate = directory.join(file_name);
            let already_open =
                self.workspace.tiles.iter().any(|tile| {
                    Some(tile.id) != excluded_tile_id && tile.document.path == candidate
                });
            if !already_open && !candidate.exists() {
                return candidate;
            }
        }

        unreachable!("untitled candidate search is unbounded")
    }

    #[cfg(test)]
    fn activate_browser_entry(&mut self, index: usize) {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return;
        }
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        if index >= browser.sidebar.entries.len() {
            return;
        }

        browser.sidebar.selected = index;
        if let Some(entry) = browser.sidebar.entries.get(index) {
            self.browser_selected_path = Some(entry.path.clone());
        }
        self.open_selected_browser_entry();
    }

    #[cfg(test)]
    fn select_browser_entry(&mut self, index: usize) {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return;
        }
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        if let Some(entry) = browser.sidebar.entries.get(index) {
            self.status_message = format!("selected {}", entry.path.display());
            browser.sidebar.selected = index;
            self.browser_selected_path = Some(entry.path.clone());
            self.pending_browser_delete = None;
        }
    }

    #[cfg(test)]
    fn open_selected_browser_entry(&mut self) {
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        match browser.activate_selected() {
            Ok(kfnotepad::GuiFileBrowserActivation::Navigated { current_dir }) => {
                self.status_message = format!("browser: {}", current_dir.display());
            }
            Ok(kfnotepad::GuiFileBrowserActivation::OpenTile { path }) => {
                let _opened = self.open_path_in_new_pane(path);
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
            }
        }
    }

    fn handle_browser_tree_event(&mut self, event: DirectoryTreeEvent) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        match &event {
            DirectoryTreeEvent::Selected(path, is_dir, _) => {
                if *is_dir {
                    self.select_browser_path(path);
                    self.status_message = format!("selected folder {}", path.display());
                } else {
                    self.select_browser_path(path);
                    self.status_message = format!("selected file {}", path.display());
                }
            }
            DirectoryTreeEvent::DragCompleted { .. } => {
                self.status_message = "file browser drag is view-only".to_string();
            }
            DirectoryTreeEvent::Toggled(_)
            | DirectoryTreeEvent::Drag(_)
            | DirectoryTreeEvent::Loaded(_) => {}
        }

        let Some(tree) = self.browser_tree.as_mut() else {
            self.status_message = "file tree unavailable".to_string();
            return Task::none();
        };
        tree.update(event).map(Message::BrowserTreeEvent)
    }

    fn toggle_local_browser_tree_path(&mut self, path: PathBuf) {
        if self.browser_expanded_paths.contains(&path) {
            self.browser_expanded_paths.remove(&path);
        } else {
            self.browser_expanded_paths.insert(path);
        }
    }

    fn select_local_browser_tree_path(&mut self, path: PathBuf, is_dir: bool) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        self.status_message = if is_dir {
            format!("selected folder {}", path.display())
        } else {
            format!("selected file {}", path.display())
        };
        Task::none()
    }

    fn activate_local_browser_tree_path(&mut self, path: PathBuf, is_dir: bool) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        if is_dir {
            self.set_browser_root(path)
        } else {
            let _opened = self.open_path_in_new_pane(path);
            Task::none()
        }
    }

    fn select_browser_path(&mut self, path: &Path) {
        self.browser_selected_path = Some(path.to_path_buf());
        let Some(browser) = self.browser.as_mut() else {
            return;
        };
        if let Some(index) = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.path == path)
        {
            browser.sidebar.selected = index;
            browser.sidebar.keep_selection_visible(1);
        }
        self.pending_browser_delete = None;
    }

    fn navigate_browser_parent(&mut self) -> Task<Message> {
        let current_dir = self.current_browser_dir();
        let Some(parent) = current_dir.parent() else {
            self.status_message = "already at filesystem root".to_string();
            return Task::none();
        };
        self.set_browser_root(parent.to_path_buf())
    }

    fn refresh_file_browser(&mut self) -> Task<Message> {
        let Some(browser) = self.browser.as_mut() else {
            return self.set_browser_root(self.current_browser_dir());
        };

        match browser.refresh() {
            Ok(()) => {
                let current_dir = browser.sidebar.current_dir.clone();
                self.browser_tree = Some(gui_directory_tree(current_dir.clone()));
                self.browser_expanded_paths.insert(current_dir.clone());
                if self
                    .browser_selected_path
                    .as_ref()
                    .is_some_and(|path| !path.exists())
                {
                    self.browser_selected_path = None;
                }
                self.status_message = format!("refreshed {}", current_dir.display());
                self.expand_browser_tree_root()
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
                Task::none()
            }
        }
    }

    fn create_browser_file_named(&mut self, raw_name: &str) -> bool {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create file failed: {message}");
                return false;
            }
        };
        let buffer = TextBuffer::from_text("");

        if let Err(error) = save_text_buffer(&path, &buffer) {
            self.status_message = format!("create file failed: {error}");
            return false;
        }

        let _refresh_task = self.refresh_file_browser();
        self.select_browser_path(&path);
        let opened = self.open_path_in_new_pane(path.clone());
        if opened {
            self.status_message = format!("created {}", path.display());
        }
        opened
    }

    fn create_browser_directory_named(&mut self, raw_name: &str) -> bool {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create directory failed: {message}");
                return false;
            }
        };

        match fs::create_dir(&path) {
            Ok(()) => {
                let _refresh_task = self.refresh_file_browser();
                self.select_browser_path(&path);
                self.browser_expanded_paths.insert(path.clone());
                self.status_message = format!("created directory {}", path.display());
                true
            }
            Err(error) => {
                self.status_message = format!("create directory failed: {error}");
                false
            }
        }
    }

    fn selected_browser_create_directory(&self) -> PathBuf {
        self.selected_browser_action_entry()
            .and_then(|entry| match entry.kind {
                FileSidebarEntryKind::Directory => Some(entry.path),
                FileSidebarEntryKind::Parent | FileSidebarEntryKind::File => None,
            })
            .unwrap_or_else(|| self.current_browser_dir())
    }

    fn selected_browser_action_entry(&self) -> Option<FileSidebarEntry> {
        if let Some(path) = self.browser_selected_path.as_deref() {
            if path.is_dir() {
                return Some(FileSidebarEntry {
                    label: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    path: path.to_path_buf(),
                    kind: FileSidebarEntryKind::Directory,
                });
            }
            if path.is_file() {
                return Some(FileSidebarEntry {
                    label: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    path: path.to_path_buf(),
                    kind: FileSidebarEntryKind::File,
                });
            }
        }

        self.browser
            .as_ref()
            .and_then(|browser| browser.selected_entry())
            .cloned()
    }

    fn delete_selected_browser_entry(&mut self) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }
        let Some(entry) = self.selected_browser_action_entry() else {
            self.status_message = "delete failed: no file-browser selection".to_string();
            return Task::none();
        };
        if entry.kind == FileSidebarEntryKind::Parent {
            self.pending_browser_delete = None;
            self.status_message = "delete failed: cannot delete parent shortcut".to_string();
            return Task::none();
        }
        if self.path_is_open_in_workspace(&entry.path) {
            self.pending_browser_delete = None;
            self.status_message =
                format!("close open tile before deleting {}", entry.path.display());
            return Task::none();
        }
        if entry.kind == FileSidebarEntryKind::Directory
            && self.directory_contains_open_tile(&entry.path)
        {
            self.pending_browser_delete = None;
            self.status_message = format!(
                "close open tiles inside {} before deleting directory",
                entry.path.display()
            );
            return Task::none();
        }

        if self.pending_browser_delete.as_deref() != Some(entry.path.as_path()) {
            self.pending_browser_delete = Some(entry.path.clone());
            self.pending_project_open = None;
            self.pending_project_delete = None;
            self.pending_managed_note_delete = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = if entry.kind == FileSidebarEntryKind::Directory {
                format!(
                    "delete directory {} and all subdirectories/files? click delete again",
                    entry.path.display()
                )
            } else {
                format!("delete file {}? click delete again", entry.path.display())
            };
            return Task::none();
        }

        self.pending_browser_delete = None;
        match delete_browser_path(&entry.path, entry.kind) {
            Ok(()) => {
                let refresh_task = self.refresh_file_browser();
                self.browser_selected_path = None;
                self.status_message = match entry.kind {
                    FileSidebarEntryKind::Directory => {
                        format!("deleted directory {}", entry.path.display())
                    }
                    FileSidebarEntryKind::File => format!("deleted file {}", entry.path.display()),
                    FileSidebarEntryKind::Parent => unreachable!("parent handled above"),
                };
                refresh_task
            }
            Err(error) => {
                self.status_message = format!("delete failed: {error}");
                Task::none()
            }
        }
    }

    fn path_is_open_in_workspace(&self, path: &Path) -> bool {
        self.workspace
            .tiles
            .iter()
            .any(|tile| gui_paths_refer_to_same_file(&tile.document.path, path))
    }

    fn directory_contains_open_tile(&self, directory: &Path) -> bool {
        let canonical_directory = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf());
        self.workspace.tiles.iter().any(|tile| {
            tile.document
                .path
                .canonicalize()
                .unwrap_or_else(|_| tile.document.path.clone())
                .starts_with(&canonical_directory)
        })
    }

    fn set_browser_root(&mut self, directory: PathBuf) -> Task<Message> {
        match GuiFileBrowser::load(directory) {
            Ok(browser) => {
                let current_dir = browser.sidebar.current_dir.clone();
                self.browser = Some(browser);
                self.browser_tree = Some(gui_directory_tree(current_dir.clone()));
                self.browser_expanded_paths.clear();
                self.browser_expanded_paths.insert(current_dir.clone());
                self.browser_selected_path = Some(current_dir.clone());
                self.status_message = format!("browser: {}", current_dir.display());
                self.expand_browser_tree_root()
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
                Task::none()
            }
        }
    }

    fn expand_browser_tree_root(&mut self) -> Task<Message> {
        let Some(tree) = self.browser_tree.as_mut() else {
            return Task::none();
        };
        let root = tree.root_path().to_path_buf();
        tree.update(DirectoryTreeEvent::Toggled(root))
            .map(Message::BrowserTreeEvent)
    }

    fn save_active_tile(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save failed: no active pane".to_string();
            return;
        };
        let result = {
            let Some(tile) = self.workspace.tile_mut(tile_id) else {
                self.status_message = "save failed: no active tile".to_string();
                return;
            };
            save_text_document(&mut tile.document)
        };

        match result {
            Ok(()) => {
                self.workspace.clear_tile_save_error(tile_id);
                if self.pending_close_tile == Some(tile_id) {
                    self.pending_close_tile = None;
                }
                self.pending_app_quit = false;
                self.external_edit_locks.remove(&tile_id);
                self.refresh_file_snapshot_for_tile(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = format!(
                    "saved {}",
                    self.workspace.active_tile().document.path.display()
                );
            }
            Err(error) => {
                let message = error.to_string();
                self.workspace
                    .mark_tile_save_failed(tile_id, message.clone());
                self.status_message = format!("save failed: {message}");
            }
        }
    }

    fn save_active_tile_as(&mut self, path: PathBuf) -> bool {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save as failed: no active pane".to_string();
            return false;
        };

        if let Some(open_tile_id) = self.open_tile_id_for_path(&path) {
            if open_tile_id != tile_id {
                self.focus_or_restore_existing_tile(open_tile_id, &path);
                self.status_message = format!(
                    "save as refused: {} is already open in another tile",
                    path.display()
                );
                return false;
            }
        }

        let result = {
            let Some(tile) = self.workspace.tile_mut(tile_id) else {
                self.status_message = "save as failed: no active tile".to_string();
                return false;
            };
            let original_path = tile.document.path.clone();
            let original_snapshot = tile.document.buffer.file_snapshot().cloned();
            tile.document.path = path.clone();
            if !gui_paths_refer_to_same_file(&original_path, &path) {
                tile.document.buffer.set_file_snapshot(None);
            }
            match save_text_document(&mut tile.document) {
                Ok(()) => Ok(()),
                Err(error) => {
                    tile.document.path = original_path;
                    tile.document.buffer.set_file_snapshot(original_snapshot);
                    Err(error)
                }
            }
        };

        match result {
            Ok(()) => {
                self.workspace.clear_tile_save_error(tile_id);
                self.pending_close_tile = None;
                self.pending_app_quit = false;
                self.external_edit_locks.remove(&tile_id);
                self.refresh_file_snapshot_for_tile(tile_id);
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = format!("saved as {}", path.display());
                true
            }
            Err(error) => {
                let message = error.to_string();
                self.workspace
                    .mark_tile_save_failed(tile_id, message.clone());
                self.status_message = format!("save as failed: {message}");
                false
            }
        }
    }

    fn request_save_as_dialog(&mut self) -> Task<Message> {
        self.path_prompt = None;
        self.path_prompt_value.clear();
        let active_path = self.workspace.active_tile().document.path.clone();
        let directory = active_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.current_browser_dir());
        let file_name = active_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("untitled.txt")
            .to_string();
        self.status_message = "save as dialog".to_string();

        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .set_directory(directory)
                    .set_file_name(file_name)
                    .save_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Message::SaveAsDialogSelected,
        )
    }

    fn handle_save_as_dialog_selected(&mut self, path: Option<PathBuf>) {
        match path {
            Some(path) => {
                let _saved = self.save_active_tile_as(path);
            }
            None => {
                self.status_message = "save as canceled".to_string();
            }
        }
    }

    fn open_managed_note_by_title(&mut self, title: &str) -> bool {
        let Some(notes_dir) = self.notes_dir.clone() else {
            self.status_message =
                "managed notes unavailable: cannot resolve data directory".to_string();
            return false;
        };

        match open_or_create_managed_note(&notes_dir, title) {
            Ok(document) => {
                let path = document.path.clone();
                self.open_document_in_new_pane(document, format!("opened note {}", path.display()))
            }
            Err(error) => {
                self.status_message = format!("managed note failed: {error}");
                false
            }
        }
    }

    fn list_managed_notes_panel(&mut self) {
        let Some(notes_dir) = self.notes_dir.as_deref() else {
            self.notes_panel = None;
            self.pending_managed_note_delete = None;
            self.status_message =
                "managed notes unavailable: cannot resolve data directory".to_string();
            return;
        };

        match list_managed_notes(notes_dir) {
            Ok(notes) => {
                let count = notes.len();
                self.notes_panel = Some(notes);
                self.pending_managed_note_delete = None;
                self.status_message = format!("managed notes: {count}");
            }
            Err(error) => {
                self.notes_panel = None;
                self.pending_managed_note_delete = None;
                self.status_message = format!("managed notes failed: {error}");
            }
        }
    }

    fn open_managed_note_from_panel(&mut self, index: usize) {
        let Some(note) = self
            .notes_panel
            .as_ref()
            .and_then(|notes| notes.get(index))
            .cloned()
        else {
            return;
        };

        if self.open_path_in_new_pane(note.path) {
            self.notes_panel = None;
            self.pending_managed_note_delete = None;
        }
    }

    fn delete_managed_note_from_panel(&mut self, index: usize) {
        let Some(note) = self
            .notes_panel
            .as_ref()
            .and_then(|notes| notes.get(index))
            .cloned()
        else {
            return;
        };

        if self
            .workspace
            .tiles
            .iter()
            .any(|tile| tile.document.path == note.path)
        {
            self.pending_managed_note_delete = None;
            self.status_message = format!("close note tile before deleting {}", note.file_name);
            return;
        }

        if self.pending_managed_note_delete.as_deref() != Some(note.path.as_path()) {
            self.pending_managed_note_delete = Some(note.path.clone());
            self.pending_project_open = None;
            self.pending_project_delete = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = format!("delete note {}? click delete again", note.file_name);
            return;
        }

        self.pending_managed_note_delete = None;
        let Some(notes_dir) = self.notes_dir.clone() else {
            self.status_message =
                "managed note delete failed: cannot resolve data directory".to_string();
            return;
        };

        match delete_managed_note(&notes_dir, &note.path) {
            Ok(ManagedNoteDeleteResult::Deleted) => {
                self.list_managed_notes_panel();
                self.status_message = format!("managed note deleted: {}", note.file_name);
            }
            Ok(ManagedNoteDeleteResult::Missing) => {
                self.list_managed_notes_panel();
                self.status_message = format!("managed note already missing: {}", note.file_name);
            }
            Err(error) => {
                self.status_message = format!("managed note delete failed: {error}");
            }
        }
    }

    fn close_active_pane(&mut self) {
        self.close_pane(self.active_pane);
    }

    fn visible_tile_count(&self) -> usize {
        self.workspace
            .tiles
            .iter()
            .filter(|tile| !tile.minimized)
            .count()
    }

    fn sync_all_panes_to_documents(&mut self) {
        let panes: Vec<_> = self.panes.iter().map(|(pane, _pane_state)| *pane).collect();
        for pane in panes {
            self.sync_pane_to_document(pane);
        }
    }

    fn has_dirty_tile(&mut self) -> bool {
        self.sync_all_panes_to_documents();
        self.workspace
            .tiles
            .iter()
            .any(|tile| tile.document.buffer.is_dirty())
    }

    fn request_app_close(&mut self, window_id: window::Id) -> Task<Message> {
        if self.has_dirty_tile() {
            if self.pending_app_quit {
                return window::close(window_id);
            }
            self.pending_app_quit = true;
            self.pending_close_tile = None;
            self.status_message =
                "unsaved changes; close window again to discard all dirty tiles".to_string();
            return Task::none();
        }

        window::close(window_id)
    }

    fn move_active_editor_to_document_cursor(&mut self) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let Some(cursor) = self.workspace.tile(tile_id).map(|tile| tile.state.cursor) else {
            return;
        };
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.move_to(cursor);
        }
    }

    fn search_active(&mut self, backwards: bool) {
        self.sync_active_editor_to_document();
        let query = self.search_query.trim().to_string();
        self.remember_search_query(&query);
        self.search_history_open = false;
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "search failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "search failed: no active tile".to_string();
            return;
        };

        let result = gui_repeat_search(
            &tile.document,
            &mut tile.state.cursor,
            &query,
            backwards,
            self.settings.search_case_sensitive,
        );
        let matched_chars = match &result {
            SearchRepeatResult::Found { query } => query.chars().count(),
            SearchRepeatResult::NoPreviousSearch | SearchRepeatResult::NoMatch { .. } => 0,
        };
        self.search_highlight = match &result {
            SearchRepeatResult::Found { query } => Some(GuiSearchHighlight {
                tile_id,
                query: query.clone(),
            }),
            SearchRepeatResult::NoPreviousSearch | SearchRepeatResult::NoMatch { .. } => None,
        };

        self.status_message = search_result_status(result, backwards);
        self.move_active_editor_to_document_cursor();
        if matched_chars > 0 {
            if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
                pane_state.editor.select_right_chars(matched_chars);
            }
        }
    }

    fn remember_search_query(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        self.search_history.retain(|entry| entry != query);
        self.search_history.insert(0, query.to_string());
        self.search_history.truncate(GUI_FIND_HISTORY_LIMIT);
    }

    fn select_search_history(&mut self, query: String) {
        self.search_query = query;
        self.search_history_open = false;
        self.search_active(false);
    }

    fn go_active_document_start(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "navigation failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "navigation failed: no active tile".to_string();
            return;
        };
        go_to_document_start(&mut tile.state.cursor);
        self.move_active_editor_to_document_cursor();
        self.status_message = "moved to document start".to_string();
    }

    fn go_active_document_end(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "navigation failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "navigation failed: no active tile".to_string();
            return;
        };
        go_to_document_end(&tile.document, &mut tile.state.cursor);
        self.move_active_editor_to_document_cursor();
        self.status_message = "moved to document end".to_string();
    }

    fn go_active_line(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "go to line failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "go to line failed: no active tile".to_string();
            return;
        };

        let result = go_to_line(
            &tile.document,
            &mut tile.state.cursor,
            self.go_to_line_query.trim(),
        );
        self.status_message = go_to_line_status(result);
        self.move_active_editor_to_document_cursor();
    }

    fn close_pane(&mut self, pane: pane_grid::Pane) {
        self.sync_pane_to_document(pane);
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            self.status_message = "close failed: no such pane".to_string();
            return;
        };

        let confirm_dirty = self.pending_close_tile == Some(tile_id);
        if self.workspace.tiles.len() <= 1 {
            self.close_last_tile(pane, tile_id, confirm_dirty);
            return;
        }

        match self.workspace.close_tile(tile_id, confirm_dirty) {
            GuiCloseTileResult::Missing => {
                self.pending_close_tile = None;
                self.status_message = "close failed: no such tile".to_string();
            }
            GuiCloseTileResult::OnlyTile => {
                self.pending_close_tile = None;
                self.close_last_tile(pane, tile_id, confirm_dirty);
            }
            GuiCloseTileResult::Dirty { tile_id } => {
                self.pending_close_tile = Some(tile_id);
                self.focus_pane(pane);
                self.status_message =
                    "unsaved changes; close again to discard this tile".to_string();
            }
            GuiCloseTileResult::Closed { tile_id, path } => {
                self.file_snapshots.remove(&tile_id);
                self.external_edit_locks.remove(&tile_id);
                self.invalidate_syntax_cache(tile_id);
                self.pending_close_tile = None;
                self.pending_app_quit = false;
                if self.panes.len() <= 1 && !self.minimized_panes.is_empty() {
                    if self.restore_first_minimized_into_pane(pane).is_some() {
                        self.status_message = format!("closed {}", path.display());
                        self.persist_layout();
                    } else {
                        self.status_message =
                            format!("closed {} but pane replacement failed", path.display());
                    }
                } else if let Some((_closed, fallback_pane)) = self.panes.close(pane) {
                    self.active_pane = fallback_pane;
                    if let Some(fallback_tile) = self
                        .panes
                        .get(fallback_pane)
                        .map(|pane_state| pane_state.tile_id)
                    {
                        self.workspace.focus_tile(fallback_tile);
                    }
                    self.status_message = format!("closed {}", path.display());
                    self.persist_layout();
                } else {
                    self.status_message =
                        format!("closed {} but pane removal failed", path.display());
                }
            }
        }
    }

    fn restore_first_minimized_into_pane(&mut self, pane: pane_grid::Pane) -> Option<GuiTileId> {
        if self.minimized_panes.is_empty() {
            return None;
        }

        let pane_state = self.minimized_panes.remove(0);
        let tile_id = pane_state.tile_id;
        if let Some(target) = self.panes.get_mut(pane) {
            *target = pane_state;
            self.workspace.set_tile_minimized(tile_id, false);
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            self.refresh_visible_syntax_caches();
            Some(tile_id)
        } else {
            self.minimized_panes.insert(0, pane_state);
            None
        }
    }

    fn close_last_tile(&mut self, pane: pane_grid::Pane, tile_id: GuiTileId, confirm_dirty: bool) {
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.pending_close_tile = None;
            self.status_message = "close failed: no such tile".to_string();
            return;
        };
        if tile.document.buffer.is_dirty() && !confirm_dirty {
            self.pending_close_tile = Some(tile_id);
            self.focus_pane(pane);
            self.status_message = "unsaved changes; close again to discard this tile".to_string();
            return;
        }

        let blank_dir = self
            .workspace
            .tile(tile_id)
            .and_then(|tile| tile.document.path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.current_browser_dir());
        let path = self.next_untitled_path_in_dir_excluding(blank_dir, Some(tile_id));
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.pending_close_tile = None;
            self.status_message = "close failed: no such tile".to_string();
            return;
        };
        tile.document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(""),
        };
        tile.state = EditorTabState::default();
        tile.minimized = false;
        if let Some(pane_state) = self.panes.get_mut(pane) {
            pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(""));
        }
        self.active_pane = pane;
        self.workspace.focus_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.file_snapshots.remove(&tile_id);
        self.external_edit_locks.remove(&tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.status_message = format!("new blank tile {}", path.display());
        self.persist_layout();
    }

    fn toggle_active_minimize(&mut self) {
        self.toggle_pane_minimized(self.active_pane);
    }

    fn toggle_active_maximize(&mut self) {
        self.toggle_pane_maximized(self.active_pane);
    }

    fn toggle_pane_maximized(&mut self, pane: pane_grid::Pane) {
        let was_maximized = self.panes.maximized() == Some(pane);
        if !self.focus_pane(pane) {
            self.status_message = "maximize failed: no such pane".to_string();
            return;
        }

        if was_maximized {
            self.panes.restore();
            self.status_message = "restored tile layout".to_string();
        } else {
            self.panes.maximize(pane);
            self.status_message = "maximized tile".to_string();
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
    }

    fn equalize_tile_layout(&mut self) {
        self.sync_all_panes_to_documents();
        let visible_tile_ids = self
            .workspace
            .tiles
            .iter()
            .filter(|tile| !tile.minimized)
            .map(|tile| tile.id)
            .collect::<Vec<_>>();
        if visible_tile_ids.len() <= 1 {
            self.status_message = "tile layout already equalized".to_string();
            return;
        }

        let Some(layout_root) = equalized_tile_layout_node(visible_tile_ids.len()) else {
            self.status_message = "equalize failed: no visible tiles".to_string();
            return;
        };
        let mut pane_states = Vec::with_capacity(visible_tile_ids.len());
        for tile_id in &visible_tile_ids {
            let Some(pane) = pane_for_tile_id(&self.panes, *tile_id) else {
                self.status_message = "equalize failed: missing pane".to_string();
                return;
            };
            let Some(pane_state) = self.panes.get(pane) else {
                self.status_message = "equalize failed: missing pane".to_string();
                return;
            };
            pane_states.push(GuiPane {
                tile_id: *tile_id,
                editor: pane_state.editor.clone_for_relayout(),
            });
        }

        let layout = GuiLayout {
            browser_visible: self.browser_visible,
            browser_width_px: Some(persisted_browser_width(self.browser_width)),
            root: layout_root,
            minimized_ordinals: self
                .workspace
                .tiles
                .iter()
                .enumerate()
                .filter_map(|(ordinal, tile)| tile.minimized.then_some(ordinal))
                .collect(),
        };
        let active_tile_id = self.workspace.active;
        let was_maximized = self.panes.maximized().is_some();
        let (mut panes, fallback_pane) = panes_from_gui_layout(layout, pane_states);
        let active_pane = pane_for_tile_id(&panes, active_tile_id).unwrap_or(fallback_pane);
        if was_maximized {
            panes.maximize(active_pane);
        }
        self.panes = panes;
        self.active_pane = active_pane;
        self.workspace.focus_tile(active_tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = format!("equalized {} tiles", visible_tile_ids.len());
        self.refresh_visible_syntax_caches();
        self.persist_layout();
    }

    fn toggle_pane_minimized(&mut self, pane: pane_grid::Pane) {
        self.sync_pane_to_document(pane);
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            self.status_message = "minimize failed: no such pane".to_string();
            return;
        };
        let Some(was_minimized) = self.workspace.tile(tile_id).map(|tile| tile.minimized) else {
            self.status_message = "minimize failed: no such tile".to_string();
            return;
        };
        if !was_minimized && self.visible_tile_count() <= 1 {
            self.focus_pane(pane);
            self.status_message = "cannot minimize the only visible tile".to_string();
            return;
        }

        let minimized = !was_minimized;
        if minimized {
            let Some((pane_state, fallback_pane)) = self.panes.close(pane) else {
                self.status_message = "minimize failed: pane close failed".to_string();
                return;
            };
            if !self.workspace.set_tile_minimized(tile_id, true) {
                self.status_message = "minimize failed: no such tile".to_string();
                return;
            }
            self.minimized_panes.push(pane_state);
            self.active_pane = fallback_pane;
            if let Some(fallback_tile_id) = self.panes.get(fallback_pane).map(|pane| pane.tile_id) {
                self.workspace.focus_tile(fallback_tile_id);
            }
            self.status_message = "minimized tile".to_string();
            self.refresh_visible_syntax_caches();
        } else {
            self.restore_minimized_tile(tile_id);
            return;
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.persist_layout();
    }

    fn restore_minimized_tile(&mut self, tile_id: GuiTileId) {
        let Some(index) = self
            .minimized_panes
            .iter()
            .position(|pane_state| pane_state.tile_id == tile_id)
        else {
            self.status_message = "restore failed: no such minimized tile".to_string();
            return;
        };
        let pane_state = self.minimized_panes.remove(index);
        if !self.workspace.set_tile_minimized(tile_id, false) {
            self.status_message = "minimize failed: no such tile".to_string();
            return;
        }

        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();
        if let Some((pane, _split)) = self.panes.split(split_axis, self.active_pane, pane_state) {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.status_message = "restored tile".to_string();
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.refresh_visible_syntax_caches();
            self.persist_layout();
        } else {
            if let Some(tile) = self.workspace.tile_mut(tile_id) {
                tile.minimized = true;
            }
            self.status_message = "restore failed: pane split failed".to_string();
        }
    }

    fn minimized_tray_items(&self) -> Vec<GuiMinimizedTrayItem> {
        self.minimized_panes
            .iter()
            .filter_map(|pane_state| {
                let tile = self.workspace.tile(pane_state.tile_id)?;
                let save_status = match tile.save_status() {
                    GuiTileSaveStatus::Saved => "saved".to_string(),
                    GuiTileSaveStatus::Modified => "modified".to_string(),
                    GuiTileSaveStatus::SaveFailed { message } => {
                        format!("save failed: {message}")
                    }
                };
                Some(GuiMinimizedTrayItem {
                    tile_id: pane_state.tile_id,
                    title: gui_tile_title_label(&tile.document.path, false, &save_status),
                    tooltip: tile.document.path.display().to_string(),
                })
            })
            .collect()
    }

    fn move_active_pane(&mut self, direction: pane_grid::Direction) {
        self.move_pane(self.active_pane, direction);
    }

    fn move_pane(&mut self, pane: pane_grid::Pane, direction: pane_grid::Direction) {
        if !self.focus_pane(pane) {
            self.status_message = "move failed: no such pane".to_string();
            return;
        }
        let Some(adjacent) = self.panes.adjacent(pane, direction) else {
            self.status_message = "move failed: no adjacent pane".to_string();
            return;
        };

        self.panes.swap(pane, adjacent);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.status_message = "moved active tile".to_string();
        self.persist_layout();
    }

    fn drag_pane(&mut self, event: pane_grid::DragEvent) {
        if let pane_grid::DragEvent::Dropped { pane, target } = event {
            self.panes.drop(pane, target);
            self.focus_pane(pane);
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = "moved tile".to_string();
            self.persist_layout();
        }
    }

    fn cycle_theme(&mut self) {
        self.settings.theme_id = self.settings.theme_id.next();
        self.status_message = format!("theme: {}", self.settings.theme_id.label());
        self.persist_settings();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
    }

    fn cycle_syntax_theme(&mut self) {
        self.settings.syntax_theme_id = self.settings.syntax_theme_id.next();
        self.status_message = format!("syntax theme: {}", self.settings.syntax_theme_id.label());
        self.persist_settings();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
    }

    fn update_settings_with_rollback(
        &mut self,
        update: impl FnOnce(&mut EditorSettings),
        success_message: impl Into<String>,
    ) {
        let previous = self.settings;
        update(&mut self.settings);
        if let Some(config_path) = self.config_path.as_deref() {
            if let Err(error) = save_editor_settings(config_path, self.settings) {
                self.settings = previous;
                self.status_message = format!("settings save failed: {error}");
                return;
            }
        }
        self.status_message = success_message.into();
    }

    fn set_restore_last_workspace(&mut self, enabled: bool) {
        let message = if enabled {
            "restore last workspace: on".to_string()
        } else {
            "restore last workspace: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.gui_restore_last_workspace = enabled,
            message,
        );
    }

    fn set_show_line_numbers(&mut self, enabled: bool) {
        let message = if enabled {
            "line numbers: on".to_string()
        } else {
            "line numbers: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.show_line_numbers = enabled,
            message,
        );
    }

    fn set_wrap_lines(&mut self, enabled: bool) {
        let message = if enabled {
            "wrap text: on".to_string()
        } else {
            "wrap text: off".to_string()
        };
        self.update_settings_with_rollback(|settings| settings.wrap_lines = enabled, message);
    }

    fn set_search_case_sensitive(&mut self, enabled: bool) {
        let message = if enabled {
            "search case sensitive: on".to_string()
        } else {
            "search case sensitive: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.search_case_sensitive = enabled,
            message,
        );
        self.search_highlight = None;
    }

    fn set_reader_mode_enabled(&mut self, enabled: bool) {
        let message = if enabled {
            "reader mode: on".to_string()
        } else {
            "reader mode: off".to_string()
        };
        self.reader_scroll_accumulator = 0.0;
        self.update_settings_with_rollback(
            |settings| settings.gui_reader_mode_enabled = enabled,
            message,
        );
    }

    fn toggle_reader_mode(&mut self) {
        self.set_reader_mode_enabled(!self.settings.gui_reader_mode_enabled);
    }

    fn set_reader_speed(&mut self, lines_per_minute: u16) {
        if !(MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE)
            .contains(&lines_per_minute)
        {
            self.status_message = format!(
                "reader speed must be {MIN_GUI_READER_LINES_PER_MINUTE}-{MAX_GUI_READER_LINES_PER_MINUTE} lines/min"
            );
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_reader_lines_per_minute = lines_per_minute,
            format!("reader speed: {lines_per_minute} lines/min"),
        );
    }

    fn cycle_gui_font_family(&mut self) {
        let next = self.settings.gui_font_family.next();
        self.update_settings_with_rollback(
            |settings| settings.gui_font_family = next,
            format!("font: {}", next.display_label()),
        );
    }

    fn set_gui_font_size(&mut self, size: u16) {
        if !(MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&size) {
            self.status_message =
                format!("editor font size must be {MIN_GUI_FONT_SIZE}-{MAX_GUI_FONT_SIZE}");
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_font_size = size,
            format!("editor font size: {size}"),
        );
    }

    fn set_gui_ui_font_size(&mut self, size: u16) {
        if !(MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&size) {
            self.status_message =
                format!("ui font size must be {MIN_GUI_FONT_SIZE}-{MAX_GUI_FONT_SIZE}");
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_ui_font_size = size,
            format!("ui font size: {size}"),
        );
    }

    fn show_path_prompt(&mut self, prompt: GuiPathPrompt) {
        self.path_prompt = Some(prompt);
        self.path_prompt_value = match prompt {
            GuiPathPrompt::Open => String::new(),
            GuiPathPrompt::SaveAs => self
                .workspace
                .active_tile()
                .document
                .path
                .display()
                .to_string(),
            GuiPathPrompt::ManagedNote => String::new(),
            GuiPathPrompt::BrowserCreateFile => String::new(),
            GuiPathPrompt::BrowserCreateDirectory => String::new(),
        };
        self.status_message = match prompt {
            GuiPathPrompt::Open => "open path".to_string(),
            GuiPathPrompt::SaveAs => "save as path".to_string(),
            GuiPathPrompt::ManagedNote => "managed note title".to_string(),
            GuiPathPrompt::BrowserCreateFile => "create file name".to_string(),
            GuiPathPrompt::BrowserCreateDirectory => "create directory name".to_string(),
        };
    }

    fn cancel_path_prompt(&mut self) {
        self.path_prompt = None;
        self.path_prompt_value.clear();
        self.notes_panel = None;
        self.pending_managed_note_delete = None;
        self.status_message = "path prompt canceled".to_string();
    }

    fn submit_path_prompt(&mut self) {
        let Some(prompt) = self.path_prompt else {
            return;
        };
        let raw_path = self.path_prompt_value.trim().to_string();
        if raw_path.is_empty() {
            self.status_message = match prompt {
                GuiPathPrompt::Open => "open failed: path required".to_string(),
                GuiPathPrompt::SaveAs => "save as failed: path required".to_string(),
                GuiPathPrompt::ManagedNote => "managed note failed: title required".to_string(),
                GuiPathPrompt::BrowserCreateFile => "create file failed: name required".to_string(),
                GuiPathPrompt::BrowserCreateDirectory => {
                    "create directory failed: name required".to_string()
                }
            };
            return;
        }

        let success = match prompt {
            GuiPathPrompt::Open => {
                let path = self.resolve_prompt_path(&raw_path);
                self.open_path_in_new_pane(path)
            }
            GuiPathPrompt::SaveAs => {
                let path = self.resolve_prompt_path(&raw_path);
                self.save_active_tile_as(path)
            }
            GuiPathPrompt::ManagedNote => self.open_managed_note_by_title(&raw_path),
            GuiPathPrompt::BrowserCreateFile => self.create_browser_file_named(&raw_path),
            GuiPathPrompt::BrowserCreateDirectory => self.create_browser_directory_named(&raw_path),
        };
        if success {
            self.path_prompt = None;
            self.path_prompt_value.clear();
            self.notes_panel = None;
            self.pending_managed_note_delete = None;
        }
    }

    fn resolve_prompt_path(&self, raw_path: &str) -> PathBuf {
        let path = PathBuf::from(raw_path);
        if path.is_absolute() {
            path
        } else {
            self.current_browser_dir().join(path)
        }
    }

    fn current_browser_dir(&self) -> PathBuf {
        self.browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
            .unwrap_or_else(|| self.current_dir.clone())
    }

    fn toggle_left_panel(&mut self) {
        self.left_panel.toggle_visibility();
        self.browser_visible = self.left_panel.visible;
        self.status_message = if self.left_panel.visible {
            format!(
                "{} panel shown",
                self.left_panel.title().to_ascii_lowercase()
            )
        } else {
            format!(
                "{} panel hidden",
                self.left_panel.title().to_ascii_lowercase()
            )
        };
        self.persist_layout();
    }

    fn select_left_panel_mode(&mut self, mode: GuiLeftPanelMode) {
        match mode {
            GuiLeftPanelMode::Files => self.left_panel.show_files(),
            GuiLeftPanelMode::Workspaces => self.left_panel.show_workspaces(),
            GuiLeftPanelMode::Preferences => self.left_panel.show_preferences(),
        }
        self.browser_visible = self.left_panel.visible;
        self.status_message = format!(
            "{} panel shown",
            self.left_panel.title().to_ascii_lowercase()
        );
        self.persist_layout();
    }

    fn refresh_workspace_projects(&mut self) {
        self.pending_project_delete = None;
        let Some(projects_dir) = self.workspace_projects_dir.as_deref() else {
            self.workspace_projects.clear();
            self.status_message =
                "workspace projects unavailable: cannot resolve config directory".to_string();
            return;
        };
        match list_gui_workspace_projects(projects_dir) {
            Ok(projects) => {
                let count = projects.len();
                self.workspace_projects = projects;
                self.status_message = format!("workspace projects: {count}");
            }
            Err(error) => {
                self.workspace_projects.clear();
                self.status_message = format!("workspace projects failed: {error}");
            }
        }
    }

    fn delete_workspace_project(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        if self.pending_project_delete != Some(index) {
            self.pending_project_delete = Some(index);
            self.pending_project_open = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = if self.is_current_workspace_project(&entry)
                && self.settings.gui_restore_last_workspace
            {
                "restore target selected; delete again to remove last-workspace restore project"
                    .to_string()
            } else {
                format!(
                    "delete workspace project {}? click delete again",
                    entry.project.name
                )
            };
            return;
        }

        self.pending_project_delete = None;
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            self.status_message =
                "workspace delete failed: cannot resolve config directory".to_string();
            return;
        };

        match delete_gui_workspace_project(&projects_dir, &entry.path) {
            Ok(GuiWorkspaceProjectDeleteResult::Deleted) => {
                self.refresh_workspace_projects();
                self.status_message = format!("workspace project deleted: {}", entry.project.name);
            }
            Ok(GuiWorkspaceProjectDeleteResult::Missing) => {
                self.refresh_workspace_projects();
                self.status_message =
                    format!("workspace project already missing: {}", entry.project.name);
            }
            Err(error) => {
                self.status_message = format!("workspace delete failed: {error}");
            }
        }
    }

    fn is_current_workspace_project(&self, entry: &GuiWorkspaceProjectEntry) -> bool {
        self.workspace_projects_dir
            .as_deref()
            .and_then(|projects_dir| gui_workspace_project_path(projects_dir, "current workspace"))
            .is_some_and(|current_path| current_path == entry.path)
    }

    fn open_workspace_project_in_new_window(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        match self.spawn_workspace_project_window(&entry.path) {
            Ok(()) => {
                self.status_message = format!(
                    "opened workspace project {} in new window",
                    entry.project.name
                );
            }
            Err(error) => {
                self.status_message = format!("workspace project new window failed: {error}");
            }
        }
    }

    #[cfg(test)]
    fn spawn_workspace_project_window(&mut self, path: &Path) -> io::Result<()> {
        self.spawned_workspace_project_paths
            .push(path.to_path_buf());
        Ok(())
    }

    #[cfg(not(test))]
    fn spawn_workspace_project_window(&mut self, path: &Path) -> io::Result<()> {
        let executable = env::current_exe()?;
        workspace_project_launch_command(&executable, path)
            .spawn()
            .map(|_| ())
    }

    fn save_current_workspace_project(&mut self) {
        self.save_workspace_project_named("current workspace", "current workspace");
    }

    fn save_named_workspace_project(&mut self) {
        let name = self.workspace_project_name.trim().to_string();
        if name.is_empty() {
            self.status_message = "workspace save failed: project name required".to_string();
            return;
        }
        self.save_workspace_project_named(&name, &name);
    }

    fn save_workspace_project_named(&mut self, project_name: &str, status_name: &str) {
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            self.status_message =
                "workspace save failed: cannot resolve config directory".to_string();
            return;
        };
        let Some(project_path) = gui_workspace_project_path(&projects_dir, project_name) else {
            self.status_message = "workspace save failed: invalid project name".to_string();
            return;
        };
        let Some(project) = self.current_workspace_project(project_name) else {
            self.status_message = "workspace save failed: cannot capture layout".to_string();
            return;
        };

        match save_gui_workspace_project(&project_path, &project) {
            Ok(()) => {
                self.refresh_workspace_projects();
                self.status_message = format!("workspace project saved: {status_name}");
            }
            Err(error) => {
                self.status_message = format!("workspace save failed: {error}");
            }
        }
    }

    fn current_workspace_project(&self, project_name: &str) -> Option<GuiWorkspaceProject> {
        let layout = gui_layout_from_state(
            &self.panes,
            &self.workspace,
            self.browser_visible,
            self.browser_width,
        )?;
        let active_ordinal = self
            .workspace
            .tiles
            .iter()
            .position(|tile| tile.id == self.workspace.active)
            .unwrap_or(0);
        Some(GuiWorkspaceProject {
            name: project_name.to_string(),
            files: self
                .workspace
                .tiles
                .iter()
                .map(|tile| tile.document.path.clone())
                .collect(),
            active_ordinal,
            layout: Some(layout),
        })
    }

    fn persist_last_workspace_if_enabled(&mut self) {
        if !self.settings.gui_restore_last_workspace {
            return;
        }
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            return;
        };
        let Some(project_path) = gui_workspace_project_path(&projects_dir, "current workspace")
        else {
            return;
        };
        let Some(project) = self.current_workspace_project("current workspace") else {
            self.status_message = "workspace autosave failed: cannot capture layout".to_string();
            return;
        };
        if let Err(error) = save_gui_workspace_project(&project_path, &project) {
            self.status_message = format!("workspace autosave failed: {error}");
        }
    }

    fn open_workspace_project_in_current_window(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        if self.has_dirty_tile() && self.pending_project_open != Some(index) {
            self.pending_project_open = Some(index);
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message =
                "unsaved changes; open project again to replace current workspace".to_string();
            return;
        }

        let mut documents = Vec::new();
        for path in &entry.project.files {
            match open_text_file(path) {
                Ok(document) => documents.push(document),
                Err(error) => {
                    self.pending_project_open = None;
                    self.status_message =
                        format!("workspace project open failed: {}: {error}", path.display());
                    return;
                }
            }
        }
        let mut documents = documents.into_iter();
        let Some(first_document) = documents.next() else {
            self.pending_project_open = None;
            self.status_message = "workspace project open failed: empty project".to_string();
            return;
        };

        let mut workspace = GuiWorkspace::from_document(first_document);
        let mut pane_states = vec![GuiPane::new(
            workspace.active,
            text_editor::Content::with_text(&workspace.active_tile().document.buffer.to_text()),
        )];
        for document in documents {
            let editor = text_editor::Content::with_text(&document.buffer.to_text());
            let tile_id = workspace.open_tile(document);
            pane_states.push(GuiPane::new(tile_id, editor));
        }

        let (panes, mut active_pane) = if let Some(layout) = entry.project.layout.clone() {
            let (panes, pane) = panes_from_gui_layout(layout.clone(), pane_states);
            for ordinal in &layout.minimized_ordinals {
                if let Some(tile_id) = workspace.tiles.get(*ordinal).map(|tile| tile.id) {
                    workspace.set_tile_minimized(tile_id, true);
                }
            }
            self.browser_visible = layout.browser_visible;
            self.left_panel.visible = layout.browser_visible;
            if let Some(width) = layout.browser_width_px {
                self.browser_width = clamp_browser_width(f32::from(width));
            }
            (panes, pane)
        } else {
            default_panes(pane_states)
        };
        let (panes, minimized_panes, active_pane_after_minimize) =
            close_minimized_panes_into_tray(panes, &workspace, active_pane);
        active_pane = active_pane_after_minimize;

        if let Some(active_tile_id) = workspace
            .tiles
            .get(entry.project.active_ordinal)
            .map(|tile| tile.id)
        {
            workspace.focus_tile(active_tile_id);
            if let Some(pane) = pane_for_tile_id(&panes, active_tile_id) {
                active_pane = pane;
            }
        }

        self.workspace = workspace;
        self.panes = panes;
        self.active_pane = active_pane;
        self.minimized_panes = minimized_panes;
        self.pending_project_open = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.external_edit_locks.clear();
        self.refresh_all_file_snapshots();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
        self.status_message = format!("opened workspace project {}", entry.project.name);
        self.persist_layout();
    }

    fn run_menu_command(&mut self, command: GuiMenuCommand) -> Task<Message> {
        match command {
            GuiMenuCommand::NewTile => self.create_new_tile(),
            GuiMenuCommand::Open => return self.request_open_dialog(),
            GuiMenuCommand::OpenPath => self.show_path_prompt(GuiPathPrompt::Open),
            GuiMenuCommand::Save => self.save_active_tile(),
            GuiMenuCommand::SaveAs => return self.request_save_as_dialog(),
            GuiMenuCommand::SaveAsPath => self.show_path_prompt(GuiPathPrompt::SaveAs),
            GuiMenuCommand::ClosePane => self.close_active_pane(),
            GuiMenuCommand::Quit => return window::latest().map(Message::QuitLatestWindow),
            GuiMenuCommand::OpenManagedNote => self.show_path_prompt(GuiPathPrompt::ManagedNote),
            GuiMenuCommand::ListManagedNotes => self.list_managed_notes_panel(),
            GuiMenuCommand::Copy => return self.copy_active_selection(),
            GuiMenuCommand::Cut => return self.cut_active_selection(),
            GuiMenuCommand::Paste => {
                self.status_message = "paste requested".to_string();
                return clipboard::read().map(Message::ClipboardPasted);
            }
            GuiMenuCommand::Undo => self.undo_active_edit(),
            GuiMenuCommand::Redo => self.redo_active_edit(),
            GuiMenuCommand::SelectAll => self.select_all_active_editor(),
            GuiMenuCommand::FindNext => self.search_active(false),
            GuiMenuCommand::FindPrevious => self.search_active(true),
            GuiMenuCommand::ToggleBrowser => self.toggle_left_panel(),
            GuiMenuCommand::CycleTheme => self.cycle_theme(),
            GuiMenuCommand::CycleSyntaxTheme => self.cycle_syntax_theme(),
            GuiMenuCommand::ToggleReaderMode => self.toggle_reader_mode(),
            GuiMenuCommand::GoDocumentStart => self.go_active_document_start(),
            GuiMenuCommand::GoDocumentEnd => self.go_active_document_end(),
            GuiMenuCommand::GoToLine => self.go_active_line(),
            GuiMenuCommand::ToggleMinimize => self.toggle_active_minimize(),
            GuiMenuCommand::ToggleMaximize => self.toggle_active_maximize(),
            GuiMenuCommand::EqualizeTiles => self.equalize_tile_layout(),
            GuiMenuCommand::MoveLeft => self.move_active_pane(pane_grid::Direction::Left),
            GuiMenuCommand::MoveRight => self.move_active_pane(pane_grid::Direction::Right),
            GuiMenuCommand::MoveUp => self.move_active_pane(pane_grid::Direction::Up),
            GuiMenuCommand::MoveDown => self.move_active_pane(pane_grid::Direction::Down),
            GuiMenuCommand::OpenHelp => self.open_help_document(),
        }
        if matches!(
            command,
            GuiMenuCommand::NewTile
                | GuiMenuCommand::ClosePane
                | GuiMenuCommand::ToggleMinimize
                | GuiMenuCommand::ToggleMaximize
                | GuiMenuCommand::EqualizeTiles
                | GuiMenuCommand::MoveLeft
                | GuiMenuCommand::MoveRight
                | GuiMenuCommand::MoveUp
                | GuiMenuCommand::MoveDown
        ) {
            self.persist_last_workspace_if_enabled();
        }
        Task::none()
    }

    fn persist_settings(&mut self) {
        let Some(config_path) = self.config_path.as_deref() else {
            return;
        };
        if let Err(error) = save_editor_settings(config_path, self.settings) {
            self.status_message = format!("settings save failed: {error}");
        }
    }

    fn persist_layout(&mut self) {
        let Some(layout_path) = self.layout_path.as_deref() else {
            return;
        };
        let Some(layout) = gui_layout_from_state(
            &self.panes,
            &self.workspace,
            self.browser_visible,
            self.browser_width,
        ) else {
            return;
        };
        if let Err(error) = save_gui_layout(layout_path, &layout) {
            self.status_message = format!("layout save failed: {error}");
        }
    }
}

fn empty_document(current_dir: PathBuf) -> TextDocument {
    TextDocument {
        path: current_dir.join("untitled.txt"),
        buffer: TextBuffer::from_text(""),
    }
}

#[cfg(not(test))]
fn current_editor_config_path() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    editor_config_path(xdg_config_home.as_deref(), home.as_deref())
}

#[cfg(not(test))]
fn current_gui_layout_path() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    gui_layout_path(xdg_config_home.as_deref(), home.as_deref())
}

#[cfg(not(test))]
fn current_gui_workspace_projects_dir() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    gui_workspace_projects_dir(xdg_config_home.as_deref(), home.as_deref())
}

#[cfg(not(test))]
fn current_gui_workspace_project_launch_path() -> Option<PathBuf> {
    env::var_os(WORKSPACE_PROJECT_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn gui_file_snapshot(path: &Path) -> io::Result<Option<GuiFileSnapshot>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return Ok(None);
            }
            Ok(Some(GuiFileSnapshot {
                len: metadata.len(),
                modified: metadata.modified().ok(),
            }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn load_workspace_project_launch(path: &Path) -> Result<GuiWorkspaceProject, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_gui_workspace_project(&text).ok_or_else(|| "invalid workspace project".to_string())
}

fn load_workspace_project_launch_documents(
    path: &Path,
) -> Result<(GuiWorkspaceProject, Vec<TextDocument>), String> {
    let project = load_workspace_project_launch(path)?;
    let mut documents = Vec::new();
    for file_path in &project.files {
        let document = open_text_file(file_path)
            .map_err(|error| format!("{}: {error}", file_path.display()))?;
        documents.push(document);
    }
    Ok((project, documents))
}

fn workspace_project_launch_command(executable: &Path, project_path: &Path) -> Command {
    let mut command = Command::new(executable);
    command.env(WORKSPACE_PROJECT_ENV, project_path);
    command
}

fn current_managed_notes_dir() -> Result<PathBuf, kfnotepad::ManagedNotesError> {
    let xdg_data_home = env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    managed_notes_dir(xdg_data_home.as_deref(), home.as_deref())
}

fn load_gui_layout(path: &std::path::Path, pane_count: usize) -> Option<GuiLayout> {
    let text = fs::read_to_string(path).ok()?;
    parse_gui_layout(&text, pane_count)
}

fn default_panes(mut pane_states: Vec<GuiPane>) -> (pane_grid::State<GuiPane>, pane_grid::Pane) {
    let first = pane_states.remove(0);
    let (mut panes, mut active_pane) = pane_grid::State::new(first);
    for pane_state in pane_states {
        let split_axis = split_axis_for_pane(&panes, active_pane);
        if let Some((pane, _split)) = panes.split(split_axis, active_pane, pane_state) {
            active_pane = pane;
        }
    }
    (panes, active_pane)
}

fn close_minimized_panes_into_tray(
    mut panes: pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
    mut active_pane: pane_grid::Pane,
) -> (pane_grid::State<GuiPane>, Vec<GuiPane>, pane_grid::Pane) {
    let minimized = panes
        .iter()
        .filter_map(|(pane, pane_state)| {
            workspace
                .tile(pane_state.tile_id)
                .and_then(|tile| tile.minimized.then_some(*pane))
        })
        .collect::<Vec<_>>();
    let mut tray = Vec::new();

    for pane in minimized {
        if panes.len() <= 1 {
            break;
        }
        if let Some((pane_state, sibling)) = panes.close(pane) {
            if active_pane == pane {
                active_pane = sibling;
            }
            tray.push(pane_state);
        }
    }

    (panes, tray, active_pane)
}

fn split_axis_for_pane(
    panes: &pane_grid::State<GuiPane>,
    pane: pane_grid::Pane,
) -> pane_grid::Axis {
    let Some(region) = panes
        .layout()
        .pane_regions(
            GUI_PANE_GRID_SPACING,
            GUI_PANE_GRID_MIN_SIZE,
            GUI_PANE_GRID_REFERENCE_SIZE,
        )
        .get(&pane)
        .copied()
    else {
        return pane_grid::Axis::Vertical;
    };

    if region.height > region.width {
        pane_grid::Axis::Horizontal
    } else {
        pane_grid::Axis::Vertical
    }
}

fn equalized_tile_layout_node(count: usize) -> Option<GuiLayoutNode> {
    if count == 0 {
        return None;
    }
    let columns = (count as f64).sqrt().ceil() as usize;
    let rows = count.div_ceil(columns);
    let mut next_ordinal = 0usize;
    let mut column_nodes = Vec::new();

    for _column in 0..columns {
        let remaining = count.saturating_sub(next_ordinal);
        if remaining == 0 {
            break;
        }
        let column_len = remaining.min(rows);
        let row_nodes = (0..column_len)
            .map(|_| {
                let ordinal = next_ordinal;
                next_ordinal += 1;
                GuiLayoutNode::Leaf { ordinal }
            })
            .collect::<Vec<_>>();
        column_nodes.push(equalized_axis_node(GuiLayoutAxis::Horizontal, row_nodes));
    }

    Some(equalized_axis_node(GuiLayoutAxis::Vertical, column_nodes))
}

fn equalized_axis_node(axis: GuiLayoutAxis, mut nodes: Vec<GuiLayoutNode>) -> GuiLayoutNode {
    assert!(
        !nodes.is_empty(),
        "equalized layout axis needs at least one child"
    );
    if nodes.len() == 1 {
        return nodes.remove(0);
    }

    let right = nodes.pop().expect("checked non-empty nodes");
    let left_count = nodes.len();
    let total = left_count + 1;
    let ratio_per_mille = ((left_count * 1000) / total).clamp(1, 999) as u16;
    GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first: Box::new(equalized_axis_node(axis, nodes)),
        second: Box::new(right),
    }
}

fn panes_from_gui_layout(
    layout: GuiLayout,
    pane_states: Vec<GuiPane>,
) -> (pane_grid::State<GuiPane>, pane_grid::Pane) {
    let mut pane_states = pane_states.into_iter().map(Some).collect::<Vec<_>>();
    let first_ordinal = first_layout_ordinal(&layout.root);
    let first_state = pane_states
        .get_mut(first_ordinal)
        .and_then(Option::take)
        .expect("parsed GUI layout root ordinal must match pane states");
    let (mut panes, first_pane) = pane_grid::State::new(first_state);
    apply_gui_layout_node(&layout.root, first_pane, &mut pane_states, &mut panes);
    for pane_state in pane_states {
        assert!(
            pane_state.is_none(),
            "parsed GUI layout must use every pane"
        );
    }
    (panes, first_pane)
}

fn first_layout_ordinal(node: &GuiLayoutNode) -> usize {
    match node {
        GuiLayoutNode::Leaf { ordinal } => *ordinal,
        GuiLayoutNode::Split { first, .. } => first_layout_ordinal(first),
    }
}

fn apply_gui_layout_node(
    node: &GuiLayoutNode,
    pane: pane_grid::Pane,
    pane_states: &mut [Option<GuiPane>],
    panes: &mut pane_grid::State<GuiPane>,
) -> pane_grid::Pane {
    match node {
        GuiLayoutNode::Leaf { .. } => pane,
        GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } => {
            let second_ordinal = first_layout_ordinal(second);
            let second_state = pane_states
                .get_mut(second_ordinal)
                .and_then(Option::take)
                .expect("parsed GUI layout child ordinal must match pane states");
            let (second_pane, split) = panes
                .split(iced_axis(*axis), pane, second_state)
                .expect("parsed GUI layout split target must exist");
            panes.resize(split, f32::from(*ratio_per_mille) / 1000.0);
            apply_gui_layout_node(first, pane, pane_states, panes);
            apply_gui_layout_node(second, second_pane, pane_states, panes);
            pane
        }
    }
}

fn gui_layout_from_state(
    panes: &pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
    browser_visible: bool,
    browser_width: f32,
) -> Option<GuiLayout> {
    let minimized_ordinals = workspace
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(ordinal, tile)| tile.minimized.then_some(ordinal))
        .collect::<Vec<_>>();
    let visible_root = gui_layout_node_from_iced(panes.layout(), panes, workspace)?;
    let root = gui_layout_with_minimized_leaves(visible_root, &minimized_ordinals);

    Some(GuiLayout {
        browser_visible,
        browser_width_px: Some(persisted_browser_width(browser_width)),
        root,
        minimized_ordinals,
    })
}

fn gui_layout_with_minimized_leaves(
    mut root: GuiLayoutNode,
    minimized_ordinals: &[usize],
) -> GuiLayoutNode {
    for ordinal in minimized_ordinals {
        root = GuiLayoutNode::Split {
            axis: GuiLayoutAxis::Vertical,
            ratio_per_mille: 500,
            first: Box::new(root),
            second: Box::new(GuiLayoutNode::Leaf { ordinal: *ordinal }),
        };
    }
    root
}

fn clamp_browser_width(width: f32) -> f32 {
    width.clamp(GUI_BROWSER_WIDTH_MIN, GUI_BROWSER_WIDTH_MAX)
}

fn persisted_browser_width(width: f32) -> u16 {
    clamp_browser_width(width).round() as u16
}

fn gui_layout_node_from_iced(
    node: &pane_grid::Node,
    panes: &pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
) -> Option<GuiLayoutNode> {
    match node {
        pane_grid::Node::Pane(pane) => {
            let tile_id = panes.get(*pane)?.tile_id;
            let ordinal = workspace.tiles.iter().position(|tile| tile.id == tile_id)?;
            Some(GuiLayoutNode::Leaf { ordinal })
        }
        pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } => Some(GuiLayoutNode::Split {
            axis: gui_layout_axis(*axis),
            ratio_per_mille: ((*ratio * 1000.0).round() as u16).clamp(1, 999),
            first: Box::new(gui_layout_node_from_iced(a, panes, workspace)?),
            second: Box::new(gui_layout_node_from_iced(b, panes, workspace)?),
        }),
    }
}

fn iced_axis(axis: GuiLayoutAxis) -> pane_grid::Axis {
    match axis {
        GuiLayoutAxis::Horizontal => pane_grid::Axis::Horizontal,
        GuiLayoutAxis::Vertical => pane_grid::Axis::Vertical,
    }
}

fn gui_layout_axis(axis: pane_grid::Axis) -> GuiLayoutAxis {
    match axis {
        pane_grid::Axis::Horizontal => GuiLayoutAxis::Horizontal,
        pane_grid::Axis::Vertical => GuiLayoutAxis::Vertical,
    }
}

fn pane_for_tile_id(
    panes: &pane_grid::State<GuiPane>,
    tile_id: GuiTileId,
) -> Option<pane_grid::Pane> {
    panes
        .iter()
        .find_map(|(pane, pane_state)| (pane_state.tile_id == tile_id).then_some(*pane))
}

fn update(state: &mut KfnotepadGui, message: Message) -> Task<Message> {
    match message {
        Message::Edit(pane, action) => {
            if !state.focus_pane(pane) {
                return Task::none();
            }
            let is_scroll = matches!(action, text_editor::Action::Scroll { .. });
            let is_edit = action.is_edit();
            let Some(tile_id) = state.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
                return Task::none();
            };
            if state.is_external_edit_locked(tile_id) && !is_scroll {
                state.status_message = "external edit lock active; unlock to edit".to_string();
                return Task::none();
            }
            state.search_highlight = None;
            if let Some(pane_state) = state.panes.get_mut(pane) {
                pane_state
                    .editor
                    .apply(GuiEditorCommand::IcedAction(action));
            }
            state.sync_pane_to_document(pane);
            if is_edit {
                state.workspace.clear_tile_save_error(tile_id);
                state.external_edit_locks.remove(&tile_id);
                state.invalidate_syntax_cache(tile_id);
                state.ensure_visible_syntax_cache_for_tile(tile_id);
                state.pending_close_tile = None;
                state.pending_app_quit = false;
                state.pending_project_open = None;
                state.status_message = "modified".to_string();
            } else if is_scroll {
                state.ensure_visible_syntax_cache_for_tile(tile_id);
                state.status_message = "scrolled".to_string();
            }
        }
        Message::BrowserTreeEvent(event) => return state.handle_browser_tree_event(event),
        Message::BrowserLocalTreeToggle(path) => {
            state.toggle_local_browser_tree_path(path);
        }
        Message::BrowserLocalTreeSelected(path, is_dir) => {
            return state.select_local_browser_tree_path(path, is_dir);
        }
        Message::BrowserLocalTreeActivated(path, is_dir) => {
            return state.activate_local_browser_tree_path(path, is_dir);
        }
        Message::BrowserParentRequested => return state.navigate_browser_parent(),
        Message::BrowserRefreshRequested => return state.refresh_file_browser(),
        Message::BrowserCreateFileRequested => {
            state.show_path_prompt(GuiPathPrompt::BrowserCreateFile)
        }
        Message::BrowserCreateDirectoryRequested => {
            state.show_path_prompt(GuiPathPrompt::BrowserCreateDirectory)
        }
        Message::BrowserDeleteSelectedRequested => return state.delete_selected_browser_entry(),
        Message::BrowserWidthChanged(width) => {
            state.browser_width = clamp_browser_width(width);
            state.status_message = format!("file browser width: {:.0}px", state.browser_width);
            state.persist_layout();
            state.persist_last_workspace_if_enabled();
        }
        Message::SelectLeftPanelMode(mode) => state.select_left_panel_mode(mode),
        Message::ToggleBrowser => state.toggle_left_panel(),
        Message::PaneClicked(pane) => {
            state.focus_pane(pane);
        }
        Message::PaneResized(event) => {
            state.panes.resize(event.split, event.ratio);
            state.persist_layout();
            state.persist_last_workspace_if_enabled();
        }
        Message::PaneDragged(event) => {
            state.drag_pane(event);
        }
        Message::NewTileRequested => {
            state.create_new_tile();
            state.persist_last_workspace_if_enabled();
        }
        Message::OpenPromptRequested => return state.request_open_dialog(),
        Message::OpenDialogSelected(path) => {
            state.handle_open_dialog_selected(path);
            state.persist_last_workspace_if_enabled();
        }
        Message::SaveAsPromptRequested => return state.request_save_as_dialog(),
        Message::SaveAsDialogSelected(path) => state.handle_save_as_dialog_selected(path),
        Message::ManagedNoteClicked(index) => state.open_managed_note_from_panel(index),
        Message::ManagedNoteDeleteClicked(index) => state.delete_managed_note_from_panel(index),
        Message::ExternalFileCheckTick => state.poll_external_file_changes(),
        Message::ReaderScrollTick => state.reader_scroll_tick(),
        Message::UnlockExternalEdit(tile_id) => state.unlock_external_edit(tile_id),
        Message::WorkspaceProjectClicked(index) => {
            state.open_workspace_project_in_current_window(index);
        }
        Message::WorkspaceProjectNewWindowClicked(index) => {
            state.open_workspace_project_in_new_window(index);
        }
        Message::WorkspaceProjectDeleteClicked(index) => {
            state.delete_workspace_project(index);
        }
        Message::SaveCurrentWorkspaceProject => state.save_current_workspace_project(),
        Message::WorkspaceProjectNameChanged(name) => {
            state.workspace_project_name = name;
        }
        Message::SaveNamedWorkspaceProject => state.save_named_workspace_project(),
        Message::RestoreLastWorkspaceChanged(enabled) => {
            state.set_restore_last_workspace(enabled);
            state.persist_last_workspace_if_enabled();
        }
        Message::ShowLineNumbersChanged(enabled) => {
            state.set_show_line_numbers(enabled);
        }
        Message::WrapLinesChanged(enabled) => {
            state.set_wrap_lines(enabled);
        }
        Message::SearchCaseSensitiveChanged(enabled) => {
            state.set_search_case_sensitive(enabled);
        }
        Message::ReaderModeChanged(enabled) => {
            state.set_reader_mode_enabled(enabled);
        }
        Message::ReaderSpeedChanged(lines_per_minute) => {
            state.set_reader_speed(lines_per_minute);
        }
        Message::CycleGuiFontFamily => {
            state.cycle_gui_font_family();
        }
        Message::GuiFontSizeChanged(size) => {
            state.set_gui_font_size(size);
        }
        Message::GuiUiFontSizeChanged(size) => {
            state.set_gui_ui_font_size(size);
        }
        Message::RefreshWorkspaceProjects => state.refresh_workspace_projects(),
        Message::PathPromptChanged(path) => {
            state.path_prompt_value = path;
        }
        Message::SubmitPathPrompt => state.submit_path_prompt(),
        Message::CancelPathPrompt => state.cancel_path_prompt(),
        Message::SaveRequested => state.save_active_tile(),
        Message::ClosePane(pane) => {
            state.close_pane(pane);
            state.persist_last_workspace_if_enabled();
        }
        Message::CloseActivePane => {
            state.close_active_pane();
            state.persist_last_workspace_if_enabled();
        }
        Message::QuitRequested(window_id) => return state.request_app_close(window_id),
        Message::ToggleMinimizePane(pane) => {
            state.toggle_pane_minimized(pane);
            state.persist_last_workspace_if_enabled();
        }
        Message::RestoreMinimizedTile(tile_id) => {
            state.restore_minimized_tile(tile_id);
            state.persist_last_workspace_if_enabled();
        }
        Message::ToggleActiveMinimize => {
            state.toggle_active_minimize();
            state.persist_last_workspace_if_enabled();
        }
        Message::ToggleActiveMaximize => {
            state.toggle_active_maximize();
            state.persist_last_workspace_if_enabled();
        }
        Message::ToggleMaximizePane(pane) => {
            state.toggle_pane_maximized(pane);
            state.persist_last_workspace_if_enabled();
        }
        Message::MoveActivePane(direction) => {
            state.move_active_pane(direction);
            state.persist_last_workspace_if_enabled();
        }
        Message::MovePane(pane, direction) => {
            state.move_pane(pane, direction);
            state.persist_last_workspace_if_enabled();
        }
        Message::CycleTheme => state.cycle_theme(),
        Message::CycleSyntaxTheme => state.cycle_syntax_theme(),
        Message::SearchQueryChanged(query) => {
            state.search_query = query;
            state.search_history_open =
                state.search_query.is_empty() && !state.search_history.is_empty();
            state.search_highlight = None;
        }
        Message::SearchHistorySelected(query) => state.select_search_history(query),
        Message::GoToLineQueryChanged(query) => {
            state.go_to_line_query = query;
        }
        Message::SearchNext => state.search_active(false),
        Message::SearchPrevious => state.search_active(true),
        Message::GoDocumentStart => {
            state.search_highlight = None;
            state.go_active_document_start();
        }
        Message::GoDocumentEnd => {
            state.search_highlight = None;
            state.go_active_document_end();
        }
        Message::GoToLineRequested => {
            state.search_highlight = None;
            state.go_active_line();
        }
        Message::ScrollActiveEditorViewport(delta) => state.scroll_active_editor_viewport(delta),
        Message::ReplacementEditorWheelScrolled(pane, delta) => {
            state.scroll_replacement_editor_pane_viewport(pane, delta);
        }
        Message::ReplacementEditorInputs(inputs) => {
            if GUI_USE_READ_ONLY_EDITOR_RENDERER {
                let Some(tile_id) = state
                    .panes
                    .get(state.active_pane)
                    .map(|pane_state| pane_state.tile_id)
                else {
                    return Task::none();
                };
                if state.is_external_edit_locked(tile_id) {
                    state.status_message = "external edit lock active; unlock to edit".to_string();
                    return Task::none();
                }
                state.search_highlight = None;
                state.apply_replacement_editor_inputs_to_active_tile(inputs);
            } else {
                state.status_message = "replacement editor inactive".to_string();
            }
        }
        Message::ReplacementEditorIme(event) => {
            if GUI_USE_READ_ONLY_EDITOR_RENDERER {
                state.apply_replacement_editor_ime_event(event);
            } else {
                state.status_message = "replacement editor inactive".to_string();
            }
        }
        Message::ToggleReplacementOverwriteMode => {
            state.replacement_overwrite_mode = !state.replacement_overwrite_mode;
            state.status_message = if state.replacement_overwrite_mode {
                "overwrite mode".to_string()
            } else {
                "insert mode".to_string()
            };
        }
        Message::ReplacementEditorPointerMoved(pane, point) => {
            state.replacement_editor_pointer_moved(pane, point);
        }
        Message::ReplacementEditorBodyPointerMoved(pane, point, edge) => {
            state.replacement_editor_body_pointer_moved(pane, point, edge);
        }
        Message::ReplacementEditorPointerPressed(pane) => {
            state.replacement_editor_pointer_pressed(pane);
        }
        Message::ReplacementEditorPointerReleased(pane) => {
            state.replacement_editor_pointer_released(pane);
        }
        Message::ReplacementEditorDragTick => {
            state.replacement_editor_drag_tick();
        }
        Message::ReplacementEditorGlobalPointerReleased => {
            state.replacement_editor_global_pointer_released();
        }
        Message::ReplacementEditorScrollbarMoved(pane, y, model) => {
            state.replacement_editor_scrollbar_moved(pane, y, model);
        }
        Message::ReplacementEditorScrollbarPressed(pane) => {
            state.replacement_editor_scrollbar_pressed(pane);
        }
        Message::ReplacementEditorScrollbarReleased(pane) => {
            state.replacement_editor_scrollbar_released(pane);
        }
        Message::MenuCommand(command) => return state.run_menu_command(command),
        Message::ClipboardPasted(contents) => state.paste_into_active_editor(contents),
        Message::QuitLatestWindow(Some(window_id)) => return state.request_app_close(window_id),
        Message::QuitLatestWindow(None) => {
            state.status_message = "quit failed: no active window".to_string();
        }
        Message::WindowCloseRequested(window_id) => return state.request_app_close(window_id),
    }

    Task::none()
}

fn subscription(state: &KfnotepadGui) -> Subscription<Message> {
    let replacement_drag_active =
        state.replacement_drag.is_some() || state.replacement_scrollbar_drag.is_some();
    let mut subscriptions = vec![
        event::listen_with(|event, status, window_id| match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::ReplacementEditorGlobalPointerReleased)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.is_empty() && matches!(key.as_ref(), Key::Named(Named::Insert)) =>
            {
                Some(Message::ToggleReplacementOverwriteMode)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'o') =>
            {
                Some(Message::OpenPromptRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("o" | "O")) =>
            {
                Some(Message::OpenPromptRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 's') =>
            {
                Some(Message::SaveAsPromptRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("s" | "S")) =>
            {
                Some(Message::SaveAsPromptRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'n') =>
            {
                Some(Message::NewTileRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("n" | "N")) =>
            {
                Some(Message::NewTileRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 's') =>
            {
                Some(Message::SaveRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("s" | "S")) =>
            {
                Some(Message::SaveRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'b') =>
            {
                Some(Message::ToggleBrowser)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("b" | "B")) =>
            {
                Some(Message::ToggleBrowser)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::F4)) =>
            {
                Some(Message::CloseActivePane)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'q') =>
            {
                Some(Message::QuitRequested(window_id))
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("q" | "Q")) =>
            {
                Some(Message::QuitRequested(window_id))
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'f') =>
            {
                Some(Message::SearchNext)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("f" | "F")) =>
            {
                Some(Message::SearchNext)
            }
            Event::Keyboard(event) if gui_editor_clipboard_shortcut_command(&event).is_some() => {
                gui_editor_clipboard_shortcut_command(&event).map(Message::MenuCommand)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if matches!(key.as_ref(), Key::Named(Named::F3)) && modifiers.shift() =>
            {
                Some(Message::SearchPrevious)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if matches!(key.as_ref(), Key::Named(Named::F3)) && modifiers.is_empty() =>
            {
                Some(Message::SearchNext)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::Home)) =>
            {
                Some(Message::GoDocumentStart)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::End)) =>
            {
                Some(Message::GoDocumentEnd)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::PageUp)) =>
            {
                Some(Message::ScrollActiveEditorViewport(
                    -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
                ))
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::PageDown)) =>
            {
                Some(Message::ScrollActiveEditorViewport(
                    GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
                ))
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'g') =>
            {
                Some(Message::GoToLineRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && matches!(key.as_ref(), Key::Character("g" | "G")) =>
            {
                Some(Message::GoToLineRequested)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'm') =>
            {
                Some(Message::ToggleActiveMaximize)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("m" | "M")) =>
            {
                Some(Message::ToggleActiveMaximize)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && !modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'm') =>
            {
                Some(Message::ToggleActiveMinimize)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && !modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("m" | "M")) =>
            {
                Some(Message::ToggleActiveMinimize)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 't') =>
            {
                Some(Message::CycleSyntaxTheme)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("t" | "T")) =>
            {
                Some(Message::CycleSyntaxTheme)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && !modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 't') =>
            {
                Some(Message::CycleTheme)
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && !modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("t" | "T")) =>
            {
                Some(Message::CycleTheme)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }) if modifiers.control()
                && !modifiers.shift()
                && key
                    .to_latin(physical_key)
                    .is_some_and(|character| character == 'r') =>
            {
                Some(Message::MenuCommand(GuiMenuCommand::ToggleReaderMode))
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control()
                    && !modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("r" | "R")) =>
            {
                Some(Message::MenuCommand(GuiMenuCommand::ToggleReaderMode))
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.control() && modifiers.shift() =>
            {
                match key.as_ref() {
                    Key::Named(Named::ArrowLeft) => {
                        Some(Message::MoveActivePane(pane_grid::Direction::Left))
                    }
                    Key::Named(Named::ArrowRight) => {
                        Some(Message::MoveActivePane(pane_grid::Direction::Right))
                    }
                    Key::Named(Named::ArrowUp) => {
                        Some(Message::MoveActivePane(pane_grid::Direction::Up))
                    }
                    Key::Named(Named::ArrowDown) => {
                        Some(Message::MoveActivePane(pane_grid::Direction::Down))
                    }
                    _ => None,
                }
            }
            Event::Keyboard(event)
                if GUI_USE_READ_ONLY_EDITOR_RENDERER
                    && matches!(status, iced::event::Status::Ignored) =>
            {
                let inputs = gui_editor_replacement_inputs_from_keyboard_event(&event);
                (!inputs.is_empty()).then_some(Message::ReplacementEditorInputs(inputs))
            }
            Event::InputMethod(event)
                if GUI_USE_READ_ONLY_EDITOR_RENDERER
                    && matches!(status, iced::event::Status::Ignored) =>
            {
                Some(Message::ReplacementEditorIme(event))
            }
            _ => None,
        }),
        window::close_requests().map(Message::WindowCloseRequested),
        iced::time::every(Duration::from_secs(1)).map(|_| Message::ExternalFileCheckTick),
    ];
    if state.settings.gui_reader_mode_enabled {
        subscriptions.push(
            iced::time::every(Duration::from_millis(GUI_READER_TICK_MS))
                .map(|_| Message::ReaderScrollTick),
        );
    }
    if replacement_drag_active {
        subscriptions.push(
            iced::time::every(Duration::from_millis(GUI_REPLACEMENT_DRAG_TICK_MS))
                .map(|_| Message::ReplacementEditorDragTick),
        );
    }
    Subscription::batch(subscriptions)
}

fn title(state: &KfnotepadGui) -> String {
    format!(
        "kfnotepad-gui - {}",
        state.workspace.active_tile().document.path.display()
    )
}

fn theme(state: &KfnotepadGui) -> Theme {
    gui_theme(state.settings.theme_id)
}

fn gui_theme(theme_id: EditorThemeId) -> Theme {
    Theme::custom(
        format!("kfnotepad {}", theme_id.label()),
        gui_theme_palette(theme_id),
    )
}

fn gui_theme_palette(theme_id: EditorThemeId) -> iced::theme::Palette {
    match theme_id {
        EditorThemeId::Nocturne => iced::theme::Palette {
            background: color(10, 12, 24),
            text: color(226, 232, 246),
            primary: color(92, 119, 255),
            success: color(56, 189, 126),
            warning: color(244, 202, 94),
            danger: color(255, 91, 112),
        },
        EditorThemeId::Aurora => iced::theme::Palette {
            background: color(6, 20, 24),
            text: color(224, 252, 241),
            primary: color(35, 211, 171),
            success: color(111, 232, 123),
            warning: color(255, 218, 92),
            danger: color(255, 99, 132),
        },
        EditorThemeId::Paper => iced::theme::Palette {
            background: color(245, 226, 244),
            text: color(34, 24, 48),
            primary: color(118, 67, 169),
            success: color(35, 128, 105),
            warning: color(139, 83, 31),
            danger: color(155, 48, 96),
        },
        EditorThemeId::Terminal => iced::theme::Palette {
            background: color(0, 18, 7),
            text: color(177, 255, 177),
            primary: color(72, 255, 112),
            success: color(44, 220, 96),
            warning: color(255, 228, 92),
            danger: color(255, 92, 92),
        },
        EditorThemeId::Abyss => iced::theme::Palette {
            background: color(3, 7, 18),
            text: color(206, 240, 255),
            primary: color(102, 229, 255),
            success: color(79, 209, 197),
            warning: color(252, 211, 77),
            danger: color(255, 64, 96),
        },
        EditorThemeId::Terror => iced::theme::Palette {
            background: color(24, 0, 30),
            text: color(255, 188, 236),
            primary: color(255, 42, 160),
            success: color(112, 255, 128),
            warning: color(255, 238, 70),
            danger: color(255, 58, 58),
        },
    }
}

fn gui_highlighter_theme(theme_id: EditorThemeId) -> highlighter::Theme {
    match theme_id {
        EditorThemeId::Paper => highlighter::Theme::InspiredGitHub,
        EditorThemeId::Terminal => highlighter::Theme::Base16Mocha,
        EditorThemeId::Abyss | EditorThemeId::Nocturne => highlighter::Theme::Base16Ocean,
        EditorThemeId::Aurora => highlighter::Theme::SolarizedDark,
        EditorThemeId::Terror => highlighter::Theme::Base16Eighties,
    }
}

fn gui_editor_font(font_family: GuiFontFamily) -> Font {
    match font_family {
        GuiFontFamily::Monospace => Font::MONOSPACE,
        GuiFontFamily::SansSerif => Font::DEFAULT,
        GuiFontFamily::Serif => Font {
            family: iced::font::Family::Serif,
            ..Font::DEFAULT
        },
        GuiFontFamily::JetBrainsMono => Font::with_name("JetBrains Mono"),
        GuiFontFamily::FiraCode => Font::with_name("Fira Code"),
    }
}

fn gui_editor_wrapping(wrap_lines: bool) -> Wrapping {
    if wrap_lines {
        Wrapping::WordOrGlyph
    } else {
        Wrapping::None
    }
}

fn gui_editor_effective_wrapping(wrap_lines: bool, show_line_numbers: bool) -> Wrapping {
    let _ = show_line_numbers;
    gui_editor_wrapping(wrap_lines)
}

fn gui_editor_surface_model<'a>(
    settings: EditorSettings,
    document: &TextDocument,
    editor: &'a GuiEditorAdapter,
    syntax_highlighter: &SyntaxHighlighter,
    syntax_cache: Option<&GuiSyntaxCache>,
) -> GuiEditorSurfaceModel<'a> {
    let render_state =
        editor.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, settings.gui_font_size);
    let render_viewport_slice = editor
        .render_viewport_slice_from_lines(document.buffer.lines(), GUI_EDITOR_RENDER_LINE_BUDGET);
    let viewport_slice =
        gui_editor_viewport_slice_with_cached_syntax(render_viewport_slice, syntax_cache);
    GuiEditorSurfaceModel {
        content: render_state.content,
        editor_font: gui_editor_font(settings.gui_font_family),
        editor_size: u32::from(settings.gui_font_size),
        wrapping: gui_editor_effective_wrapping(settings.wrap_lines, settings.show_line_numbers),
        syntax_token: syntax_highlighter.syntax_token_for_document(document),
        highlighter_theme: gui_highlighter_theme(settings.syntax_theme_id),
        line_numbers: settings
            .show_line_numbers
            .then_some(render_state.line_numbers),
        viewport_slice,
    }
}

fn gui_ui_font_size(settings: EditorSettings) -> u32 {
    settings.gui_ui_font_size.into()
}

fn gui_ui_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
}

fn gui_ui_small_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_sub(2)
        .max(MIN_GUI_FONT_SIZE.into())
}

fn gui_ui_heading_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_add(4)
        .min(MAX_GUI_FONT_SIZE.into())
}

fn gui_ui_icon_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_add(1)
        .min(MAX_GUI_FONT_SIZE.into())
}

fn gui_ui_tooltip_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_sub(2)
        .max(MIN_GUI_FONT_SIZE.into())
}

fn gui_line_number_gutter_text(
    first_line: usize,
    line_count: usize,
    visible_lines: usize,
) -> String {
    let total = line_count.max(1);
    let start = first_line.clamp(1, total);
    let end = (start + visible_lines.saturating_sub(1)).min(total);

    (start..=end)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn gui_line_number_gutter_width(line_count: usize, editor_font_size: u16) -> f32 {
    let digits = line_count.max(1).to_string().len() as f32;
    let digit_width = f32::from(editor_font_size) * 0.62;
    GUI_LINE_NUMBER_GUTTER_HORIZONTAL_PADDING + digits * digit_width
}

#[cfg(test)]
fn gui_editor_viewport_slice(
    text: &str,
    line_count: usize,
    viewport: GuiEditorViewportState,
    cursor: DocumentCursor,
    selection: Option<GuiEditorReplacementSelection>,
) -> GuiEditorViewportSlice {
    let document_lines = gui_document_lines(text, line_count);
    gui_editor_viewport_slice_from_lines(&document_lines, line_count, viewport, cursor, selection)
}

fn gui_editor_viewport_slice_from_lines(
    document_lines: &[String],
    line_count: usize,
    viewport: GuiEditorViewportState,
    cursor: DocumentCursor,
    selection: Option<GuiEditorReplacementSelection>,
) -> GuiEditorViewportSlice {
    let total = line_count.max(1);
    let first_line = viewport.first_line.clamp(1, total);
    let last_line = viewport.last_visible_line(total);

    let lines = (first_line..=last_line)
        .map(|number| {
            let row = number.saturating_sub(1);
            GuiEditorViewportLine {
                number,
                text: document_lines.get(row).cloned().unwrap_or_default(),
                cursor_column: (cursor.row == row).then_some(cursor.column),
                syntax_segments: None,
                selection: gui_editor_viewport_selection_span(
                    document_lines
                        .get(row)
                        .map(String::as_str)
                        .unwrap_or_default(),
                    row,
                    selection,
                ),
            }
        })
        .collect();

    GuiEditorViewportSlice {
        line_count: total,
        first_line,
        lines,
    }
}

#[cfg(test)]
fn gui_document_lines(text: &str, line_count: usize) -> Vec<String> {
    let total = line_count.max(1);
    let mut lines = text
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    lines.resize(total, String::new());
    lines.truncate(total);
    lines
}

fn gui_editor_viewport_slice_with_cached_syntax(
    mut slice: GuiEditorViewportSlice,
    syntax_cache: Option<&GuiSyntaxCache>,
) -> GuiEditorViewportSlice {
    let Some(cache) = syntax_cache else {
        return slice;
    };

    for line in &mut slice.lines {
        let row = line.number.saturating_sub(1);
        line.syntax_segments = cache.lines.get(row).cloned().flatten();
    }

    slice
}

fn gui_syntax_segments_from_syntect(
    segments: Vec<(SyntectStyle, String)>,
    theme_id: EditorThemeId,
) -> Vec<GuiEditorSyntaxSegment> {
    segments
        .into_iter()
        .filter_map(|(style, text)| {
            (!text.is_empty()).then_some(GuiEditorSyntaxSegment {
                text,
                color: gui_color_from_syntect(style.foreground, theme_id),
            })
        })
        .collect()
}

fn gui_color_from_syntect(color: syntect::highlighting::Color, theme_id: EditorThemeId) -> Color {
    let (r, g, b) = gui_syntax_rgb_for_theme(color.r, color.g, color.b, theme_id);
    Color::from_rgba8(r, g, b, f32::from(color.a) / 255.0)
}

fn gui_syntax_rgb_for_theme(red: u8, green: u8, blue: u8, theme_id: EditorThemeId) -> (u8, u8, u8) {
    let role = gui_syntax_color_role(red, green, blue);
    let rgb = gui_syntax_role_rgb(theme_id, role);
    gui_ensure_syntax_contrast_rgb(rgb, gui_theme_palette(theme_id).background)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GuiSyntaxColorRole {
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

fn gui_syntax_color_role(red: u8, green: u8, blue: u8) -> GuiSyntaxColorRole {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let chroma = max.saturating_sub(min);
    let luminance = 0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue);

    if chroma < 24 {
        return if luminance < 150.0 {
            GuiSyntaxColorRole::Comment
        } else {
            GuiSyntaxColorRole::Text
        };
    }

    let hue = gui_rgb_hue_degrees(red, green, blue);
    if !(25.0..345.0).contains(&hue) {
        GuiSyntaxColorRole::Rose
    } else if hue < 55.0 {
        GuiSyntaxColorRole::Orange
    } else if hue < 78.0 {
        GuiSyntaxColorRole::Yellow
    } else if hue < 160.0 {
        GuiSyntaxColorRole::Green
    } else if hue < 200.0 {
        GuiSyntaxColorRole::Cyan
    } else if hue < 255.0 {
        GuiSyntaxColorRole::Blue
    } else if hue < 315.0 {
        GuiSyntaxColorRole::Purple
    } else {
        GuiSyntaxColorRole::Rose
    }
}

fn gui_rgb_hue_degrees(red: u8, green: u8, blue: u8) -> f32 {
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

fn gui_syntax_role_rgb(theme_id: EditorThemeId, role: GuiSyntaxColorRole) -> (u8, u8, u8) {
    match theme_id {
        EditorThemeId::Nocturne => match role {
            GuiSyntaxColorRole::Text => (213, 224, 246),
            GuiSyntaxColorRole::Comment => (126, 141, 170),
            GuiSyntaxColorRole::Rose => (255, 126, 154),
            GuiSyntaxColorRole::Orange => (246, 177, 116),
            GuiSyntaxColorRole::Yellow => (238, 213, 122),
            GuiSyntaxColorRole::Green => (139, 222, 160),
            GuiSyntaxColorRole::Cyan => (99, 221, 224),
            GuiSyntaxColorRole::Blue => (132, 172, 255),
            GuiSyntaxColorRole::Purple => (202, 158, 255),
        },
        EditorThemeId::Aurora => match role {
            GuiSyntaxColorRole::Text => (218, 255, 241),
            GuiSyntaxColorRole::Comment => (112, 162, 156),
            GuiSyntaxColorRole::Rose => (255, 129, 162),
            GuiSyntaxColorRole::Orange => (255, 183, 112),
            GuiSyntaxColorRole::Yellow => (245, 224, 128),
            GuiSyntaxColorRole::Green => (104, 241, 151),
            GuiSyntaxColorRole::Cyan => (65, 234, 217),
            GuiSyntaxColorRole::Blue => (119, 198, 255),
            GuiSyntaxColorRole::Purple => (208, 151, 255),
        },
        EditorThemeId::Paper => match role {
            GuiSyntaxColorRole::Text => (80, 67, 91),
            GuiSyntaxColorRole::Comment => (119, 105, 130),
            GuiSyntaxColorRole::Rose => (154, 62, 100),
            GuiSyntaxColorRole::Orange => (158, 87, 48),
            GuiSyntaxColorRole::Yellow => (125, 94, 20),
            GuiSyntaxColorRole::Green => (45, 116, 93),
            GuiSyntaxColorRole::Cyan => (37, 111, 126),
            GuiSyntaxColorRole::Blue => (67, 89, 153),
            GuiSyntaxColorRole::Purple => (118, 72, 156),
        },
        EditorThemeId::Terminal => match role {
            GuiSyntaxColorRole::Text => (168, 255, 176),
            GuiSyntaxColorRole::Comment => (83, 165, 95),
            GuiSyntaxColorRole::Rose => (255, 126, 126),
            GuiSyntaxColorRole::Orange => (247, 186, 96),
            GuiSyntaxColorRole::Yellow => (240, 250, 127),
            GuiSyntaxColorRole::Green => (80, 255, 119),
            GuiSyntaxColorRole::Cyan => (113, 255, 207),
            GuiSyntaxColorRole::Blue => (142, 215, 255),
            GuiSyntaxColorRole::Purple => (205, 168, 255),
        },
        EditorThemeId::Abyss => match role {
            GuiSyntaxColorRole::Text => (214, 244, 255),
            GuiSyntaxColorRole::Comment => (100, 132, 158),
            GuiSyntaxColorRole::Rose => (255, 97, 137),
            GuiSyntaxColorRole::Orange => (255, 169, 111),
            GuiSyntaxColorRole::Yellow => (241, 218, 111),
            GuiSyntaxColorRole::Green => (111, 230, 172),
            GuiSyntaxColorRole::Cyan => (93, 239, 255),
            GuiSyntaxColorRole::Blue => (126, 174, 255),
            GuiSyntaxColorRole::Purple => (196, 145, 255),
        },
        EditorThemeId::Terror => match role {
            GuiSyntaxColorRole::Text => (255, 193, 238),
            GuiSyntaxColorRole::Comment => (157, 103, 148),
            GuiSyntaxColorRole::Rose => (255, 62, 166),
            GuiSyntaxColorRole::Orange => (255, 120, 75),
            GuiSyntaxColorRole::Yellow => (255, 226, 82),
            GuiSyntaxColorRole::Green => (91, 255, 141),
            GuiSyntaxColorRole::Cyan => (90, 238, 230),
            GuiSyntaxColorRole::Blue => (136, 172, 255),
            GuiSyntaxColorRole::Purple => (221, 97, 255),
        },
    }
}

fn gui_mix_syntax_rgb(
    red: u8,
    green: u8,
    blue: u8,
    target: (f32, f32, f32),
    mix: f32,
) -> (u8, u8, u8) {
    (
        (f32::from(red) * (1.0 - mix) + target.0 * mix).round() as u8,
        (f32::from(green) * (1.0 - mix) + target.1 * mix).round() as u8,
        (f32::from(blue) * (1.0 - mix) + target.2 * mix).round() as u8,
    )
}

fn gui_ensure_syntax_contrast_rgb(mut rgb: (u8, u8, u8), background: Color) -> (u8, u8, u8) {
    let background_rgb = gui_color_to_rgb(background);
    if gui_contrast_ratio(rgb, background_rgb) >= 4.5 {
        return rgb;
    }

    let background_luminance = gui_relative_luminance(background_rgb);
    let target = if background_luminance > 0.5 {
        (28.0, 24.0, 36.0)
    } else {
        (238.0, 248.0, 255.0)
    };
    for _ in 0..8 {
        rgb = gui_mix_syntax_rgb(rgb.0, rgb.1, rgb.2, target, 0.25);
        if gui_contrast_ratio(rgb, background_rgb) >= 4.5 {
            break;
        }
    }
    rgb
}

fn gui_color_to_rgb(color: Color) -> (u8, u8, u8) {
    (
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

fn gui_contrast_ratio(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f32 {
    let foreground = gui_relative_luminance(foreground);
    let background = gui_relative_luminance(background);
    let lighter = foreground.max(background);
    let darker = foreground.min(background);
    (lighter + 0.05) / (darker + 0.05)
}

fn gui_relative_luminance((red, green, blue): (u8, u8, u8)) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)
}

fn gui_editor_viewport_selection_span(
    line: &str,
    row: usize,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<GuiEditorSelectionSpan> {
    let (start, end) = selection?.normalized();
    if start == end || row < start.row || row > end.row {
        return None;
    }

    let line_columns = line.chars().count();
    let start_column = if row == start.row {
        start.column.min(line_columns)
    } else {
        0
    };
    let end_column = if row == end.row {
        end.column.min(line_columns)
    } else {
        line_columns
    };

    if row == start.row && row == end.row && start_column == end_column {
        return None;
    }

    Some(GuiEditorSelectionSpan {
        start_column,
        end_column,
    })
}

#[cfg(test)]
fn gui_editor_read_only_render_model(
    slice: &GuiEditorViewportSlice,
) -> GuiEditorReadOnlyRenderModel {
    let cursor = slice
        .lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| line.cursor_column.map(|column| (index, column)));

    GuiEditorReadOnlyRenderModel {
        line_count: slice.line_count,
        first_line: slice.first_line,
        gutter_text: slice
            .lines
            .iter()
            .map(|line| line.number.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        body_text: slice
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        cursor_row_in_view: cursor.map(|(row, _column)| row),
        cursor_column: cursor.map(|(_row, column)| column),
    }
}

fn gui_editor_read_only_line_segments(
    line: &GuiEditorViewportLine,
) -> Vec<GuiEditorReadOnlyLineSegment> {
    let line_columns = line.text.chars().count();
    let syntax_colors = gui_editor_line_syntax_colors(line);
    let overlay = line
        .selection
        .map(|selection| {
            (
                selection.start_column.min(line_columns),
                selection.end_column.min(line_columns),
            )
        })
        .or_else(|| {
            line.cursor_column.map(|cursor_column| {
                let cursor = cursor_column.min(line_columns);
                (cursor, cursor.saturating_add(1).min(line_columns))
            })
        });

    if line_columns == 0 {
        return vec![GuiEditorReadOnlyLineSegment {
            text: if overlay.is_some() {
                " ".to_string()
            } else {
                String::new()
            },
            selected: overlay.is_some(),
            syntax_color: None,
        }];
    }

    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_selected = false;
    let mut current_color = None;

    for (index, character) in line.text.chars().enumerate() {
        let selected =
            overlay.is_some_and(|(start, end)| index >= start && index < end.max(start + 1));
        let syntax_color = if selected {
            None
        } else {
            syntax_colors.get(index).copied().flatten()
        };

        if current_text.is_empty() {
            current_selected = selected;
            current_color = syntax_color;
        } else if current_selected != selected || current_color != syntax_color {
            gui_editor_push_read_only_segment(
                &mut segments,
                &mut current_text,
                current_selected,
                current_color,
            );
            current_selected = selected;
            current_color = syntax_color;
        }
        current_text.push(character);
    }

    gui_editor_push_read_only_segment(
        &mut segments,
        &mut current_text,
        current_selected,
        current_color,
    );

    if overlay.is_some_and(|(start, end)| start == end && start >= line_columns) {
        segments.push(GuiEditorReadOnlyLineSegment {
            text: " ".to_string(),
            selected: true,
            syntax_color: None,
        });
    }

    segments
}

fn gui_editor_read_only_visual_rows(
    lines: &[GuiEditorViewportLine],
    first_line: usize,
    wrapping: Wrapping,
    body_columns: usize,
) -> Vec<GuiEditorReadOnlyVisualRow> {
    let mut rows = Vec::new();
    let wrap_columns = body_columns.max(1);

    for line in lines {
        let viewport_row = line.number.saturating_sub(first_line);
        let ranges = if wrapping == Wrapping::None {
            vec![(0, line.text.chars().count())]
        } else {
            gui_editor_word_wrap_ranges(&line.text, wrap_columns)
        };

        for (index, (start, end)) in ranges.into_iter().enumerate() {
            rows.push(GuiEditorReadOnlyVisualRow {
                line: gui_editor_viewport_line_slice(line, start, end),
                viewport_row,
                source_column_start: start,
                show_line_number: index == 0,
            });
        }
    }

    rows
}

fn gui_editor_word_wrap_ranges(text: &str, max_columns: usize) -> Vec<(usize, usize)> {
    let max_columns = max_columns.max(1);
    let chars = text.chars().collect::<Vec<_>>();
    let len = chars.len();
    if len == 0 {
        return vec![(0, 0)];
    }

    let mut ranges = Vec::new();
    let mut start = 0;
    while start < len {
        let hard_end = gui_editor_display_width_hard_end(&chars, start, max_columns);
        if hard_end >= len {
            ranges.push((start, len));
            break;
        }

        let break_at = (start + 1..hard_end)
            .rev()
            .find(|index| chars[*index].is_whitespace())
            .filter(|index| index.saturating_sub(start) >= max_columns / 3)
            .map(|index| index + 1)
            .unwrap_or(hard_end);
        ranges.push((start, break_at.max(start + 1)));
        start = break_at.max(start + 1);
    }

    ranges
}

fn gui_editor_display_width_hard_end(chars: &[char], start: usize, max_columns: usize) -> usize {
    let mut end = start;
    let mut width = 0usize;
    while end < chars.len() {
        let next_width = gui_editor_char_display_width(chars[end]);
        if end > start && width.saturating_add(next_width) > max_columns {
            break;
        }
        width = width.saturating_add(next_width);
        end += 1;
    }

    end.max(start.saturating_add(1)).min(chars.len())
}

fn gui_editor_char_display_width(character: char) -> usize {
    if character == '\t' {
        GUI_TAB_WIDTH
    } else {
        UnicodeWidthChar::width(character).unwrap_or(0)
    }
}

fn gui_editor_char_column_from_pixel_x(text: &str, x: f32, character_width: f32) -> usize {
    let x = x.max(0.0);
    let character_width = character_width.max(1.0);
    let mut display_width = 0usize;

    for (column, character) in text.chars().enumerate() {
        let char_width = gui_editor_char_display_width(character).max(1);
        let start = display_width as f32 * character_width;
        let end = display_width.saturating_add(char_width) as f32 * character_width;
        let midpoint = start + (end - start) / 2.0;
        if x < midpoint {
            return column;
        }
        if x < end {
            return column + 1;
        }
        display_width = display_width.saturating_add(char_width);
    }

    text.chars().count()
}

fn gui_editor_viewport_line_slice(
    line: &GuiEditorViewportLine,
    start: usize,
    end: usize,
) -> GuiEditorViewportLine {
    let line_columns = line.text.chars().count();
    let start = start.min(line_columns);
    let end = end.min(line_columns).max(start);
    let row_text = line
        .text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    let cursor_column = line.cursor_column.and_then(|cursor| {
        if start == end {
            (cursor == start).then_some(0)
        } else if cursor >= start && cursor < end {
            Some(cursor - start)
        } else if end == line_columns && cursor == end {
            Some(end - start)
        } else {
            None
        }
    });
    let selection = line.selection.and_then(|selection| {
        let selected_start = selection.start_column.max(start);
        let selected_end = selection.end_column.min(end);
        (selected_start < selected_end).then_some(GuiEditorSelectionSpan {
            start_column: selected_start - start,
            end_column: selected_end - start,
        })
    });
    let syntax_segments = gui_editor_slice_syntax_segments(line, start, end);

    GuiEditorViewportLine {
        number: line.number,
        text: row_text,
        cursor_column,
        selection,
        syntax_segments,
    }
}

fn gui_editor_slice_syntax_segments(
    line: &GuiEditorViewportLine,
    start: usize,
    end: usize,
) -> Option<Vec<GuiEditorSyntaxSegment>> {
    line.syntax_segments.as_ref()?;
    if start == end {
        return Some(Vec::new());
    }
    let colors = gui_editor_line_syntax_colors(line);
    let line_chars = line.text.chars().collect::<Vec<_>>();
    let row_colors = colors.get(start..end)?;
    if row_colors.iter().any(Option::is_none) {
        return None;
    }

    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_color = row_colors.first().and_then(|color| *color)?;
    for (character, color) in line_chars[start..end]
        .iter()
        .copied()
        .zip(row_colors.iter())
    {
        let color = color.unwrap_or(current_color);
        if !current_text.is_empty() && color != current_color {
            segments.push(GuiEditorSyntaxSegment {
                text: std::mem::take(&mut current_text),
                color: current_color,
            });
            current_color = color;
        }
        current_text.push(character);
    }
    if !current_text.is_empty() {
        segments.push(GuiEditorSyntaxSegment {
            text: current_text,
            color: current_color,
        });
    }

    Some(segments)
}

fn gui_editor_viewport_line_with_ime_preedit(
    mut line: GuiEditorViewportLine,
    preedit: Option<&GuiImePreedit>,
) -> GuiEditorViewportLine {
    let Some(preedit) = preedit else {
        return line;
    };
    if preedit.content.is_empty() {
        return line;
    }
    let Some(cursor_column) = line.cursor_column else {
        return line;
    };

    let line_columns = line.text.chars().count();
    let cursor_column = cursor_column.min(line_columns);
    let before = line.text.chars().take(cursor_column).collect::<String>();
    let after = line.text.chars().skip(cursor_column).collect::<String>();
    let preedit_columns = preedit.content.chars().count();
    let (selected_start, selected_end) =
        gui_ime_preedit_selection_columns(preedit).unwrap_or((0, preedit_columns));

    line.text = format!("{before}{}{after}", preedit.content);
    line.cursor_column = None;
    line.selection = Some(GuiEditorSelectionSpan {
        start_column: cursor_column.saturating_add(selected_start.min(preedit_columns)),
        end_column: cursor_column
            .saturating_add(selected_end.min(preedit_columns))
            .max(cursor_column.saturating_add(1)),
    });
    line.syntax_segments = None;
    line
}

fn gui_ime_preedit_selection_columns(preedit: &GuiImePreedit) -> Option<(usize, usize)> {
    let selection = preedit.selection.as_ref()?;
    let start = gui_byte_index_to_char_column(&preedit.content, selection.start);
    let end = gui_byte_index_to_char_column(&preedit.content, selection.end).max(start);
    Some((start, end))
}

fn gui_byte_index_to_char_column(text: &str, byte_index: usize) -> usize {
    let clamped = byte_index.min(text.len());
    text.char_indices()
        .take_while(|(index, _)| *index < clamped)
        .count()
}

fn gui_editor_line_syntax_colors(line: &GuiEditorViewportLine) -> Vec<Option<Color>> {
    let mut colors = Vec::new();
    if let Some(segments) = &line.syntax_segments {
        for segment in segments {
            colors.extend(segment.text.chars().map(|_| Some(segment.color)));
        }
    }
    colors.resize(line.text.chars().count(), None);
    colors
}

fn gui_editor_push_read_only_segment(
    segments: &mut Vec<GuiEditorReadOnlyLineSegment>,
    text: &mut String,
    selected: bool,
    syntax_color: Option<Color>,
) {
    if text.is_empty() {
        return;
    }
    segments.push(GuiEditorReadOnlyLineSegment {
        text: std::mem::take(text),
        selected,
        syntax_color,
    });
}

fn apply_gui_editor_replacement_input(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    input: GuiEditorReplacementInput,
) {
    apply_gui_editor_replacement_input_with_mode(
        document, cursor, viewport, selection, false, input,
    );
}

fn apply_gui_editor_replacement_input_with_mode(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    overwrite_mode: bool,
    input: GuiEditorReplacementInput,
) {
    match input {
        GuiEditorReplacementInput::InsertChar(value) => {
            let deleted_selection =
                delete_gui_editor_replacement_selection(document, cursor, selection);
            if overwrite_mode && !deleted_selection {
                let _ = document.buffer.delete_char(cursor.row, cursor.column);
            }
            if document
                .buffer
                .insert_char(cursor.row, cursor.column, value)
                .is_ok()
            {
                cursor.column = cursor.column.saturating_add(1);
            }
        }
        GuiEditorReplacementInput::InsertNewline => {
            delete_gui_editor_replacement_selection(document, cursor, selection);
            if document
                .buffer
                .insert_newline(cursor.row, cursor.column)
                .is_ok()
            {
                cursor.row = cursor.row.saturating_add(1);
                cursor.column = 0;
            }
        }
        GuiEditorReplacementInput::DeleteBackward => {
            if delete_gui_editor_replacement_selection(document, cursor, selection) {
                // Selection deletion already positioned the cursor at the start of the range.
            } else if let Ok(next_cursor) = document.buffer.delete_before_cursor(*cursor) {
                *cursor = next_cursor;
            }
        }
        GuiEditorReplacementInput::DeleteForward => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = document.buffer.delete_char(cursor.row, cursor.column);
            }
        }
        GuiEditorReplacementInput::DeletePreviousWord => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_previous_word(document, cursor);
            }
        }
        GuiEditorReplacementInput::DeleteNextWord => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_next_word(document, cursor);
            }
        }
        GuiEditorReplacementInput::DeleteToLineEnd => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_to_line_end(document, cursor);
            }
        }
        GuiEditorReplacementInput::Move(direction) => {
            *selection = None;
            if let Ok(next_cursor) = document.buffer.move_cursor(*cursor, direction) {
                *cursor = next_cursor;
            }
        }
        GuiEditorReplacementInput::MoveLineStart => {
            *selection = None;
            cursor.column = 0;
        }
        GuiEditorReplacementInput::MoveLineEnd => {
            *selection = None;
            cursor.column = document
                .buffer
                .line_char_count(cursor.row)
                .unwrap_or(cursor.column);
        }
        GuiEditorReplacementInput::ScrollViewportLines(delta) => {
            viewport.scroll_by(delta, document.buffer.line_count());
            let visible_cursor =
                viewport.clamp_cursor_to_visible(*cursor, document.buffer.line_count());
            *cursor = visible_cursor;
        }
        GuiEditorReplacementInput::SelectAll => {
            let start = DocumentCursor { row: 0, column: 0 };
            let end = gui_editor_replacement_document_end_cursor(&document.buffer);
            *cursor = end;
            *selection = GuiEditorReplacementSelection::new(start, end);
        }
        GuiEditorReplacementInput::SelectRange { anchor, focus } => {
            if gui_editor_replacement_cursor_is_valid(&document.buffer, anchor)
                && gui_editor_replacement_cursor_is_valid(&document.buffer, focus)
            {
                *cursor = focus;
                *selection = GuiEditorReplacementSelection::new(anchor, focus);
            }
        }
        GuiEditorReplacementInput::ClearSelection => {
            *selection = None;
        }
    }
    viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
}

fn delete_gui_editor_replacement_selection(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    selection: &mut Option<GuiEditorReplacementSelection>,
) -> bool {
    let Some(active_selection) = selection.take() else {
        return false;
    };
    let (start, end) = active_selection.normalized();
    if gui_editor_replacement_selection_covers_full_text(document, start, end) {
        document.buffer.replace_text("");
        *cursor = DocumentCursor { row: 0, column: 0 };
        return true;
    }
    if gui_editor_replacement_delete_range(&mut document.buffer, start, end).is_ok() {
        *cursor = start;
        true
    } else {
        false
    }
}

fn gui_editor_replacement_document_end_cursor(buffer: &TextBuffer) -> DocumentCursor {
    let row = buffer.line_count().saturating_sub(1);
    DocumentCursor {
        row,
        column: buffer.line_char_count(row).unwrap_or_default(),
    }
}

fn gui_editor_replacement_selection_covers_full_text(
    document: &TextDocument,
    start: DocumentCursor,
    end: DocumentCursor,
) -> bool {
    start == (DocumentCursor { row: 0, column: 0 })
        && end == gui_editor_replacement_document_end_cursor(&document.buffer)
}

fn gui_editor_replacement_cursor_is_valid(buffer: &TextBuffer, cursor: DocumentCursor) -> bool {
    buffer
        .line_char_count(cursor.row)
        .is_ok_and(|columns| cursor.column <= columns)
}

fn validate_gui_editor_replacement_cursor(
    buffer: &TextBuffer,
    cursor: DocumentCursor,
) -> Result<(), kfnotepad::BufferError> {
    let columns = buffer.line_char_count(cursor.row)?;
    if cursor.column <= columns {
        Ok(())
    } else {
        Err(kfnotepad::BufferError::ColumnOutOfBounds {
            column: cursor.column,
            columns,
        })
    }
}

fn document_cursor_is_before_or_equal(left: DocumentCursor, right: DocumentCursor) -> bool {
    (left.row, left.column) <= (right.row, right.column)
}

#[allow(dead_code)]
fn gui_editor_replacement_selected_text(
    document: &TextDocument,
    selection: GuiEditorReplacementSelection,
) -> Option<String> {
    let (start, end) = selection.normalized();
    if !gui_editor_replacement_cursor_is_valid(&document.buffer, start)
        || !gui_editor_replacement_cursor_is_valid(&document.buffer, end)
    {
        return None;
    }

    if gui_editor_replacement_selection_covers_full_text(document, start, end) {
        return Some(document.buffer.to_text());
    }

    let lines = document.buffer.lines();
    if start.row == end.row {
        return lines
            .get(start.row)
            .map(|line| char_slice(line, start.column, end.column));
    }

    let mut selected = Vec::new();
    let first = lines.get(start.row)?;
    selected.push(char_suffix(first, start.column));
    for row in (start.row + 1)..end.row {
        selected.push(lines.get(row)?.to_string());
    }
    let last = lines.get(end.row)?;
    selected.push(char_prefix(last, end.column));
    Some(selected.join("\n"))
}

#[allow(dead_code)]
fn gui_editor_replacement_copy_selection(
    document: &TextDocument,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_selected_text(document, selection?)?;
    (!selected.is_empty()).then_some(selected)
}

#[allow(dead_code)]
fn gui_editor_replacement_cut_selection(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_copy_selection(document, *selection)?;
    delete_gui_editor_replacement_selection(document, cursor, selection);
    viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
    Some(selected)
}

#[allow(dead_code)]
fn gui_editor_replacement_paste_text(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    text: &str,
) {
    if text.is_empty() {
        return;
    }

    for character in text.chars() {
        let input = if character == '\n' {
            GuiEditorReplacementInput::InsertNewline
        } else {
            GuiEditorReplacementInput::InsertChar(character)
        };
        apply_gui_editor_replacement_input(document, cursor, viewport, selection, input);
    }
}

#[allow(dead_code)]
fn gui_editor_replacement_cursor_from_mouse_point(
    buffer: &TextBuffer,
    viewport: GuiEditorViewportState,
    point: GuiEditorReplacementMousePoint,
) -> DocumentCursor {
    let total = buffer.line_count().max(1);
    let row = viewport
        .first_line
        .saturating_sub(1)
        .saturating_add(point.viewport_row)
        .min(total.saturating_sub(1));
    let column = buffer
        .line_char_count(row)
        .map(|columns| point.column.min(columns))
        .unwrap_or_default();
    DocumentCursor { row, column }
}

#[cfg(test)]
fn gui_editor_replacement_mouse_point_from_line_point(
    point: iced::Point,
    viewport_row: usize,
    settings: EditorSettings,
    body_width: f32,
    wrapping: Wrapping,
) -> GuiEditorReplacementMousePoint {
    let character_width = (f32::from(settings.gui_font_size) * 0.62).max(1.0);
    let column_in_visual_row = (point.x.max(0.0) / character_width).floor() as usize;
    let visual_row = if wrapping == Wrapping::None {
        0
    } else {
        let line_height = (f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT).max(1.0);
        (point.y.max(0.0) / line_height).floor() as usize
    };
    let visual_row_columns = if wrapping == Wrapping::None {
        0
    } else {
        (body_width.max(character_width) / character_width)
            .floor()
            .max(1.0) as usize
    };

    GuiEditorReplacementMousePoint {
        viewport_row,
        column: visual_row
            .saturating_mul(visual_row_columns)
            .saturating_add(column_in_visual_row),
    }
}

fn gui_editor_replacement_mouse_point_from_visual_row_point(
    point: iced::Point,
    viewport_row: usize,
    source_column_start: usize,
    visual_row_text: &str,
    settings: EditorSettings,
) -> GuiEditorReplacementMousePoint {
    let column_in_visual_row = gui_editor_char_column_from_pixel_x(
        visual_row_text,
        point.x,
        gui_editor_replacement_character_width(settings),
    );
    GuiEditorReplacementMousePoint {
        viewport_row,
        column: source_column_start.saturating_add(column_in_visual_row),
    }
}

fn gui_editor_replacement_mouse_point_from_body_point(
    point: iced::Point,
    source_lines: &[GuiEditorViewportLine],
    first_line: usize,
    wrapping: Wrapping,
    hit_test: GuiEditorBodyHitTest,
    settings: EditorSettings,
) -> GuiEditorReplacementMousePoint {
    let text_point = iced::Point::new(point.x - hit_test.text_origin_x, point.y);
    let row_height = gui_editor_replacement_row_height(settings);
    let target_visual_row = (point.y.max(0.0) / row_height).floor() as usize;
    let visual_rows =
        gui_editor_read_only_visual_rows(source_lines, first_line, wrapping, hit_test.columns)
            .into_iter()
            .take(hit_test.visible_rows.max(1))
            .collect::<Vec<_>>();

    if let Some(visual_row) =
        visual_rows.get(target_visual_row.min(visual_rows.len().saturating_sub(1)))
    {
        return gui_editor_replacement_mouse_point_from_visual_row_point(
            text_point,
            visual_row.viewport_row,
            visual_row.source_column_start,
            &visual_row.line.text,
            settings,
        );
    }

    let display_column =
        (text_point.x.max(0.0) / gui_editor_replacement_character_width(settings)).floor() as usize;
    GuiEditorReplacementMousePoint {
        viewport_row: target_visual_row.min(hit_test.visible_rows.saturating_sub(1)),
        column: display_column,
    }
}

fn gui_editor_drag_edge_from_body_point(
    pane: pane_grid::Pane,
    point: iced::Point,
    surface_height: f32,
    point_column: usize,
    settings: EditorSettings,
) -> GuiEditorDragEdge {
    let edge_zone = gui_editor_replacement_row_height(settings).max(1.0);
    let direction = if point.y <= edge_zone {
        -1
    } else if point.y >= (surface_height - edge_zone).max(edge_zone) {
        1
    } else {
        0
    };
    GuiEditorDragEdge {
        pane,
        direction,
        column: point_column,
    }
}

fn gui_editor_replacement_character_width(settings: EditorSettings) -> f32 {
    (f32::from(settings.gui_font_size) * 0.62).max(1.0)
}

fn gui_editor_replacement_row_height(settings: EditorSettings) -> f32 {
    (f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT)
        .ceil()
        .max(1.0)
}

fn gui_editor_visible_row_budget(surface_height: f32, row_height: f32) -> usize {
    (surface_height.max(row_height) / row_height.max(1.0))
        .floor()
        .max(1.0) as usize
}

fn gui_editor_replacement_scroll_delta_lines(
    delta: mouse::ScrollDelta,
    settings: EditorSettings,
) -> i32 {
    let lines = match delta {
        mouse::ScrollDelta::Lines { y, .. } => -y,
        mouse::ScrollDelta::Pixels { y, .. } => {
            let line_height = gui_editor_replacement_row_height(settings);
            -(y / line_height)
        }
    };
    let rounded = lines.round() as i32;
    rounded.clamp(
        -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
        GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
    )
}

#[allow(dead_code)]
fn gui_editor_replacement_mouse_click(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    point: GuiEditorReplacementMousePoint,
) {
    *cursor = gui_editor_replacement_cursor_from_mouse_point(&document.buffer, *viewport, point);
    *selection = None;
}

#[allow(dead_code)]
fn gui_editor_replacement_mouse_drag(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    anchor: DocumentCursor,
    focus: GuiEditorReplacementMousePoint,
) {
    let focus = gui_editor_replacement_cursor_from_mouse_point(&document.buffer, *viewport, focus);
    *cursor = focus;
    *selection = GuiEditorReplacementSelection::new(anchor, focus);
}

fn gui_editor_replacement_delete_range(
    buffer: &mut TextBuffer,
    start: DocumentCursor,
    end: DocumentCursor,
) -> Result<(), kfnotepad::BufferError> {
    if !document_cursor_is_before_or_equal(start, end) {
        return gui_editor_replacement_delete_range(buffer, end, start);
    }
    if start == end {
        return Ok(());
    }
    validate_gui_editor_replacement_cursor(buffer, start)?;
    validate_gui_editor_replacement_cursor(buffer, end)?;

    let lines = buffer.lines();
    let mut replacement_lines = Vec::new();
    replacement_lines.extend(lines[..start.row].iter().cloned());

    if start.row == end.row {
        let line = lines
            .get(start.row)
            .ok_or(kfnotepad::BufferError::RowOutOfBounds {
                row: start.row,
                rows: lines.len(),
            })?;
        replacement_lines.push(format!(
            "{}{}",
            char_prefix(line, start.column),
            char_suffix(line, end.column)
        ));
    } else {
        let start_line = lines
            .get(start.row)
            .ok_or(kfnotepad::BufferError::RowOutOfBounds {
                row: start.row,
                rows: lines.len(),
            })?;
        let end_line = lines
            .get(end.row)
            .ok_or(kfnotepad::BufferError::RowOutOfBounds {
                row: end.row,
                rows: lines.len(),
            })?;
        replacement_lines.push(format!(
            "{}{}",
            char_prefix(start_line, start.column),
            char_suffix(end_line, end.column)
        ));
    }

    replacement_lines.extend(lines[(end.row + 1)..].iter().cloned());
    buffer.replace_text(&replacement_lines.join("\n"));
    Ok(())
}

fn char_prefix(value: &str, end_column: usize) -> String {
    value.chars().take(end_column).collect()
}

fn char_suffix(value: &str, start_column: usize) -> String {
    value.chars().skip(start_column).collect()
}

#[allow(dead_code)]
fn char_slice(value: &str, start_column: usize, end_column: usize) -> String {
    value
        .chars()
        .skip(start_column)
        .take(end_column.saturating_sub(start_column))
        .collect()
}

fn gui_editor_replacement_inputs_from_keyboard_event(
    event: &keyboard::Event,
) -> Vec<GuiEditorReplacementInput> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        text,
        ..
    } = event
    else {
        return Vec::new();
    };

    let modified_command = modifiers.control() || modifiers.alt() || modifiers.logo();
    if modifiers.control() && !modifiers.alt() && !modifiers.logo() {
        match key.as_ref() {
            Key::Character("k" | "K") => return vec![GuiEditorReplacementInput::DeleteToLineEnd],
            Key::Named(Named::ArrowLeft) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::WordLeft,
                )];
            }
            Key::Named(Named::ArrowRight) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::WordRight,
                )];
            }
            Key::Named(Named::Backspace) => {
                return vec![GuiEditorReplacementInput::DeletePreviousWord];
            }
            Key::Named(Named::Delete) => return vec![GuiEditorReplacementInput::DeleteNextWord],
            _ => {}
        }
    }
    if modifiers.control()
        && !modifiers.alt()
        && !modifiers.logo()
        && matches!(key.as_ref(), Key::Character("a" | "A"))
    {
        return vec![GuiEditorReplacementInput::SelectAll];
    }
    if !modified_command {
        match key.as_ref() {
            Key::Named(Named::Enter) => return vec![GuiEditorReplacementInput::InsertNewline],
            Key::Named(Named::Backspace) => {
                return vec![GuiEditorReplacementInput::DeleteBackward];
            }
            Key::Named(Named::Delete) => return vec![GuiEditorReplacementInput::DeleteForward],
            Key::Named(Named::Escape) => return vec![GuiEditorReplacementInput::ClearSelection],
            Key::Named(Named::Home) => return vec![GuiEditorReplacementInput::MoveLineStart],
            Key::Named(Named::End) => return vec![GuiEditorReplacementInput::MoveLineEnd],
            Key::Named(Named::ArrowLeft) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Left)];
            }
            Key::Named(Named::ArrowRight) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::Right,
                )];
            }
            Key::Named(Named::ArrowUp) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Up)];
            }
            Key::Named(Named::ArrowDown) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down)];
            }
            Key::Named(Named::PageUp) => {
                return vec![GuiEditorReplacementInput::ScrollViewportLines(
                    -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
                )];
            }
            Key::Named(Named::PageDown) => {
                return vec![GuiEditorReplacementInput::ScrollViewportLines(
                    GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
                )];
            }
            _ => {}
        }
    }

    if modified_command {
        return Vec::new();
    }

    gui_editor_replacement_inputs_from_text(text.as_deref().unwrap_or_default())
}

fn gui_editor_clipboard_shortcut_command(event: &keyboard::Event) -> Option<GuiMenuCommand> {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    } = event
    else {
        return None;
    };
    if !modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }

    let character = key.to_latin(*physical_key).or_else(|| match key.as_ref() {
        Key::Character(value) => value.chars().next().map(|value| value.to_ascii_lowercase()),
        _ => None,
    })?;

    match character.to_ascii_lowercase() {
        'c' => Some(GuiMenuCommand::Copy),
        'x' => Some(GuiMenuCommand::Cut),
        'v' => Some(GuiMenuCommand::Paste),
        'z' if modifiers.shift() => Some(GuiMenuCommand::Redo),
        'z' => Some(GuiMenuCommand::Undo),
        'y' => Some(GuiMenuCommand::Redo),
        _ => None,
    }
}

#[cfg(test)]
fn gui_editor_replacement_inputs_from_ime_event(
    event: &input_method::Event,
) -> Vec<GuiEditorReplacementInput> {
    match event {
        input_method::Event::Commit(text) => gui_editor_replacement_inputs_from_text(text),
        input_method::Event::Opened
        | input_method::Event::Preedit(_, _)
        | input_method::Event::Closed => Vec::new(),
    }
}

fn gui_editor_replacement_inputs_from_text(text: &str) -> Vec<GuiEditorReplacementInput> {
    text.chars()
        .filter(|value| !value.is_control())
        .map(GuiEditorReplacementInput::InsertChar)
        .collect()
}

fn gui_left_panel_width(visible: bool, configured_width: f32) -> f32 {
    if visible {
        clamp_browser_width(configured_width)
    } else {
        0.0
    }
}

#[derive(Debug)]
struct GuiTreeIconTheme;

impl IconTheme for GuiTreeIconTheme {
    fn glyph(&self, role: IconRole) -> IconSpec {
        gui_tree_icon_spec(role)
    }
}

fn gui_tree_icon_spec(role: IconRole) -> IconSpec {
    let glyph = match role {
        IconRole::FolderClosed => nf::cod::COD_FOLDER,
        IconRole::FolderOpen => nf::cod::COD_FOLDER_OPENED,
        IconRole::File => nf::cod::COD_FILE,
        IconRole::Error => nf::cod::COD_ERROR,
        IconRole::CaretRight => nf::oct::OCT_CHEVRON_RIGHT,
        IconRole::CaretDown => nf::oct::OCT_CHEVRON_DOWN,
        _ => "?",
    };
    IconSpec::new(glyph).with_size(13.0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiFileTreeRowModel {
    path: PathBuf,
    label: String,
    kind: FileSidebarEntryKind,
    depth: usize,
    expanded: bool,
    selected: bool,
    error: bool,
}

fn gui_file_tree_rows(
    root: &Path,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
) -> Vec<GuiFileTreeRowModel> {
    let mut rows = Vec::new();
    push_gui_file_tree_rows(root, 0, expanded_paths, selected_path, &mut rows);
    rows
}

fn push_gui_file_tree_rows(
    path: &Path,
    depth: usize,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
    rows: &mut Vec<GuiFileTreeRowModel>,
) {
    let expanded = expanded_paths.contains(path);
    rows.push(GuiFileTreeRowModel {
        path: path.to_path_buf(),
        label: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
        kind: FileSidebarEntryKind::Directory,
        depth,
        expanded,
        selected: selected_path == Some(path),
        error: false,
    });

    if !expanded || depth >= GUI_FILE_TREE_MAX_DEPTH {
        return;
    }

    let Ok(entries) = list_file_sidebar_entries(path) else {
        rows.push(GuiFileTreeRowModel {
            path: path.to_path_buf(),
            label: "cannot read directory".to_string(),
            kind: FileSidebarEntryKind::File,
            depth: depth + 1,
            expanded: false,
            selected: false,
            error: true,
        });
        return;
    };

    for entry in entries {
        if entry.kind == FileSidebarEntryKind::Parent {
            continue;
        }
        match entry.kind {
            FileSidebarEntryKind::Directory => {
                push_gui_file_tree_rows(&entry.path, depth + 1, expanded_paths, selected_path, rows)
            }
            FileSidebarEntryKind::File => rows.push(GuiFileTreeRowModel {
                selected: selected_path == Some(entry.path.as_path()),
                path: entry.path,
                label: entry.label,
                kind: FileSidebarEntryKind::File,
                depth: depth + 1,
                expanded: false,
                error: false,
            }),
            FileSidebarEntryKind::Parent => {}
        }
    }
}

fn gui_file_tree_text_size(settings: EditorSettings) -> u32 {
    gui_ui_text_size(settings)
}

fn gui_file_tree_icon_size(settings: EditorSettings) -> u32 {
    gui_ui_icon_text_size(settings)
}

fn gui_file_tree_view<'a>(
    root: &Path,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let rows = gui_file_tree_rows(root, expanded_paths, selected_path)
        .into_iter()
        .map(|row| gui_file_tree_row(row, settings, palette))
        .collect::<Vec<_>>();

    scrollable(
        column(rows)
            .spacing(GUI_FILE_TREE_ROW_SPACING)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn gui_file_tree_row<'a>(
    row_model: GuiFileTreeRowModel,
    settings: EditorSettings,
    palette: iced::theme::Palette,
) -> Element<'a, Message> {
    let is_dir = row_model.kind != FileSidebarEntryKind::File;
    let path = row_model.path.clone();
    let caret: Element<'a, Message> = if is_dir {
        let glyph = if row_model.expanded {
            nf::oct::OCT_CHEVRON_DOWN
        } else {
            nf::oct::OCT_CHEVRON_RIGHT
        };
        button(
            text(glyph)
                .font(gui_icon_font())
                .size(gui_file_tree_icon_size(settings)),
        )
        .padding(0)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(22.0))
        .style(move |_theme: &Theme, status| gui_file_tree_button_style(palette, false, status))
        .on_press(Message::BrowserLocalTreeToggle(path.clone()))
        .into()
    } else {
        container(text(""))
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(22.0))
            .into()
    };

    let icon = match row_model.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory if row_model.expanded => {
            nf::cod::COD_FOLDER_OPENED
        }
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => nf::cod::COD_FOLDER,
        FileSidebarEntryKind::File => nf::cod::COD_FILE,
    };
    let label_color = gui_file_tree_row_text_color(palette, row_model.selected, row_model.error);
    let content = row![
        text(icon)
            .font(gui_icon_font())
            .size(gui_file_tree_icon_size(settings))
            .color(label_color),
        text(row_model.label)
            .size(gui_file_tree_text_size(settings))
            .color(label_color)
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    let select_path = row_model.path.clone();
    let activate_path = row_model.path.clone();
    let row_content = container(content)
        .padding([1, 3])
        .width(Length::Fill)
        .style(move |_theme| gui_file_tree_row_style(palette, row_model.selected));
    let select_button: Element<'a, Message> = if row_model.error {
        row_content.into()
    } else {
        mouse_area(row_content)
            .on_press(Message::BrowserLocalTreeSelected(select_path, is_dir))
            .on_double_click(Message::BrowserLocalTreeActivated(activate_path, is_dir))
            .interaction(mouse::Interaction::Pointer)
            .into()
    };

    row![
        container(text("")).width(Length::Fixed(row_model.depth as f32 * GUI_FILE_TREE_INDENT)),
        caret,
        select_button,
    ]
    .spacing(1)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn gui_file_tree_row_text_color(
    palette: iced::theme::Palette,
    selected: bool,
    error: bool,
) -> Color {
    if error {
        Color::from_rgb(0.55, 0.55, 0.55)
    } else if selected {
        palette.background
    } else {
        palette.text
    }
}

fn gui_file_tree_row_style(palette: iced::theme::Palette, selected: bool) -> container::Style {
    container::Style {
        text_color: Some(if selected {
            palette.background
        } else {
            palette.text
        }),
        background: selected.then_some(Background::Color(palette.primary)),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

fn gui_file_tree_button_style(
    palette: iced::theme::Palette,
    selected: bool,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let active_background = if selected {
        Some(Background::Color(palette.primary))
    } else {
        None
    };
    let mut style = iced::widget::button::Style {
        background: active_background,
        text_color: if selected {
            palette.background
        } else {
            palette.text
        },
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..iced::widget::button::Style::default()
    };

    if matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    ) && !selected
    {
        style.background = Some(Background::Color(Color {
            a: 0.16,
            ..palette.primary
        }));
    }

    style
}

fn gui_directory_tree(root: PathBuf) -> DirectoryTree {
    DirectoryTree::new(root)
        .with_filter(DirectoryFilter::FilesAndFolders)
        .with_icon_theme(Arc::new(GuiTreeIconTheme))
}

fn resolve_browser_child_path(base_directory: &Path, raw_name: &str) -> Result<PathBuf, String> {
    let name = raw_name.trim();
    if name.is_empty() {
        return Err("name required".to_string());
    }
    let relative = Path::new(name);
    if relative.is_absolute() {
        return Err("absolute paths are not allowed here".to_string());
    }
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("parent paths are not allowed here".to_string());
    }
    let Some(file_name) = relative.file_name().and_then(|name| name.to_str()) else {
        return Err("valid UTF-8 name required".to_string());
    };
    if file_name == "." || file_name.is_empty() {
        return Err("valid name required".to_string());
    }

    Ok(base_directory.join(relative))
}

fn delete_browser_path(path: &Path, kind: FileSidebarEntryKind) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "refusing to delete symlink",
        ));
    }
    match kind {
        FileSidebarEntryKind::File if metadata.is_file() => fs::remove_file(path),
        FileSidebarEntryKind::Directory if metadata.is_dir() => fs::remove_dir_all(path),
        FileSidebarEntryKind::Parent => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "cannot delete parent shortcut",
        )),
        FileSidebarEntryKind::File => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected path is not a file",
        )),
        FileSidebarEntryKind::Directory => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected path is not a directory",
        )),
    }
}

fn color(red: u8, green: u8, blue: u8) -> Color {
    Color::from_rgb8(red, green, blue)
}

fn document_cursor_from_editor(cursor: text_editor::Cursor) -> DocumentCursor {
    let position = match cursor.selection {
        Some(selection)
            if (selection.line, selection.column)
                < (cursor.position.line, cursor.position.column) =>
        {
            selection
        }
        _ => cursor.position,
    };

    DocumentCursor {
        row: position.line,
        column: position.column,
    }
}

fn editor_cursor_from_document(cursor: DocumentCursor) -> text_editor::Cursor {
    text_editor::Cursor {
        position: text_editor::Position {
            line: cursor.row,
            column: cursor.column,
        },
        selection: None,
    }
}

fn search_result_status(result: SearchRepeatResult, backwards: bool) -> String {
    match result {
        SearchRepeatResult::NoPreviousSearch => "search query required".to_string(),
        SearchRepeatResult::Found { query } if backwards => format!("found previous: {query}"),
        SearchRepeatResult::Found { query } => format!("found next: {query}"),
        SearchRepeatResult::NoMatch { query } => format!("no match: {query}"),
    }
}

fn gui_repeat_search(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    query: &str,
    backwards: bool,
    case_sensitive: bool,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }
    if case_sensitive {
        return if backwards {
            repeat_search_previous(document, cursor, query)
        } else {
            repeat_search_next(document, cursor, query)
        };
    }

    let found = if backwards {
        gui_find_previous_case_insensitive(document, query, *cursor)
    } else {
        let start = gui_next_search_start(document, *cursor);
        gui_find_next_case_insensitive(document, query, start).or_else(|| {
            gui_find_next_case_insensitive(document, query, DocumentCursor { row: 0, column: 0 })
        })
    };

    if let Some(found) = found {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}

fn gui_next_search_start(document: &TextDocument, cursor: DocumentCursor) -> DocumentCursor {
    let columns = document.buffer.line_char_count(cursor.row).unwrap_or(0);
    if cursor.column < columns {
        return DocumentCursor {
            row: cursor.row,
            column: cursor.column + 1,
        };
    }
    if cursor.row + 1 < document.buffer.line_count() {
        return DocumentCursor {
            row: cursor.row + 1,
            column: 0,
        };
    }
    DocumentCursor { row: 0, column: 0 }
}

fn gui_find_next_case_insensitive(
    document: &TextDocument,
    query: &str,
    start: DocumentCursor,
) -> Option<DocumentCursor> {
    if query.is_empty() || start.row >= document.buffer.line_count() {
        return None;
    }
    for row in start.row..document.buffer.line_count() {
        let column = if row == start.row { start.column } else { 0 };
        if let Some(cursor) = gui_find_in_line_case_insensitive(
            document.buffer.line(row).unwrap_or_default(),
            query,
            column,
            row,
        ) {
            return Some(cursor);
        }
    }
    None
}

fn gui_find_previous_case_insensitive(
    document: &TextDocument,
    query: &str,
    start: DocumentCursor,
) -> Option<DocumentCursor> {
    if query.is_empty() || document.buffer.line_count() == 0 {
        return None;
    }
    let start_row = start
        .row
        .min(document.buffer.line_count().saturating_sub(1));
    for row in (0..=start_row).rev() {
        let line = document.buffer.line(row).unwrap_or_default();
        let max_column = line.chars().count();
        let before_column = if row == start_row {
            start.column.min(max_column)
        } else {
            max_column
        };
        if let Some(cursor) =
            gui_find_last_in_line_case_insensitive(line, query, before_column, row)
        {
            return Some(cursor);
        }
    }
    None
}

fn gui_find_in_line_case_insensitive(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
) -> Option<DocumentCursor> {
    let start_byte = line
        .char_indices()
        .nth(start_column)
        .map(|(index, _)| index)
        .unwrap_or(line.len());
    let lower_tail = line[start_byte..].to_lowercase();
    let lower_query = query.to_lowercase();
    let match_byte = lower_tail.find(&lower_query)?;
    let column = line[..start_byte].chars().count() + lower_tail[..match_byte].chars().count();
    Some(DocumentCursor { row, column })
}

fn gui_find_last_in_line_case_insensitive(
    line: &str,
    query: &str,
    before_column: usize,
    row: usize,
) -> Option<DocumentCursor> {
    let before_byte = line
        .char_indices()
        .nth(before_column)
        .map(|(index, _)| index)
        .unwrap_or(line.len());
    let lower_prefix = line[..before_byte].to_lowercase();
    let lower_query = query.to_lowercase();
    let match_byte = lower_prefix.rfind(&lower_query)?;
    let column = lower_prefix[..match_byte].chars().count();
    Some(DocumentCursor { row, column })
}

fn go_to_line_status(result: GoToLineResult) -> String {
    match result {
        GoToLineResult::Empty => "Line number is empty".to_string(),
        GoToLineResult::Invalid => "Line number is invalid".to_string(),
        GoToLineResult::OutOfRange { line_number } => {
            format!("Line out of range: {line_number}")
        }
        GoToLineResult::Moved { line_number } => format!("Line {line_number}"),
    }
}

fn gui_menu_groups() -> [GuiMenuGroup; 7] {
    [
        GuiMenuGroup::File,
        GuiMenuGroup::Edit,
        GuiMenuGroup::View,
        GuiMenuGroup::Go,
        GuiMenuGroup::Notes,
        GuiMenuGroup::Tile,
        GuiMenuGroup::Help,
    ]
}

fn gui_menu_group_label(group: GuiMenuGroup) -> &'static str {
    match group {
        GuiMenuGroup::File => "File",
        GuiMenuGroup::Edit => "Edit",
        GuiMenuGroup::View => "View",
        GuiMenuGroup::Go => "Nav",
        GuiMenuGroup::Notes => "Notes",
        GuiMenuGroup::Tile => "Tile",
        GuiMenuGroup::Help => "Help",
    }
}

fn gui_menu_items(group: GuiMenuGroup) -> Vec<GuiMenuItem> {
    match group {
        GuiMenuGroup::File => vec![
            GuiMenuItem {
                label: LABEL_NEW_TILE,
                command: GuiMenuCommand::NewTile,
            },
            GuiMenuItem {
                label: LABEL_OPEN,
                command: GuiMenuCommand::Open,
            },
            GuiMenuItem {
                label: LABEL_OPEN_PATH,
                command: GuiMenuCommand::OpenPath,
            },
            GuiMenuItem {
                label: LABEL_SAVE,
                command: GuiMenuCommand::Save,
            },
            GuiMenuItem {
                label: LABEL_SAVE_AS,
                command: GuiMenuCommand::SaveAs,
            },
            GuiMenuItem {
                label: LABEL_SAVE_AS_PATH,
                command: GuiMenuCommand::SaveAsPath,
            },
            GuiMenuItem {
                label: LABEL_CLOSE_TILE,
                command: GuiMenuCommand::ClosePane,
            },
            GuiMenuItem {
                label: LABEL_QUIT,
                command: GuiMenuCommand::Quit,
            },
        ],
        GuiMenuGroup::Edit => vec![
            GuiMenuItem {
                label: LABEL_UNDO,
                command: GuiMenuCommand::Undo,
            },
            GuiMenuItem {
                label: LABEL_REDO,
                command: GuiMenuCommand::Redo,
            },
            GuiMenuItem {
                label: LABEL_COPY,
                command: GuiMenuCommand::Copy,
            },
            GuiMenuItem {
                label: LABEL_CUT,
                command: GuiMenuCommand::Cut,
            },
            GuiMenuItem {
                label: LABEL_PASTE,
                command: GuiMenuCommand::Paste,
            },
            GuiMenuItem {
                label: LABEL_SELECT_ALL,
                command: GuiMenuCommand::SelectAll,
            },
            GuiMenuItem {
                label: LABEL_FIND_NEXT,
                command: GuiMenuCommand::FindNext,
            },
            GuiMenuItem {
                label: LABEL_FIND_PREVIOUS,
                command: GuiMenuCommand::FindPrevious,
            },
        ],
        GuiMenuGroup::View => vec![
            GuiMenuItem {
                label: LABEL_FILES,
                command: GuiMenuCommand::ToggleBrowser,
            },
            GuiMenuItem {
                label: LABEL_THEME,
                command: GuiMenuCommand::CycleTheme,
            },
            GuiMenuItem {
                label: LABEL_SYNTAX_THEME,
                command: GuiMenuCommand::CycleSyntaxTheme,
            },
            GuiMenuItem {
                label: LABEL_READER_MODE,
                command: GuiMenuCommand::ToggleReaderMode,
            },
        ],
        GuiMenuGroup::Go => vec![
            GuiMenuItem {
                label: LABEL_GO_TO_LINE,
                command: GuiMenuCommand::GoToLine,
            },
            GuiMenuItem {
                label: LABEL_DOCUMENT_START,
                command: GuiMenuCommand::GoDocumentStart,
            },
            GuiMenuItem {
                label: LABEL_DOCUMENT_END,
                command: GuiMenuCommand::GoDocumentEnd,
            },
        ],
        GuiMenuGroup::Notes => vec![
            GuiMenuItem {
                label: LABEL_OPEN_NOTE,
                command: GuiMenuCommand::OpenManagedNote,
            },
            GuiMenuItem {
                label: LABEL_LIST_NOTES,
                command: GuiMenuCommand::ListManagedNotes,
            },
        ],
        GuiMenuGroup::Tile => vec![
            GuiMenuItem {
                label: LABEL_MINIMIZE,
                command: GuiMenuCommand::ToggleMinimize,
            },
            GuiMenuItem {
                label: LABEL_MAXIMIZE,
                command: GuiMenuCommand::ToggleMaximize,
            },
            GuiMenuItem {
                label: LABEL_EQUALIZE_TILES,
                command: GuiMenuCommand::EqualizeTiles,
            },
            GuiMenuItem {
                label: LABEL_MOVE_LEFT,
                command: GuiMenuCommand::MoveLeft,
            },
            GuiMenuItem {
                label: LABEL_MOVE_RIGHT,
                command: GuiMenuCommand::MoveRight,
            },
            GuiMenuItem {
                label: LABEL_MOVE_UP,
                command: GuiMenuCommand::MoveUp,
            },
            GuiMenuItem {
                label: LABEL_MOVE_DOWN,
                command: GuiMenuCommand::MoveDown,
            },
        ],
        GuiMenuGroup::Help => vec![GuiMenuItem {
            label: LABEL_OPEN_HELP,
            command: GuiMenuCommand::OpenHelp,
        }],
    }
}

fn gui_path_prompt_label(prompt: GuiPathPrompt) -> &'static str {
    match prompt {
        GuiPathPrompt::Open => "Open path",
        GuiPathPrompt::SaveAs => "Save as path",
        GuiPathPrompt::ManagedNote => "Note title",
        GuiPathPrompt::BrowserCreateFile => "File name",
        GuiPathPrompt::BrowserCreateDirectory => "Folder name",
    }
}

#[cfg(test)]
fn gui_icon_label(icon: &str, label: &str) -> String {
    format!("{icon} {label}")
}

fn gui_icon_only_label(icon: &str) -> String {
    icon.to_string()
}

fn gui_file_name_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn gui_paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn gui_sidebar_path_label(path: &Path) -> String {
    let label = path.display().to_string();
    if label.chars().count() <= GUI_PANEL_PATH_MAX_CHARS {
        return label;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if !file_name.is_empty() {
        let suffix = format!(".../{file_name}");
        if suffix.chars().count() <= GUI_PANEL_PATH_MAX_CHARS {
            return suffix;
        }
    }

    let keep = GUI_PANEL_PATH_MAX_CHARS.saturating_sub(3);
    format!(
        "...{}",
        label
            .chars()
            .rev()
            .take(keep)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    )
}

fn gui_header_layout_mode(viewport_width: f32) -> GuiHeaderLayoutMode {
    if viewport_width < GUI_HEADER_SPLIT_WIDTH {
        GuiHeaderLayoutMode::SplitActions
    } else {
        GuiHeaderLayoutMode::SingleRow
    }
}

fn gui_search_layout_mode(viewport_width: f32) -> GuiSearchLayoutMode {
    if viewport_width < GUI_SEARCH_SPLIT_WIDTH {
        GuiSearchLayoutMode::SplitRows
    } else {
        GuiSearchLayoutMode::SingleRow
    }
}

fn gui_tile_title_label(path: &Path, active: bool, save_status: &str) -> String {
    let file_name = gui_file_name_label(path);
    let label = if save_status == "saved" {
        file_name
    } else {
        format!("{file_name} | {save_status}")
    };

    if active {
        format!("active | {label}")
    } else {
        label
    }
}

fn gui_tile_title_controls_attached(_active: bool) -> bool {
    true
}

fn gui_tile_border_color(palette: iced::theme::Palette, active: bool) -> Color {
    if active {
        palette.primary
    } else {
        Color {
            a: 0.55,
            ..palette.primary
        }
    }
}

fn gui_tile_body_style(palette: iced::theme::Palette, _active: bool) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: Some(palette.background.into()),
        border: iced::Border {
            color: gui_tile_border_color(palette, _active),
            width: 1.0,
            radius: GUI_TILE_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

fn gui_tile_title_style(palette: iced::theme::Palette, active: bool) -> container::Style {
    let background = if active {
        Color {
            a: 0.18,
            ..palette.primary
        }
    } else {
        Color {
            a: 0.08,
            ..palette.primary
        }
    };

    container::Style {
        text_color: Some(if active {
            palette.primary
        } else {
            palette.text
        }),
        background: Some(background.into()),
        border: iced::Border {
            color: gui_tile_border_color(palette, active),
            width: 1.0,
            radius: GUI_TILE_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

fn gui_pane_grid_style(palette: iced::theme::Palette) -> pane_grid::Style {
    pane_grid::Style {
        hovered_region: pane_grid::Highlight {
            background: Background::Color(Color::TRANSPARENT),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: GUI_TILE_RADIUS.into(),
            },
        },
        picked_split: pane_grid::Line {
            color: palette.primary,
            width: 1.0,
        },
        hovered_split: pane_grid::Line {
            color: palette.primary,
            width: 1.0,
        },
    }
}

fn gui_menu_group_chrome_label(group: GuiMenuGroup) -> String {
    gui_menu_group_label(group).to_string()
}

#[cfg(test)]
fn gui_menu_group_index(group: GuiMenuGroup) -> usize {
    gui_menu_groups()
        .iter()
        .position(|candidate| *candidate == group)
        .expect("menu group belongs to static menu list")
}

#[cfg(test)]
fn gui_menu_dropdown_labels(group: GuiMenuGroup) -> Vec<&'static str> {
    gui_menu_items(group)
        .into_iter()
        .map(|item| item.label)
        .collect()
}

#[cfg(test)]
fn gui_menu_uses_iced_aw_menu_tree() -> bool {
    true
}

#[cfg(test)]
fn gui_menu_submenu_policy() -> &'static str {
    "Keep current root command groups flat until a group gains enough depth to justify nested hover submenus."
}

fn gui_menu_panel_style(palette: iced::theme::Palette) -> iced_aw::style::menu_bar::Style {
    iced_aw::style::menu_bar::Style {
        bar_background: palette.background.into(),
        bar_border: iced::Border {
            color: palette.background,
            width: 0.0,
            radius: 0.0.into(),
        },
        bar_shadow: Shadow::default(),
        menu_background: palette.background.into(),
        menu_border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: GUI_MENU_DROPDOWN_RADIUS.into(),
        },
        menu_shadow: Shadow {
            color: Color {
                a: 0.35,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        path: palette.primary.into(),
        path_border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: GUI_MENU_ITEM_RADIUS.into(),
        },
    }
}

fn gui_menu_item_button_style(
    palette: iced::theme::Palette,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut style = iced::widget::button::Style {
        background: Some(palette.background.into()),
        text_color: palette.text,
        border: iced::Border {
            color: palette.primary,
            width: 0.0,
            radius: GUI_MENU_ITEM_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        iced::widget::button::Status::Active => style,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            style.background = Some(palette.primary.into());
            style.text_color = palette.background;
            style.border.width = 1.0;
            style
        }
        iced::widget::button::Status::Disabled => {
            style.text_color = Color {
                a: 0.45,
                ..palette.text
            };
            style
        }
    }
}

fn gui_menu_root_style(palette: iced::theme::Palette) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: None,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

fn gui_chrome_button_style(
    palette: iced::theme::Palette,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut style = iced::widget::button::Style {
        background: Some(palette.primary.into()),
        text_color: palette.background,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    if matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    ) {
        style.background = Some(
            Color {
                a: 0.86,
                ..palette.primary
            }
            .into(),
        );
    }

    if matches!(status, iced::widget::button::Status::Disabled) {
        style.background = Some(
            Color {
                a: 0.35,
                ..palette.primary
            }
            .into(),
        );
        style.text_color = Color {
            a: 0.55,
            ..palette.background
        };
    }

    style
}

fn gui_scrollbar_track_style(palette: iced::theme::Palette, enabled: bool) -> container::Style {
    container::Style {
        background: Some(
            Color {
                a: if enabled { 0.12 } else { 0.04 },
                ..palette.primary
            }
            .into(),
        ),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: (GUI_EDITOR_SCROLLBAR_WIDTH / 2.0).into(),
        },
        ..container::Style::default()
    }
}

fn gui_scrollbar_thumb_style(palette: iced::theme::Palette, enabled: bool) -> container::Style {
    container::Style {
        background: Some(
            Color {
                a: if enabled { 0.78 } else { 0.24 },
                ..palette.primary
            }
            .into(),
        ),
        border: iced::Border {
            color: Color {
                a: if enabled { 0.82 } else { 0.28 },
                ..palette.primary
            },
            width: 1.0,
            radius: (GUI_EDITOR_SCROLLBAR_WIDTH / 2.0).into(),
        },
        ..container::Style::default()
    }
}

fn gui_text_input_style(
    palette: iced::theme::Palette,
    _status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: palette.background.into(),
        border: iced::Border {
            color: Color {
                a: 0.65,
                ..palette.primary
            },
            width: 1.0,
            radius: 2.0.into(),
        },
        icon: palette.primary,
        placeholder: Color {
            a: 0.58,
            ..palette.text
        },
        value: palette.text,
        selection: Color {
            a: 0.7,
            ..palette.primary
        },
    }
}

fn gui_checkbox_style(
    palette: iced::theme::Palette,
    status: iced::widget::checkbox::Status,
) -> iced::widget::checkbox::Style {
    let checked = match status {
        iced::widget::checkbox::Status::Active { is_checked }
        | iced::widget::checkbox::Status::Hovered { is_checked }
        | iced::widget::checkbox::Status::Disabled { is_checked } => is_checked,
    };

    let background = if checked {
        palette.primary.into()
    } else {
        palette.background.into()
    };

    iced::widget::checkbox::Style {
        background,
        icon_color: palette.background,
        border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: 2.0.into(),
        },
        text_color: Some(palette.text),
    }
}

fn gui_native_editor_style(
    palette: iced::theme::Palette,
    _status: text_editor::Status,
    search_highlight_active: bool,
) -> text_editor::Style {
    let selection = if search_highlight_active {
        Color {
            a: 0.95,
            ..palette.primary
        }
    } else {
        Color {
            a: 0.7,
            ..palette.primary
        }
    };

    text_editor::Style {
        background: palette.background.into(),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: Color {
            a: 0.42,
            ..palette.text
        },
        value: palette.text,
        selection,
    }
}

fn gui_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    tooltip_text: impl Into<String>,
    position: iced::widget::tooltip::Position,
    settings: EditorSettings,
) -> Element<'a, Message> {
    iced::widget::tooltip(
        content,
        container(text(tooltip_text.into()).size(gui_ui_tooltip_text_size(settings)))
            .padding(8)
            .style(|_theme| container::Style {
                text_color: Some(color(226, 240, 255)),
                background: Some(color(3, 7, 18).into()),
                border: iced::Border {
                    color: color(102, 229, 255),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..container::Style::default()
            }),
        position,
    )
    .gap(6)
    .snap_within_viewport(true)
    .style(|_theme| container::Style {
        text_color: Some(color(226, 240, 255)),
        background: Some(color(3, 7, 18).into()),
        border: iced::Border {
            color: color(102, 229, 255),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn gui_tooltip_button<'a>(
    label: impl Into<String>,
    message: Message,
    tooltip_text: impl Into<String>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    gui_tooltip(
        button(text(label.into()).size(gui_ui_text_size(settings)))
            .padding(GUI_CHROME_PADDING)
            .style(move |_theme, status| gui_chrome_button_style(palette, status))
            .on_press(message),
        tooltip_text,
        iced::widget::tooltip::Position::Bottom,
        settings,
    )
}

fn gui_icon_font() -> Font {
    Font::with_name(GUI_ICON_FONT_NAME)
}

fn gui_centered_icon<'a>(icon: &'a str, settings: EditorSettings) -> Element<'a, Message> {
    container(
        text(gui_icon_only_label(icon))
            .font(gui_icon_font())
            .size(gui_ui_icon_text_size(settings))
            .line_height(GUI_ICON_LINE_HEIGHT)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Shrink)
            .height(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

fn gui_icon_tooltip_button<'a>(
    icon: &'static str,
    message: Message,
    tooltip_text: impl Into<String>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    gui_tooltip(
        button(gui_centered_icon(icon, settings))
            .width(Length::Fixed(GUI_ICON_BUTTON_SIDE))
            .height(Length::Fixed(GUI_ICON_BUTTON_SIDE))
            .padding(0)
            .style(move |_theme, status| gui_chrome_button_style(palette, status))
            .on_press(message),
        tooltip_text,
        iced::widget::tooltip::Position::Bottom,
        settings,
    )
}

fn gui_header_action_row<'a>(state: &'a KfnotepadGui) -> Element<'a, Message> {
    row![
        gui_icon_tooltip_button(
            ICON_NEW_TILE,
            Message::NewTileRequested,
            "Create a new tile (Ctrl-N)",
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FILES,
            Message::ToggleBrowser,
            if state.left_panel.visible {
                format!("Hide {} panel (Ctrl-B)", state.left_panel.title())
            } else {
                format!("Show {} panel (Ctrl-B)", state.left_panel.title())
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_THEME,
            Message::CycleTheme,
            format!("Cycle theme: {}", state.settings.theme_id.label()),
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_SYNTAX_THEME,
            Message::CycleSyntaxTheme,
            format!(
                "Cycle syntax theme: {} (Ctrl-Shift-T)",
                state.settings.syntax_theme_id.label()
            ),
            state.settings,
        ),
        gui_icon_tooltip_button(
            if state.settings.gui_reader_mode_enabled {
                ICON_READER_MODE_PAUSE
            } else {
                ICON_READER_MODE_PLAY
            },
            Message::MenuCommand(GuiMenuCommand::ToggleReaderMode),
            if state.settings.gui_reader_mode_enabled {
                format!(
                    "Stop reader mode (Ctrl-R), {} lines/min",
                    state.settings.gui_reader_lines_per_minute
                )
            } else {
                format!(
                    "Start reader mode (Ctrl-R), {} lines/min",
                    state.settings.gui_reader_lines_per_minute
                )
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_SAVE,
            Message::SaveRequested,
            "Save active tile (Ctrl-S)",
            state.settings,
        ),
    ]
    .spacing(GUI_HEADER_ACTION_SPACING)
    .align_y(Alignment::Center)
    .into()
}

fn gui_minimized_tray<'a>(state: &'a KfnotepadGui) -> Option<Element<'a, Message>> {
    let items = state.minimized_tray_items();
    if items.is_empty() {
        return None;
    }

    let palette = gui_theme_palette(state.settings.theme_id);
    let mut tray = row![text("Minimized").size(gui_ui_small_text_size(state.settings))]
        .spacing(6)
        .align_y(Alignment::Center);

    for item in items {
        let tooltip = format!("Restore {}", item.tooltip);
        tray = tray.push(gui_tooltip(
            button(text(item.title).size(gui_ui_small_text_size(state.settings)))
                .padding(GUI_CHROME_PADDING)
                .on_press(Message::RestoreMinimizedTile(item.tile_id)),
            tooltip,
            iced::widget::tooltip::Position::Bottom,
            state.settings,
        ));
    }

    Some(
        container(tray)
            .width(Length::Fill)
            .padding([2, 4])
            .style(move |_theme| container::Style {
                text_color: Some(palette.text),
                background: Some(palette.background.into()),
                border: iced::Border {
                    color: Color {
                        a: 0.55,
                        ..palette.primary
                    },
                    width: 1.0,
                    radius: GUI_TILE_RADIUS.into(),
                },
                ..container::Style::default()
            })
            .into(),
    )
}

fn gui_editor_read_only_view(
    pane: pane_grid::Pane,
    editor_surface: &GuiEditorSurfaceModel<'_>,
    settings: EditorSettings,
    search_highlight_active: bool,
    ime_preedit: Option<GuiImePreedit>,
) -> Element<'static, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let line_number_width = editor_surface
        .line_numbers
        .as_ref()
        .map(|line_numbers| line_numbers.width);
    let source_lines = editor_surface.viewport_slice.lines.clone();
    let first_line = editor_surface.viewport_slice.first_line;
    let line_count = editor_surface.viewport_slice.line_count;
    let wrapping = editor_surface.wrapping;
    let editor_font = editor_surface.editor_font;
    let editor_size = editor_surface.editor_size;

    responsive(move |surface_size| {
        let gutter_width = line_number_width.unwrap_or_default()
            + line_number_width
                .map(|_| GUI_LINE_NUMBER_SEPARATOR_WIDTH)
                .unwrap_or_default();
        let body_width = (surface_size.width - gutter_width).max(1.0);
        let character_width = gui_editor_replacement_character_width(settings);
        let row_height = gui_editor_replacement_row_height(settings);
        let body_columns = (body_width / character_width).floor().max(1.0) as usize;
        let visible_row_budget = gui_editor_visible_row_budget(surface_size.height, row_height);
        let scrollbar_model = gui_editor_scrollbar_model(
            line_count,
            first_line,
            visible_row_budget,
            surface_size.height,
        );
        let visual_rows =
            gui_editor_read_only_visual_rows(&source_lines, first_line, wrapping, body_columns)
                .into_iter()
                .take(visible_row_budget);
        let mut editor_rows = iced::widget::Column::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Shrink);
        let mut ime_request = None;

        for (rendered_row, visual_row) in visual_rows.enumerate() {
            let viewport_row = visual_row.viewport_row;
            let source_column_start = visual_row.source_column_start;
            if let Some(cursor_column) = visual_row.line.cursor_column {
                ime_request = Some(GuiImeInputMethodRequest {
                    visual_row: rendered_row,
                    cursor_column,
                    gutter_width,
                    character_width,
                    row_height,
                    preedit: ime_preedit.as_ref().map(|preedit| input_method::Preedit {
                        content: preedit.content.clone(),
                        selection: preedit.selection.clone(),
                        text_size: Some(Pixels(editor_size as f32)),
                    }),
                });
            }
            let line_for_render =
                gui_editor_viewport_line_with_ime_preedit(visual_row.line, ime_preedit.as_ref());
            let visual_row_text = line_for_render.text.clone();
            let line_spans =
                gui_editor_read_only_line_spans(&line_for_render, palette, search_highlight_active);
            let line_text = rich_text(line_spans)
                .font(editor_font)
                .size(editor_size)
                .line_height(GUI_EDITOR_LINE_HEIGHT)
                .wrapping(Wrapping::None)
                .width(Length::Fill)
                .color(palette.text);
            let line_body = mouse_area(
                container(line_text)
                    .width(Length::Fill)
                    .height(Length::Fixed(row_height))
                    .style(move |_theme| container::Style {
                        text_color: Some(palette.text),
                        background: Some(palette.background.into()),
                        ..container::Style::default()
                    }),
            )
            .on_move(move |point| {
                Message::ReplacementEditorPointerMoved(
                    pane,
                    gui_editor_replacement_mouse_point_from_visual_row_point(
                        point,
                        viewport_row,
                        source_column_start,
                        &visual_row_text,
                        settings,
                    ),
                )
            })
            .on_press(Message::ReplacementEditorPointerPressed(pane))
            .on_release(Message::ReplacementEditorPointerReleased(pane));

            let mut line_row = iced::widget::Row::new()
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fixed(row_height));
            if let Some(line_number_width) = line_number_width {
                let line_number_label = if visual_row.show_line_number {
                    line_for_render.number.to_string()
                } else {
                    String::new()
                };
                let line_number_text = text(line_number_label)
                    .font(editor_font)
                    .size(editor_size)
                    .line_height(GUI_EDITOR_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .color(palette.primary);
                let gutter_separator = container(text(""))
                    .width(Length::Fixed(GUI_LINE_NUMBER_SEPARATOR_WIDTH))
                    .height(Length::Fixed(row_height))
                    .style(move |_theme| container::Style {
                        background: Some(
                            Color {
                                a: 0.55,
                                ..palette.primary
                            }
                            .into(),
                        ),
                        ..container::Style::default()
                    });
                line_row = line_row
                    .push(
                        container(line_number_text)
                            .width(Length::Fixed(line_number_width))
                            .height(Length::Fixed(row_height))
                            .padding([0, 2])
                            .style(move |_theme| container::Style {
                                text_color: Some(palette.primary),
                                background: Some(palette.background.into()),
                                ..container::Style::default()
                            }),
                    )
                    .push(gutter_separator);
            }
            editor_rows = editor_rows.push(line_row.push(line_body));
        }

        let body_source_lines = source_lines.clone();
        let editor_body: Element<'static, Message> = mouse_area(
            container(editor_rows)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme| container::Style {
                    text_color: Some(palette.text),
                    background: Some(palette.background.into()),
                    ..container::Style::default()
                }),
        )
        .on_move(move |point| {
            let pointer = gui_editor_replacement_mouse_point_from_body_point(
                point,
                &body_source_lines,
                first_line,
                wrapping,
                GuiEditorBodyHitTest {
                    columns: body_columns,
                    visible_rows: visible_row_budget,
                    text_origin_x: gutter_width,
                },
                settings,
            );
            let edge = gui_editor_drag_edge_from_body_point(
                pane,
                point,
                surface_size.height,
                pointer.column,
                settings,
            );
            Message::ReplacementEditorBodyPointerMoved(pane, pointer, edge)
        })
        .on_press(Message::ReplacementEditorPointerPressed(pane))
        .on_release(Message::ReplacementEditorPointerReleased(pane))
        .on_scroll(move |delta| {
            Message::ReplacementEditorWheelScrolled(
                pane,
                gui_editor_replacement_scroll_delta_lines(delta, settings),
            )
        })
        .into();

        let editor_with_scrollbar: Element<'static, Message> = row![
            editor_body,
            gui_editor_scrollbar_view(pane, scrollbar_model, palette, settings)
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        Element::new(GuiInputMethodArea::new(editor_with_scrollbar, ime_request))
    })
    .into()
}

fn gui_editor_scrollbar_view(
    pane: pane_grid::Pane,
    model: GuiEditorScrollbarModel,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> Element<'static, Message> {
    let top_height = model.thumb_top.max(0.0);
    let thumb_height = model.thumb_height.max(1.0);
    let bottom_height = (model.track_height - model.thumb_top - model.thumb_height).max(0.0);

    let track_above = gui_scrollbar_track_segment(top_height, palette, model.visible);
    let thumb = container(text(""))
        .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
        .height(Length::Fixed(thumb_height))
        .style(move |_theme| gui_scrollbar_thumb_style(palette, model.visible));
    let track_below = gui_scrollbar_track_segment(bottom_height, palette, model.visible);

    mouse_area(
        column![track_above, thumb, track_below]
            .spacing(0)
            .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
            .height(Length::Fill),
    )
    .on_move(move |point| Message::ReplacementEditorScrollbarMoved(pane, point.y, model))
    .on_press(Message::ReplacementEditorScrollbarPressed(pane))
    .on_release(Message::ReplacementEditorScrollbarReleased(pane))
    .on_scroll(move |delta| {
        Message::ReplacementEditorWheelScrolled(
            pane,
            gui_editor_replacement_scroll_delta_lines(delta, settings),
        )
    })
    .into()
}

fn gui_scrollbar_track_segment(
    height: f32,
    palette: iced::theme::Palette,
    enabled: bool,
) -> Element<'static, Message> {
    if !enabled || height < 1.0 {
        return container(text(""))
            .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
            .height(Length::Fixed(height.max(0.0)))
            .style(move |_theme| gui_scrollbar_track_style(palette, false))
            .into();
    }

    container(text(""))
        .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
        .height(Length::Fixed(height))
        .style(move |_theme| gui_scrollbar_track_style(palette, enabled))
        .into()
}

fn gui_editor_read_only_line_spans(
    line: &GuiEditorViewportLine,
    palette: iced::theme::Palette,
    search_highlight_active: bool,
) -> Vec<iced::widget::text::Span<'static, Message, Font>> {
    gui_editor_read_only_line_segments(line)
        .into_iter()
        .map(|segment| {
            let selected = segment.selected;
            let segment_color = if selected {
                palette.background
            } else {
                segment.syntax_color.unwrap_or(palette.text)
            };
            let mut text_span = span(segment.text).color(segment_color);
            if selected {
                text_span = text_span
                    .background(gui_replacement_editor_overlay_color(
                        palette,
                        search_highlight_active,
                    ))
                    .padding(1);
            }
            text_span
        })
        .collect()
}

fn gui_replacement_editor_overlay_color(
    palette: iced::theme::Palette,
    search_highlight_active: bool,
) -> Color {
    Color {
        a: if search_highlight_active { 0.95 } else { 0.78 },
        ..palette.primary
    }
}

fn gui_find_controls<'a>(state: &'a KfnotepadGui, field_width: f32) -> Element<'a, Message> {
    let palette = gui_theme_palette(state.settings.theme_id);
    let input = text_input("Find", &state.search_query)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchNext)
        .size(gui_ui_text_size(state.settings))
        .style(move |_theme, status| gui_text_input_style(palette, status))
        .width(Length::Fixed(field_width));
    let mut input_stack = column![input].spacing(2).width(Length::Fixed(field_width));
    if state.search_history_open
        && state.search_query.is_empty()
        && !state.search_history.is_empty()
    {
        let mut history = column![].spacing(1).width(Length::Fill);
        for query in state.search_history.iter().take(GUI_FIND_HISTORY_LIMIT) {
            let query_for_message = query.clone();
            history = history.push(
                button(
                    text(query)
                        .size(gui_ui_small_text_size(state.settings))
                        .width(Length::Fill),
                )
                .width(Length::Fill)
                .padding([2, 5])
                .style(move |_theme, status| gui_menu_item_button_style(palette, status))
                .on_press(Message::SearchHistorySelected(query_for_message)),
            );
        }
        input_stack = input_stack.push(
            container(history)
                .width(Length::Fill)
                .padding(3)
                .style(move |_theme| gui_find_history_style(palette)),
        );
    }

    row![
        input_stack,
        gui_icon_tooltip_button(
            ICON_CASE_SENSITIVE,
            Message::SearchCaseSensitiveChanged(!state.settings.search_case_sensitive),
            if state.settings.search_case_sensitive {
                "Case-sensitive search on"
            } else {
                "Case-sensitive search off"
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FIND_PREVIOUS,
            Message::SearchPrevious,
            LABEL_FIND_PREVIOUS,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FIND_NEXT,
            Message::SearchNext,
            LABEL_FIND_NEXT,
            state.settings
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn gui_find_history_style(palette: iced::theme::Palette) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: Some(palette.background.into()),
        border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..container::Style::default()
    }
}

fn gui_navigation_controls<'a>(state: &'a KfnotepadGui, field_width: f32) -> Element<'a, Message> {
    let palette = gui_theme_palette(state.settings.theme_id);
    row![
        text_input("Line", &state.go_to_line_query)
            .on_input(Message::GoToLineQueryChanged)
            .on_submit(Message::GoToLineRequested)
            .size(gui_ui_text_size(state.settings))
            .style(move |_theme, status| gui_text_input_style(palette, status))
            .width(Length::Fixed(field_width)),
        gui_icon_tooltip_button(
            ICON_GO_TO_LINE,
            Message::GoToLineRequested,
            LABEL_GO_TO_LINE,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_DOCUMENT_START,
            Message::GoDocumentStart,
            LABEL_DOCUMENT_START,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_DOCUMENT_END,
            Message::GoDocumentEnd,
            LABEL_DOCUMENT_END,
            state.settings,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn gui_search_controls<'a>(state: &'a KfnotepadGui, viewport_width: f32) -> Element<'a, Message> {
    match gui_search_layout_mode(viewport_width) {
        GuiSearchLayoutMode::SingleRow => row![
            gui_find_controls(state, 220.0),
            gui_navigation_controls(state, 90.0),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        GuiSearchLayoutMode::SplitRows => column![
            gui_find_controls(state, 190.0),
            gui_navigation_controls(state, 82.0),
        ]
        .spacing(6)
        .into(),
    }
}

fn gui_status_bar<'a>(status_message: &'a str, settings: EditorSettings) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    text(status_message)
        .size(gui_ui_small_text_size(settings))
        .color(palette.text)
        .into()
}

fn gui_menu_command_item<'a>(
    item: GuiMenuItem,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> iced_aw::menu::Item<'a, Message, Theme, iced::Renderer> {
    iced_aw::menu::Item::new(
        button(text(item.label).size(gui_ui_text_size(settings)))
            .width(Length::Fill)
            .padding(GUI_MENU_ITEM_PADDING)
            .style(move |_theme, status| gui_menu_item_button_style(palette, status))
            .on_press(Message::MenuCommand(item.command)),
    )
    .close_on_click(true)
}

fn gui_menu_dropdown<'a>(
    group: GuiMenuGroup,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> Menu<'a, Message, Theme, iced::Renderer> {
    let items = gui_menu_items(group)
        .into_iter()
        .map(|item| gui_menu_command_item(item, palette, settings))
        .collect();

    Menu::new(items)
        .width(Length::Fixed(GUI_MENU_DROPDOWN_WIDTH))
        .spacing(3)
        .padding(7)
        .close_on_item_click(true)
        .close_on_background_click(true)
}

fn gui_menu_root_item<'a>(
    group: GuiMenuGroup,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> iced_aw::menu::Item<'a, Message, Theme, iced::Renderer> {
    // iced_aw's menu tree expects roots to be Item::with_menu and commands to be
    // regular widgets inside Item::new. Keep the current shallow command groups
    // flat until nested hover submenus have a clear UX win.
    iced_aw::menu::Item::with_menu(
        container(
            text(gui_menu_group_chrome_label(group))
                .size(gui_ui_text_size(settings))
                .color(palette.text),
        )
        .height(Length::Fixed(GUI_MENU_ROOT_HEIGHT))
        .padding([
            GUI_MENU_ROOT_VERTICAL_PADDING,
            GUI_MENU_ROOT_HORIZONTAL_PADDING,
        ])
        .style(move |_theme| gui_menu_root_style(palette)),
        gui_menu_dropdown(group, palette, settings),
    )
}

fn gui_menu_bar<'a>(settings: EditorSettings) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let roots = gui_menu_groups()
        .into_iter()
        .map(|group| gui_menu_root_item(group, palette, settings))
        .collect();

    MenuBar::new(roots)
        .spacing(GUI_MENU_BAR_SPACING)
        .padding(0)
        .draw_path(menu::DrawPath::Backdrop)
        .close_on_item_click_global(true)
        .close_on_background_click_global(true)
        .style(move |_theme, _status| gui_menu_panel_style(palette))
        .into()
}

#[cfg(test)]
fn gui_action_descriptors() -> Vec<GuiActionDescriptor> {
    vec![
        GuiActionDescriptor {
            label: LABEL_NEW_TILE,
            shortcut: Some("Ctrl-N"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_OPEN,
            shortcut: Some("Ctrl-O"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_SAVE,
            shortcut: Some("Ctrl-S"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_SAVE_AS,
            shortcut: Some("Ctrl-Shift-S"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_CLOSE_TILE,
            shortcut: Some("Ctrl-F4"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_QUIT,
            shortcut: Some("Ctrl-Q"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_FIND_NEXT,
            shortcut: Some("F3"),
            menu_group: Some(GuiMenuGroup::Edit),
        },
        GuiActionDescriptor {
            label: LABEL_FIND_PREVIOUS,
            shortcut: Some("Shift-F3"),
            menu_group: Some(GuiMenuGroup::Edit),
        },
        GuiActionDescriptor {
            label: LABEL_FILES,
            shortcut: Some("Ctrl-B"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_THEME,
            shortcut: Some("Ctrl-T"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_SYNTAX_THEME,
            shortcut: Some("Ctrl-Shift-T"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_READER_MODE,
            shortcut: Some("Ctrl-R"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_GO_TO_LINE,
            shortcut: Some("Ctrl-G"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: LABEL_DOCUMENT_START,
            shortcut: Some("Ctrl-Home"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: LABEL_DOCUMENT_END,
            shortcut: Some("Ctrl-End"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: "Scroll viewport up",
            shortcut: Some("Ctrl-PageUp"),
            menu_group: None,
        },
        GuiActionDescriptor {
            label: "Scroll viewport down",
            shortcut: Some("Ctrl-PageDown"),
            menu_group: None,
        },
        GuiActionDescriptor {
            label: LABEL_OPEN_NOTE,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Notes),
        },
        GuiActionDescriptor {
            label: LABEL_LIST_NOTES,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Notes),
        },
        GuiActionDescriptor {
            label: LABEL_MINIMIZE,
            shortcut: Some("Ctrl-M"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MAXIMIZE,
            shortcut: Some("Ctrl-Shift-M"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_LEFT,
            shortcut: Some("Ctrl-Shift-Left"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_RIGHT,
            shortcut: Some("Ctrl-Shift-Right"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_UP,
            shortcut: Some("Ctrl-Shift-Up"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_DOWN,
            shortcut: Some("Ctrl-Shift-Down"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_OPEN_HELP,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Help),
        },
    ]
}

#[cfg(test)]
fn gui_focus_order_descriptors(browser_visible: bool, tile_minimized: bool) -> Vec<GuiFocusStep> {
    let mut steps = gui_menu_groups()
        .into_iter()
        .map(|group| GuiFocusStep {
            area: "menu",
            label: gui_menu_group_label(group),
            keyboard: None,
        })
        .collect::<Vec<_>>();

    steps.extend([
        GuiFocusStep {
            area: "header",
            label: LABEL_NEW_TILE,
            keyboard: Some("Ctrl-N"),
        },
        GuiFocusStep {
            area: "header",
            label: if browser_visible {
                "Hide Files"
            } else {
                "Show Files"
            },
            keyboard: Some("Ctrl-B"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_THEME,
            keyboard: Some("Ctrl-T"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_SYNTAX_THEME,
            keyboard: Some("Ctrl-Shift-T"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_READER_MODE,
            keyboard: Some("Ctrl-R"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_SAVE,
            keyboard: Some("Ctrl-S"),
        },
    ]);

    if browser_visible {
        steps.extend([
            GuiFocusStep {
                area: "file browser",
                label: "Parent directory",
                keyboard: None,
            },
            GuiFocusStep {
                area: "file browser",
                label: LABEL_REFRESH,
                keyboard: None,
            },
            GuiFocusStep {
                area: "file browser",
                label: LABEL_CREATE_FILE,
                keyboard: None,
            },
        ]);
        steps.push(GuiFocusStep {
            area: "file browser",
            label: "File browser entries",
            keyboard: None,
        });
    }

    steps.extend([
        GuiFocusStep {
            area: "search",
            label: "Find field",
            keyboard: Some("Ctrl-F"),
        },
        GuiFocusStep {
            area: "search",
            label: "Case-sensitive search",
            keyboard: None,
        },
        GuiFocusStep {
            area: "search",
            label: LABEL_FIND_PREVIOUS,
            keyboard: Some("Shift-F3"),
        },
        GuiFocusStep {
            area: "search",
            label: LABEL_FIND_NEXT,
            keyboard: Some("F3"),
        },
        GuiFocusStep {
            area: "navigation",
            label: "Line field",
            keyboard: Some("Ctrl-G"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_GO,
            keyboard: Some("Enter in line field"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_DOCUMENT_START,
            keyboard: Some("Ctrl-Home"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_DOCUMENT_END,
            keyboard: Some("Ctrl-End"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_LEFT,
            keyboard: Some("Ctrl-Shift-Left"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_RIGHT,
            keyboard: Some("Ctrl-Shift-Right"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_UP,
            keyboard: Some("Ctrl-Shift-Up"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_DOWN,
            keyboard: Some("Ctrl-Shift-Down"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: if tile_minimized {
                LABEL_RESTORE
            } else {
                LABEL_MINIMIZE
            },
            keyboard: Some("Ctrl-M"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MAXIMIZE,
            keyboard: Some("Ctrl-Shift-M"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_CLOSE_TILE,
            keyboard: Some("Ctrl-F4"),
        },
    ]);

    if !tile_minimized {
        steps.push(GuiFocusStep {
            area: "editor",
            label: "Active editor",
            keyboard: None,
        });
    }

    steps
}

fn view(state: &KfnotepadGui) -> Element<'_, Message> {
    responsive(move |viewport_size| view_sized(state, viewport_size)).into()
}

fn view_sized(state: &KfnotepadGui, viewport_size: Size) -> Element<'_, Message> {
    let palette = gui_theme_palette(state.settings.theme_id);
    let menu_bar = gui_menu_bar(state.settings);

    let header: Element<'_, Message> = match gui_header_layout_mode(viewport_size.width) {
        GuiHeaderLayoutMode::SingleRow => row![menu_bar, gui_header_action_row(state),]
            .spacing(GUI_HEADER_GROUP_SPACING)
            .align_y(Alignment::Center)
            .into(),
        GuiHeaderLayoutMode::SplitActions => column![menu_bar, gui_header_action_row(state),]
            .spacing(GUI_HEADER_SPLIT_SPACING)
            .into(),
    };

    let path_prompt = state.path_prompt.map(|prompt| {
        container(
            row![
                text(gui_path_prompt_label(prompt)).size(gui_ui_text_size(state.settings)),
                text_input("Path", &state.path_prompt_value)
                    .on_input(Message::PathPromptChanged)
                    .on_submit(Message::SubmitPathPrompt)
                    .size(gui_ui_text_size(state.settings))
                    .style(move |_theme, status| gui_text_input_style(palette, status))
                    .width(Length::Fill),
                gui_tooltip_button(
                    LABEL_GO,
                    Message::SubmitPathPrompt,
                    "Apply path prompt",
                    state.settings,
                ),
                gui_tooltip_button(
                    LABEL_CANCEL,
                    Message::CancelPathPrompt,
                    "Cancel path prompt",
                    state.settings,
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(8)
    });

    let notes_panel = state.notes_panel.as_ref().map(|notes| {
        let mut items = row![text("Notes").size(gui_ui_text_size(state.settings))]
            .spacing(8)
            .align_y(Alignment::Center);
        if notes.is_empty() {
            items = items.push(text("No managed notes").size(gui_ui_text_size(state.settings)));
        } else {
            for (index, note) in notes.iter().enumerate() {
                let delete_tooltip =
                    if state.pending_managed_note_delete.as_deref() == Some(note.path.as_path()) {
                        format!("Confirm delete {}", note.file_name)
                    } else {
                        format!("Delete {}", note.file_name)
                    };
                items = items.push(
                    row![
                        button(text(&note.file_name).size(gui_ui_text_size(state.settings)))
                            .on_press(Message::ManagedNoteClicked(index)),
                        gui_tooltip(
                            button(gui_centered_icon(ICON_DELETE, state.settings))
                                .width(Length::Fixed(GUI_ICON_BUTTON_SIDE))
                                .height(Length::Fixed(GUI_ICON_BUTTON_SIDE))
                                .padding(0)
                                .on_press(Message::ManagedNoteDeleteClicked(index)),
                            delete_tooltip,
                            iced::widget::tooltip::Position::Bottom,
                            state.settings,
                        ),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center),
                );
            }
        }
        items = items.push(gui_tooltip_button(
            LABEL_CANCEL,
            Message::CancelPathPrompt,
            "Close notes panel",
            state.settings,
        ));
        container(items).width(Length::Fill).padding(8)
    });

    let sidebar = if state.left_panel.visible {
        let panel_tabs = row![
            gui_icon_tooltip_button(
                ICON_FILES,
                Message::SelectLeftPanelMode(GuiLeftPanelMode::Files),
                "Show Files panel",
                state.settings,
            ),
            gui_icon_tooltip_button(
                ICON_WORKSPACES,
                Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
                "Show Workspaces panel",
                state.settings,
            ),
            gui_icon_tooltip_button(
                ICON_PREFERENCES,
                Message::SelectLeftPanelMode(GuiLeftPanelMode::Preferences),
                "Show Preferences panel",
                state.settings,
            ),
        ]
        .spacing(GUI_PANEL_CONTROL_SPACING);
        let panel_content = match state.left_panel.mode {
            GuiLeftPanelMode::Files => match &state.browser {
                Some(browser) => {
                    let current_dir = &browser.sidebar.current_dir;
                    let selected_path = state.browser_selected_path.as_deref().or_else(|| {
                        browser
                            .sidebar
                            .selected_entry()
                            .map(|entry| entry.path.as_path())
                    });
                    let tree_view = gui_file_tree_view(
                        current_dir,
                        &state.browser_expanded_paths,
                        selected_path,
                        state.settings,
                    );
                    column![
                        panel_tabs,
                        row![
                            text(LABEL_FILES).size(gui_ui_heading_text_size(state.settings)),
                            text(format!("{:.0}px", state.browser_width))
                                .size(gui_ui_small_text_size(state.settings)),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        gui_tooltip(
                            text(gui_sidebar_path_label(current_dir))
                                .size(gui_ui_small_text_size(state.settings)),
                            current_dir.display().to_string(),
                            iced::widget::tooltip::Position::Bottom,
                            state.settings,
                        ),
                        slider(
                            GUI_BROWSER_WIDTH_MIN..=GUI_BROWSER_WIDTH_MAX,
                            state.browser_width,
                            Message::BrowserWidthChanged,
                        )
                        .step(10.0),
                        row![
                            gui_icon_tooltip_button(
                                ICON_PARENT_DIR,
                                Message::BrowserParentRequested,
                                "Go to parent directory",
                                state.settings,
                            ),
                            gui_icon_tooltip_button(
                                ICON_REFRESH,
                                Message::BrowserRefreshRequested,
                                "Refresh file browser",
                                state.settings,
                            ),
                            gui_icon_tooltip_button(
                                ICON_CREATE_FILE,
                                Message::BrowserCreateFileRequested,
                                format!("{LABEL_CREATE_FILE} in selected folder"),
                                state.settings,
                            ),
                            gui_icon_tooltip_button(
                                ICON_CREATE_DIRECTORY,
                                Message::BrowserCreateDirectoryRequested,
                                "Create folder in selected folder",
                                state.settings,
                            ),
                            gui_icon_tooltip_button(
                                ICON_DELETE,
                                Message::BrowserDeleteSelectedRequested,
                                "Delete selected file or folder",
                                state.settings,
                            ),
                        ]
                        .spacing(GUI_PANEL_CONTROL_SPACING)
                        .align_y(Alignment::Center),
                        container(tree_view).padding(iced::Padding {
                            top: GUI_PANEL_TREE_TOP_PADDING,
                            right: 0.0,
                            bottom: 0.0,
                            left: 0.0,
                        }),
                    ]
                    .spacing(GUI_PANEL_SECTION_SPACING)
                }
                None => column![
                    panel_tabs,
                    text(LABEL_FILES).size(gui_ui_heading_text_size(state.settings)),
                    text("file browser unavailable").size(gui_ui_text_size(state.settings)),
                ],
            },
            GuiLeftPanelMode::Workspaces => {
                let mut projects = column![
                    panel_tabs,
                    text(LABEL_WORKSPACES).size(gui_ui_heading_text_size(state.settings)),
                    row![
                        gui_icon_tooltip_button(
                            ICON_SAVE,
                            Message::SaveCurrentWorkspaceProject,
                            "Save current workspace project",
                            state.settings,
                        ),
                        gui_icon_tooltip_button(
                            ICON_REFRESH,
                            Message::RefreshWorkspaceProjects,
                            "Refresh saved workspace projects",
                            state.settings,
                        ),
                    ]
                    .spacing(6),
                    text_input("Project name", &state.workspace_project_name)
                        .on_input(Message::WorkspaceProjectNameChanged)
                        .on_submit(Message::SaveNamedWorkspaceProject)
                        .size(gui_ui_text_size(state.settings))
                        .style(move |_theme, status| gui_text_input_style(palette, status)),
                    gui_icon_tooltip_button(
                        ICON_SAVE,
                        Message::SaveNamedWorkspaceProject,
                        "Save workspace project with this name",
                        state.settings,
                    ),
                    checkbox(state.settings.gui_restore_last_workspace)
                        .label("Restore last workspace")
                        .text_size(gui_ui_text_size(state.settings))
                        .spacing(8)
                        .style(move |_theme, status| gui_checkbox_style(palette, status))
                        .on_toggle(Message::RestoreLastWorkspaceChanged),
                ]
                .spacing(5);
                if state.workspace_projects.is_empty() {
                    projects = projects.push(
                        text("No saved workspace projects").size(gui_ui_text_size(state.settings)),
                    );
                } else {
                    for (index, entry) in state.workspace_projects.iter().enumerate() {
                        projects = projects.push(
                            row![
                                button(
                                    column![
                                        text(&entry.project.name)
                                            .size(gui_ui_text_size(state.settings)),
                                        text(format!("{} files", entry.project.files.len()))
                                            .size(gui_ui_small_text_size(state.settings)),
                                    ]
                                    .spacing(2),
                                )
                                .width(Length::Fill)
                                .padding(6)
                                .on_press(Message::WorkspaceProjectClicked(index)),
                                gui_icon_tooltip_button(
                                    ICON_NEW_WINDOW,
                                    Message::WorkspaceProjectNewWindowClicked(index),
                                    "Open workspace project in a new window",
                                    state.settings,
                                ),
                                gui_icon_tooltip_button(
                                    ICON_DELETE,
                                    Message::WorkspaceProjectDeleteClicked(index),
                                    "Delete workspace project",
                                    state.settings,
                                ),
                            ]
                            .spacing(4)
                            .align_y(Alignment::Center),
                        );
                    }
                }
                projects
            }
            GuiLeftPanelMode::Preferences => column![
                panel_tabs,
                text(LABEL_PREFERENCES).size(gui_ui_heading_text_size(state.settings)),
                gui_tooltip_button(
                    format!("Font: {}", state.settings.gui_font_family.display_label()),
                    Message::CycleGuiFontFamily,
                    "Cycle editor font family",
                    state.settings,
                ),
                gui_tooltip_button(
                    format!("Syntax: {}", state.settings.syntax_theme_id.label()),
                    Message::CycleSyntaxTheme,
                    "Cycle syntax highlighting theme",
                    state.settings,
                ),
                row![
                    text(format!("Editor size: {}", state.settings.gui_font_size))
                        .size(gui_ui_text_size(state.settings)),
                    slider(
                        MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE,
                        state.settings.gui_font_size,
                        Message::GuiFontSizeChanged,
                    )
                    .step(1u16),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text(format!("UI size: {}", state.settings.gui_ui_font_size))
                        .size(gui_ui_text_size(state.settings)),
                    slider(
                        MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE,
                        state.settings.gui_ui_font_size,
                        Message::GuiUiFontSizeChanged,
                    )
                    .step(1u16),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                checkbox(state.settings.show_line_numbers)
                    .label("Line numbers")
                    .text_size(gui_ui_text_size(state.settings))
                    .spacing(8)
                    .style(move |_theme, status| gui_checkbox_style(palette, status))
                    .on_toggle(Message::ShowLineNumbersChanged),
                checkbox(state.settings.wrap_lines)
                    .label("Wrap text")
                    .text_size(gui_ui_text_size(state.settings))
                    .spacing(8)
                    .style(move |_theme, status| gui_checkbox_style(palette, status))
                    .on_toggle(Message::WrapLinesChanged),
                checkbox(state.settings.search_case_sensitive)
                    .label("Case-sensitive search")
                    .text_size(gui_ui_text_size(state.settings))
                    .spacing(8)
                    .style(move |_theme, status| gui_checkbox_style(palette, status))
                    .on_toggle(Message::SearchCaseSensitiveChanged),
                checkbox(state.settings.gui_reader_mode_enabled)
                    .label("Reader mode")
                    .text_size(gui_ui_text_size(state.settings))
                    .spacing(8)
                    .style(move |_theme, status| gui_checkbox_style(palette, status))
                    .on_toggle(Message::ReaderModeChanged),
                row![
                    text(format!(
                        "Reader speed: {} lpm",
                        state.settings.gui_reader_lines_per_minute
                    ))
                    .size(gui_ui_text_size(state.settings)),
                    slider(
                        MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE,
                        state.settings.gui_reader_lines_per_minute,
                        Message::ReaderSpeedChanged,
                    )
                    .step(5u16),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                checkbox(state.settings.gui_restore_last_workspace)
                    .label("Restore last workspace")
                    .text_size(gui_ui_text_size(state.settings))
                    .spacing(8)
                    .style(move |_theme, status| gui_checkbox_style(palette, status))
                    .on_toggle(Message::RestoreLastWorkspaceChanged),
            ]
            .spacing(5),
        };

        Some(
            container(scrollable(panel_content))
                .width(Length::Fixed(gui_left_panel_width(
                    state.left_panel.visible,
                    state.browser_width,
                )))
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: GUI_PANEL_PADDING_VERTICAL,
                    right: GUI_PANEL_PADDING_RIGHT,
                    bottom: GUI_PANEL_PADDING_VERTICAL,
                    left: GUI_PANEL_PADDING_LEFT,
                }),
        )
    } else {
        None
    };

    let search_controls = gui_search_controls(state, viewport_size.width);

    let panes = pane_grid(&state.panes, |pane, pane_state, is_maximized| {
        let Some(tile) = state.workspace.tile(pane_state.tile_id) else {
            return pane_grid::Content::new(text("Missing tile"));
        };
        let mut save_status = match tile.save_status() {
            GuiTileSaveStatus::Saved => "saved".to_string(),
            GuiTileSaveStatus::Modified => "modified".to_string(),
            GuiTileSaveStatus::SaveFailed { message } => format!("save failed: {message}"),
        };
        if state.is_external_edit_locked(pane_state.tile_id) {
            save_status = if save_status == "saved" {
                "locked".to_string()
            } else {
                format!("{save_status} | locked")
            };
        }
        let title =
            gui_tile_title_label(&tile.document.path, pane == state.active_pane, &save_status);
        let title_tooltip = tile.document.path.display().to_string();
        let tile_palette = gui_theme_palette(state.settings.theme_id);
        let active_tile = pane == state.active_pane;
        let minimize_icon = if tile.minimized {
            ICON_RESTORE
        } else {
            ICON_MINIMIZE
        };
        let maximize_icon = if is_maximized {
            ICON_RESTORE
        } else {
            ICON_MAXIMIZE
        };
        let mut title_bar = pane_grid::TitleBar::new(gui_tooltip(
            text(title).size(gui_ui_text_size(state.settings)),
            title_tooltip,
            iced::widget::tooltip::Position::Bottom,
            state.settings,
        ))
        .padding(GUI_TILE_TITLE_PADDING)
        .style(move |_theme| gui_tile_title_style(tile_palette, active_tile));
        if gui_tile_title_controls_attached(pane == state.active_pane) {
            let mut controls = row![
                gui_tooltip(
                    button(gui_centered_icon(ICON_MOVE_LEFT, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::MovePane(pane, pane_grid::Direction::Left)),
                    LABEL_MOVE_LEFT,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(ICON_MOVE_RIGHT, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::MovePane(pane, pane_grid::Direction::Right)),
                    LABEL_MOVE_RIGHT,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(ICON_MOVE_UP, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::MovePane(pane, pane_grid::Direction::Up)),
                    LABEL_MOVE_UP,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(ICON_MOVE_DOWN, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::MovePane(pane, pane_grid::Direction::Down)),
                    LABEL_MOVE_DOWN,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(minimize_icon, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::ToggleMinimizePane(pane)),
                    if tile.minimized {
                        LABEL_RESTORE
                    } else {
                        LABEL_MINIMIZE
                    },
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(maximize_icon, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::ToggleMaximizePane(pane)),
                    if is_maximized {
                        LABEL_RESTORE
                    } else {
                        LABEL_MAXIMIZE
                    },
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                gui_tooltip(
                    button(gui_centered_icon(ICON_CLOSE, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::ClosePane(pane)),
                    LABEL_CLOSE_TILE,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
            ]
            .spacing(GUI_TILE_CONTROL_SPACING);
            if state.is_external_edit_locked(pane_state.tile_id) {
                controls = controls.push(gui_tooltip(
                    button(gui_centered_icon(ICON_UNLOCK, state.settings))
                        .width(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .height(Length::Fixed(GUI_TILE_CONTROL_BUTTON_SIDE))
                        .padding(0)
                        .style(move |_theme, status| gui_chrome_button_style(tile_palette, status))
                        .on_press(Message::UnlockExternalEdit(pane_state.tile_id)),
                    LABEL_UNLOCK_EXTERNAL_EDIT,
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ));
            }
            title_bar = title_bar.controls(pane_grid::Controls::new(controls));
        }
        let body = if tile.minimized {
            column![text("Minimized").size(gui_ui_text_size(state.settings))]
                .padding(GUI_TILE_BODY_PADDING)
                .height(Length::Fill)
        } else {
            let editor_surface = gui_editor_surface_model(
                state.settings,
                &tile.document,
                &pane_state.editor,
                &state.syntax_highlighter,
                state.syntax_caches.get(&pane_state.tile_id),
            );
            let search_highlight_active = state
                .search_highlight
                .as_ref()
                .is_some_and(|highlight| highlight.tile_id == pane_state.tile_id);
            let ime_preedit = state
                .replacement_ime_preedit
                .as_ref()
                .filter(|preedit| preedit.tile_id == pane_state.tile_id)
                .cloned();
            let editor_body: Element<'_, Message> = if GUI_USE_READ_ONLY_EDITOR_RENDERER {
                gui_editor_read_only_view(
                    pane,
                    &editor_surface,
                    state.settings,
                    search_highlight_active,
                    ime_preedit,
                )
            } else {
                let editor = text_editor(editor_surface.content)
                    .placeholder("Type here...")
                    .font(editor_surface.editor_font)
                    .size(editor_surface.editor_size)
                    .line_height(GUI_EDITOR_LINE_HEIGHT)
                    .wrapping(editor_surface.wrapping)
                    .highlight(
                        &editor_surface.syntax_token,
                        editor_surface.highlighter_theme,
                    )
                    .style(move |_theme, status| {
                        gui_native_editor_style(tile_palette, status, search_highlight_active)
                    })
                    .on_action(move |action| Message::Edit(pane, action))
                    .height(Length::Fill);
                let _line_numbers = editor_surface.line_numbers;
                editor.into()
            };

            column![editor_body]
                .padding(GUI_TILE_BODY_PADDING)
                .height(Length::Fill)
        };

        pane_grid::Content::new(body)
            .title_bar(title_bar)
            .style(move |_theme| gui_tile_body_style(tile_palette, active_tile))
    })
    .height(Length::Fill)
    .spacing(GUI_PANE_GRID_SPACING)
    .style(move |_theme| gui_pane_grid_style(gui_theme_palette(state.settings.theme_id)))
    .on_click(Message::PaneClicked)
    .on_resize(8, Message::PaneResized)
    .on_drag(Message::PaneDragged);

    let editor = container(
        column![
            search_controls,
            panes,
            gui_status_bar(&state.status_message, state.settings),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(GUI_EDITOR_PADDING);

    let minimized_tray = gui_minimized_tray(state);
    let mut body = column![].spacing(8);
    if let Some(path_prompt) = path_prompt {
        body = body.push(path_prompt);
    }
    if let Some(notes_panel) = notes_panel {
        body = body.push(notes_panel);
    }
    let content = if let Some(sidebar) = sidebar {
        row![sidebar, editor].height(Length::Fill)
    } else {
        row![editor].height(Length::Fill)
    };
    body = body.push(content);

    let mut root = column![header].spacing(8);
    if let Some(minimized_tray) = minimized_tray {
        root = root.push(minimized_tray);
    }
    root = root.push(body);
    container(root)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(GUI_ROOT_PADDING)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kfnotepad::DEFAULT_GUI_FONT_SIZE;
    use std::fs;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    fn numbered_lines(count: usize) -> String {
        (1..=count)
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn gui_test_syntax_cache_for_document(
        highlighter: &SyntaxHighlighter,
        document: &TextDocument,
        visible_rows: usize,
    ) -> GuiSyntaxCache {
        let (highlighted, state) =
            highlighter.highlight_lines_incremental(document, 0, visible_rows, None);
        GuiSyntaxCache {
            path: document.path.clone(),
            line_count: document.buffer.line_count().max(1),
            highlighted_until: highlighted.len(),
            lines: highlighted
                .into_iter()
                .map(|line| {
                    line.map(|segments| {
                        gui_syntax_segments_from_syntect(segments, EditorThemeId::Nocturne)
                    })
                })
                .collect(),
            state,
        }
    }

    #[test]
    fn gui_launch_loads_requested_file_into_editor_state() {
        let temp = TempArea::new("gui-open");
        let path = temp.path("note.txt");
        fs::write(&path, "alpha\nbeta\n").expect("write file");

        let state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![path.clone()],
        });

        assert_eq!(state.workspace.active_tile().document.path, path);
        assert_eq!(state.active_editor().text(), "alpha\nbeta\n");
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
    }

    #[test]
    fn gui_launches_multiple_requested_files_as_tiles_and_panes() {
        let temp = TempArea::new("gui-multi-open");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");

        let state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.active_editor().text(), "second\n");
        assert!(state.panes.iter().any(|(_pane, pane_state)| state
            .workspace
            .tile(pane_state.tile_id)
            .is_some_and(|tile| tile.document.path == first)));
    }

    #[test]
    fn gui_launch_uses_balanced_halloy_style_split_axes_for_three_files() {
        let temp = TempArea::new("gui-balanced-launch");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let third = temp.path("third.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::write(&third, "third\n").expect("write third");

        let state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone(), third.clone()],
        });

        let pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } = state.panes.layout()
        else {
            panic!("expected root split");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical);
        assert_eq!(*ratio, 0.5);
        assert_eq!(node_path(&state, a), Some(first));
        let pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } = &**b
        else {
            panic!("expected active-pane nested split");
        };
        assert_eq!(*axis, pane_grid::Axis::Horizontal);
        assert_eq!(*ratio, 0.5);
        assert_eq!(node_path(&state, a), Some(second));
        assert_eq!(node_path(&state, b), Some(third));
    }

    #[test]
    fn gui_file_open_splits_the_active_tall_pane_horizontally() {
        let temp = TempArea::new("gui-balanced-open");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let third = temp.path("third.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::write(&third, "third\n").expect("write third");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });

        state.open_path_in_new_pane(third.clone());

        let pane_grid::Node::Split { axis, b, .. } = state.panes.layout() else {
            panic!("expected root split");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical);
        let pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } = &**b
        else {
            panic!("expected new file split inside active pane");
        };
        assert_eq!(*axis, pane_grid::Axis::Horizontal);
        assert_eq!(*ratio, 0.5);
        assert_eq!(node_path(&state, a), Some(second));
        assert_eq!(node_path(&state, b), Some(third.clone()));
        assert_eq!(state.workspace.active_tile().document.path, third);
        assert!(state.status_message.starts_with("opened "));
    }

    #[test]
    fn gui_new_tile_splits_active_pane_without_creating_a_file() {
        let temp = TempArea::new("gui-new-tile");
        let existing_untitled = temp.path("untitled.txt");
        fs::write(&existing_untitled, "already here\n").expect("write existing untitled");
        let first = temp.path("first.txt");
        fs::write(&first, "first\n").expect("write first");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first.clone()],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::NewTileRequested);

        let expected = temp.path("untitled-2.txt");
        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, expected);
        assert_eq!(state.active_editor().text(), "");
        assert!(!temp.path("untitled-2.txt").exists());
        assert_eq!(
            fs::read_to_string(&existing_untitled).expect("read existing untitled"),
            "already here\n"
        );
        assert!(state.status_message.starts_with("new tile "));
    }

    #[test]
    fn gui_browser_width_updates_with_clamped_bounds() {
        let temp = TempArea::new("gui-browser-width");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::BrowserWidthChanged(190.0));
        assert_eq!(state.browser_width, 190.0);
        assert_eq!(state.status_message, "file browser width: 190px");

        let _ = update(&mut state, Message::BrowserWidthChanged(999.0));
        assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MAX);

        let _ = update(&mut state, Message::BrowserWidthChanged(1.0));
        assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MIN);
    }

    #[test]
    fn gui_editor_sync_marks_document_dirty() {
        let temp = TempArea::new("gui-dirty");
        let path = temp.path("note.txt");
        fs::write(&path, "alpha\n").expect("write file");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![path],
        });

        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("changed\n");
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor
            .move_to(DocumentCursor { row: 0, column: 4 });
        state.sync_active_editor_to_document();

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "changed\n"
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 4 }
        );
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Modified
        );
    }

    #[test]
    fn gui_save_active_tile_uses_existing_save_adapter() {
        let temp = TempArea::new("gui-save");
        let path = temp.path("note.txt");
        fs::write(&path, "alpha\n").expect("write file");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![path.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("saved through gui\n");
        state.sync_active_editor_to_document();

        state.save_active_tile();

        assert_eq!(
            fs::read_to_string(&path).expect("read saved file"),
            "saved through gui\n"
        );
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
        assert!(state.status_message.starts_with("saved "));
    }

    #[test]
    fn gui_save_only_writes_the_focused_tile() {
        let temp = TempArea::new("gui-save-focused");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });

        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("second changed\n");
        state.sync_active_editor_to_document();
        state.save_active_tile();

        assert_eq!(fs::read_to_string(&first).expect("read first"), "first\n");
        assert_eq!(
            fs::read_to_string(&second).expect("read second"),
            "second changed\n"
        );
    }

    #[test]
    fn gui_open_prompt_opens_relative_path_into_new_pane() {
        let temp = TempArea::new("gui-open-prompt");
        let initial = temp.path("initial.txt");
        let opened = temp.path("opened.txt");
        fs::write(&initial, "initial\n").expect("write initial");
        fs::write(&opened, "opened\n").expect("write opened");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![initial.clone()],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenPath));
        assert_eq!(state.path_prompt, Some(GuiPathPrompt::Open));
        let _ = update(
            &mut state,
            Message::PathPromptChanged("opened.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        assert_eq!(state.path_prompt, None);
        assert_eq!(state.path_prompt_value, "");
        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, opened);
        assert_eq!(state.active_editor().text(), "opened\n");
        assert!(state.status_message.starts_with("opened "));
    }

    #[test]
    fn gui_save_as_prompt_writes_to_relative_path_and_retargets_tile() {
        let temp = TempArea::new("gui-save-as-prompt");
        let original = temp.path("original.txt");
        let target = temp.path("saved-as.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("saved elsewhere\n");

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAsPath));
        assert_eq!(state.path_prompt, Some(GuiPathPrompt::SaveAs));
        let _ = update(
            &mut state,
            Message::PathPromptChanged("saved-as.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        assert_eq!(state.path_prompt, None);
        assert_eq!(state.workspace.active_tile().document.path, target);
        assert_eq!(
            fs::read_to_string(temp.path("saved-as.txt")).expect("read save-as"),
            "saved elsewhere\n"
        );
        assert_eq!(
            fs::read_to_string(original).expect("read original"),
            "original\n"
        );
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
        assert!(state.status_message.starts_with("saved as "));
    }

    #[test]
    fn gui_save_as_refuses_path_already_open_in_another_tile() {
        let temp = TempArea::new("gui-save-as-duplicate");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("second retarget attempt\n");

        let saved = state.save_active_tile_as(first.clone());

        assert!(!saved);
        assert_eq!(fs::read_to_string(&first).expect("read first"), "first\n");
        assert_eq!(
            fs::read_to_string(&second).expect("read second"),
            "second\n"
        );
        assert_eq!(
            state
                .workspace
                .tiles
                .iter()
                .filter(|tile| gui_paths_refer_to_same_file(&tile.document.path, &first))
                .count(),
            1
        );
        assert_eq!(
            state
                .workspace
                .tiles
                .iter()
                .filter(|tile| gui_paths_refer_to_same_file(&tile.document.path, &second))
                .count(),
            1
        );
        assert!(gui_paths_refer_to_same_file(
            &state.workspace.active_tile().document.path,
            &first
        ));
        assert!(state.status_message.starts_with("save as refused: "));
    }

    #[test]
    fn gui_save_as_failure_keeps_original_tile_path_and_prompt_open() {
        let temp = TempArea::new("gui-save-as-fail");
        let original = temp.path("original.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("not saved\n");

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAsPath));
        let _ = update(
            &mut state,
            Message::PathPromptChanged("missing-parent/out.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        assert_eq!(state.path_prompt, Some(GuiPathPrompt::SaveAs));
        assert_eq!(
            state.workspace.active_tile().document.path,
            original.clone()
        );
        assert!(!temp.path("missing-parent").exists());
        assert_eq!(
            fs::read_to_string(original).expect("read original"),
            "original\n"
        );
        assert!(state.status_message.starts_with("save as failed: "));
    }

    #[test]
    fn gui_save_refuses_external_modification_since_open() {
        let temp = TempArea::new("gui-save-conflict");
        let path = temp.path("note.txt");
        fs::write(&path, "original\n").expect("write original");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![path.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("gui edit\n");
        fs::write(&path, "external\n").expect("external edit");

        state.save_active_tile();

        assert_eq!(
            fs::read_to_string(&path).expect("read conflicting file"),
            "external\n"
        );
        assert!(matches!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::SaveFailed { .. }
        ));
        assert!(state
            .status_message
            .contains("file changed on disk since open or last save"));
    }

    #[test]
    fn gui_native_open_request_uses_dialog_without_showing_path_prompt() {
        let temp = TempArea::new("gui-native-open-request");
        let initial = temp.path("initial.txt");
        fs::write(&initial, "initial\n").expect("write initial");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![initial],
            },
            temp.root.clone(),
        );

        let _task = update(&mut state, Message::MenuCommand(GuiMenuCommand::Open));

        assert_eq!(state.path_prompt, None);
        assert_eq!(state.path_prompt_value, "");
        assert_eq!(state.status_message, "open dialog");
    }

    #[test]
    fn gui_native_open_dialog_selection_uses_existing_open_adapter() {
        let temp = TempArea::new("gui-native-open-selection");
        let initial = temp.path("initial.txt");
        let opened = temp.path("opened.txt");
        fs::write(&initial, "initial\n").expect("write initial");
        fs::write(&opened, "opened\n").expect("write opened");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![initial],
            },
            temp.root.clone(),
        );

        let _ = update(
            &mut state,
            Message::OpenDialogSelected(Some(opened.clone())),
        );

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, opened);
        assert_eq!(state.active_editor().text(), "opened\n");
        assert!(state.status_message.starts_with("opened "));
    }

    #[test]
    fn gui_native_open_dialog_cancel_is_noop() {
        let temp = TempArea::new("gui-native-open-cancel");
        let initial = temp.path("initial.txt");
        fs::write(&initial, "initial\n").expect("write initial");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![initial.clone()],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::OpenDialogSelected(None));

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, initial);
        assert_eq!(state.active_editor().text(), "initial\n");
        assert_eq!(state.status_message, "open canceled");
    }

    #[test]
    fn gui_native_save_as_request_uses_dialog_without_showing_path_prompt() {
        let temp = TempArea::new("gui-native-save-as-request");
        let original = temp.path("original.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original],
            },
            temp.root.clone(),
        );

        let _task = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAs));

        assert_eq!(state.path_prompt, None);
        assert_eq!(state.path_prompt_value, "");
        assert_eq!(state.status_message, "save as dialog");
    }

    #[test]
    fn gui_native_save_as_dialog_selection_uses_existing_save_adapter() {
        let temp = TempArea::new("gui-native-save-as-selection");
        let original = temp.path("original.txt");
        let target = temp.path("saved-as.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("saved through dialog\n");

        let _ = update(
            &mut state,
            Message::SaveAsDialogSelected(Some(target.clone())),
        );

        assert_eq!(state.workspace.active_tile().document.path, target);
        assert_eq!(
            fs::read_to_string(temp.path("saved-as.txt")).expect("read save-as"),
            "saved through dialog\n"
        );
        assert_eq!(
            fs::read_to_string(original).expect("read original"),
            "original\n"
        );
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
        assert!(state.status_message.starts_with("saved as "));
    }

    #[test]
    fn gui_native_save_as_dialog_cancel_is_noop() {
        let temp = TempArea::new("gui-native-save-as-cancel");
        let original = temp.path("original.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::SaveAsDialogSelected(None));

        assert_eq!(
            state.workspace.active_tile().document.path,
            original.clone()
        );
        assert_eq!(
            fs::read_to_string(original).expect("read original"),
            "original\n"
        );
        assert_eq!(state.status_message, "save as canceled");
    }

    #[test]
    fn gui_managed_note_prompt_creates_and_opens_note_in_new_pane() {
        let temp = TempArea::new("gui-managed-note-prompt");
        let first = temp.path("first.txt");
        let notes_dir = temp.path("notes");
        fs::write(&first, "first\n").expect("write first");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first],
            },
            temp.root.clone(),
        );
        state.notes_dir = Some(notes_dir.clone());

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::OpenManagedNote),
        );
        assert_eq!(state.path_prompt, Some(GuiPathPrompt::ManagedNote));
        let _ = update(
            &mut state,
            Message::PathPromptChanged("Daily Note".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        let expected = notes_dir.join("daily-note.md");
        assert_eq!(state.path_prompt, None);
        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, expected);
        assert_eq!(state.active_editor().text(), "");
        assert_eq!(
            fs::read_to_string(notes_dir.join("daily-note.md")).expect("read note"),
            ""
        );
        assert!(state.status_message.starts_with("opened note "));
    }

    #[test]
    fn gui_managed_notes_panel_lists_and_opens_existing_note() {
        let temp = TempArea::new("gui-managed-notes-panel");
        let first = temp.path("first.txt");
        let notes_dir = temp.path("notes");
        fs::write(&first, "first\n").expect("write first");
        fs::create_dir_all(&notes_dir).expect("create notes dir");
        fs::write(notes_dir.join("alpha.md"), "alpha\n").expect("write note");
        fs::write(notes_dir.join("zeta.md"), "zeta\n").expect("write note");
        fs::write(notes_dir.join("ignore.txt"), "ignored\n").expect("write ignored");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first],
            },
            temp.root.clone(),
        );
        state.notes_dir = Some(notes_dir.clone());

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
        );

        let notes = state.notes_panel.as_ref().expect("notes panel");
        assert_eq!(
            notes
                .iter()
                .map(|note| note.file_name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha.md", "zeta.md"]
        );
        assert_eq!(state.status_message, "managed notes: 2");

        let _ = update(&mut state, Message::ManagedNoteClicked(0));

        assert_eq!(state.notes_panel, None);
        assert_eq!(
            state.workspace.active_tile().document.path,
            notes_dir.join("alpha.md")
        );
        assert_eq!(state.active_editor().text(), "alpha\n");
    }

    #[test]
    fn gui_managed_note_delete_requires_confirmation_and_refreshes_list() {
        let temp = TempArea::new("gui-managed-note-delete");
        let first = temp.path("first.txt");
        let notes_dir = temp.path("notes");
        let alpha = notes_dir.join("alpha.md");
        fs::write(&first, "first\n").expect("write first");
        fs::create_dir_all(&notes_dir).expect("create notes dir");
        fs::write(&alpha, "alpha\n").expect("write note");
        fs::write(notes_dir.join("zeta.md"), "zeta\n").expect("write note");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first],
            },
            temp.root.clone(),
        );
        state.notes_dir = Some(notes_dir.clone());
        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
        );

        let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

        assert_eq!(state.pending_managed_note_delete, Some(alpha.clone()));
        assert!(alpha.exists());
        assert_eq!(
            state.status_message,
            "delete note alpha.md? click delete again"
        );

        let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

        assert_eq!(state.pending_managed_note_delete, None);
        assert!(!alpha.exists());
        assert_eq!(
            state
                .notes_panel
                .as_ref()
                .expect("notes panel")
                .iter()
                .map(|note| note.file_name.as_str())
                .collect::<Vec<_>>(),
            vec!["zeta.md"]
        );
        assert_eq!(state.status_message, "managed note deleted: alpha.md");
    }

    #[test]
    fn gui_managed_note_delete_refuses_open_note_tile() {
        let temp = TempArea::new("gui-managed-note-delete-open");
        let first = temp.path("first.txt");
        let notes_dir = temp.path("notes");
        let alpha = notes_dir.join("alpha.md");
        fs::write(&first, "first\n").expect("write first");
        fs::create_dir_all(&notes_dir).expect("create notes dir");
        fs::write(&alpha, "alpha\n").expect("write note");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first],
            },
            temp.root.clone(),
        );
        state.notes_dir = Some(notes_dir.clone());
        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
        );
        let _ = update(&mut state, Message::ManagedNoteClicked(0));
        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
        );

        let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

        assert_eq!(state.pending_managed_note_delete, None);
        assert!(alpha.exists());
        assert_eq!(
            state.status_message,
            "close note tile before deleting alpha.md"
        );
    }

    #[test]
    fn gui_external_file_change_refreshes_clean_tile_and_locks_editing() {
        let temp = TempArea::new("gui-external-refresh");
        let file = temp.path("watched.txt");
        fs::write(&file, "one\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file.clone()],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        let pane = state.active_pane;

        fs::write(&file, "one\ntwo\n").expect("external write");
        let _ = update(&mut state, Message::ExternalFileCheckTick);

        assert_eq!(state.active_editor().text(), "one\ntwo\n");
        assert!(!state.workspace.active_tile().document.buffer.is_dirty());
        assert!(state.is_external_edit_locked(tile_id));
        assert_eq!(state.status_message, "external update loaded: watched.txt");

        let _ = update(
            &mut state,
            Message::Edit(
                pane,
                text_editor::Action::Edit(text_editor::Edit::Insert('X')),
            ),
        );

        assert_eq!(state.active_editor().text(), "one\ntwo\n");
        assert_eq!(
            state.status_message,
            "external edit lock active; unlock to edit"
        );
    }

    #[test]
    fn gui_external_file_unlock_allows_editing_again() {
        let temp = TempArea::new("gui-external-unlock");
        let file = temp.path("watched.txt");
        fs::write(&file, "one\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file.clone()],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        let pane = state.active_pane;
        fs::write(&file, "external\n").expect("external write");
        let _ = update(&mut state, Message::ExternalFileCheckTick);

        let _ = update(&mut state, Message::UnlockExternalEdit(tile_id));
        let _ = update(
            &mut state,
            Message::Edit(
                pane,
                text_editor::Action::Edit(text_editor::Edit::Insert('X')),
            ),
        );

        assert!(!state.is_external_edit_locked(tile_id));
        assert_eq!(state.active_editor().text(), "Xexternal\n");
        assert!(state.workspace.active_tile().document.buffer.is_dirty());
    }

    #[test]
    fn gui_external_file_change_continues_refreshing_while_locked() {
        let temp = TempArea::new("gui-external-refresh-locked");
        let file = temp.path("watched.txt");
        fs::write(&file, "one\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file.clone()],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        fs::write(&file, "external one\n").expect("external write one");
        let _ = update(&mut state, Message::ExternalFileCheckTick);

        fs::write(&file, "external one\nexternal two\n").expect("external write two");
        let _ = update(&mut state, Message::ExternalFileCheckTick);

        assert_eq!(state.active_editor().text(), "external one\nexternal two\n");
        assert!(state.is_external_edit_locked(tile_id));
    }

    #[test]
    fn gui_external_file_change_does_not_overwrite_dirty_tile() {
        let temp = TempArea::new("gui-external-dirty-conflict");
        let file = temp.path("watched.txt");
        fs::write(&file, "one\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file.clone()],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("local dirty\n");
        state.sync_active_editor_to_document();

        fs::write(&file, "external replacement\n").expect("external write");
        let _ = update(&mut state, Message::ExternalFileCheckTick);

        assert_eq!(state.active_editor().text(), "local dirty\n");
        assert!(state.workspace.active_tile().document.buffer.is_dirty());
        assert!(state.is_external_edit_locked(tile_id));
        assert!(state
            .status_message
            .contains("save or close local edits before refresh"));
    }

    #[test]
    fn gui_browser_file_single_click_selects_without_opening_tile() {
        let temp = TempArea::new("gui-browser-open");
        let file = temp.path("from-browser.txt");
        fs::write(&file, "browser file\n").expect("write browser file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let initial_path = state.workspace.active_tile().document.path.clone();
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "from-browser.txt")
            .expect("browser file entry");

        state.select_browser_entry(index);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, initial_path);
        assert_eq!(
            state
                .browser
                .as_ref()
                .expect("browser")
                .selected_entry()
                .expect("selected")
                .path,
            file
        );
    }

    #[test]
    fn gui_browser_file_double_click_replaces_initial_blank_tile() {
        let temp = TempArea::new("gui-browser-open-double");
        let file = temp.path("from-browser.txt");
        fs::write(&file, "browser file\n").expect("write browser file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "from-browser.txt")
            .expect("browser file entry");

        state.activate_browser_entry(index);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, file);
        assert_eq!(state.active_editor().text(), "browser file\n");
    }

    #[test]
    fn gui_browser_tree_file_selection_does_not_open_tile() {
        let temp = TempArea::new("gui-browser-tree-file");
        let file = temp.path("from-tree.txt");
        fs::write(&file, "tree file\n").expect("write browser file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let initial_path = state.workspace.active_tile().document.path.clone();

        let _ = update(
            &mut state,
            Message::BrowserTreeEvent(DirectoryTreeEvent::Selected(
                file.clone(),
                false,
                iced_swdir_tree::SelectionMode::Replace,
            )),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, initial_path);
        assert_eq!(
            state
                .browser
                .as_ref()
                .expect("browser")
                .selected_entry()
                .expect("selected")
                .path,
            file
        );
        assert_eq!(state.browser_selected_path.as_deref(), Some(file.as_path()));
    }

    #[test]
    fn gui_browser_tree_file_double_click_uses_existing_open_adapter() {
        let temp = TempArea::new("gui-browser-tree-file-double");
        let file = temp.path("from-tree.txt");
        fs::write(&file, "tree file\n").expect("write browser file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );

        let _ = update(
            &mut state,
            Message::BrowserLocalTreeActivated(file.clone(), false),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, file);
        assert_eq!(state.active_editor().text(), "tree file\n");
    }

    #[test]
    fn gui_browser_tree_directory_selection_does_not_reset_root() {
        let temp = TempArea::new("gui-browser-tree-dir");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        fs::write(subdir.join("inside.txt"), "inside\n").expect("write inside");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );

        let _ = update(
            &mut state,
            Message::BrowserTreeEvent(DirectoryTreeEvent::Selected(
                subdir.clone(),
                true,
                iced_swdir_tree::SelectionMode::Replace,
            )),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(
            state.browser.as_ref().expect("browser").sidebar.current_dir,
            temp.root.canonicalize().expect("canonical root")
        );
        assert_eq!(
            state
                .browser
                .as_ref()
                .expect("browser")
                .selected_entry()
                .expect("selected")
                .path,
            subdir
        );
        assert_eq!(
            state.browser_selected_path.as_deref(),
            Some(subdir.as_path())
        );
    }

    #[test]
    fn gui_browser_tree_directory_double_click_resets_root_without_opening_tile() {
        let temp = TempArea::new("gui-browser-tree-dir-double");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        fs::write(subdir.join("inside.txt"), "inside\n").expect("write inside");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );

        let _ = update(
            &mut state,
            Message::BrowserLocalTreeActivated(subdir.clone(), true),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(
            state.browser.as_ref().expect("browser").sidebar.current_dir,
            subdir.canonicalize().expect("canonical subdir")
        );
        assert_eq!(
            state
                .browser_tree
                .as_ref()
                .expect("tree")
                .root_path()
                .to_path_buf(),
            subdir.canonicalize().expect("canonical tree subdir")
        );
        assert!(state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "inside.txt"));
    }

    #[test]
    fn gui_browser_parent_request_resets_tree_root_to_parent_directory() {
        let temp = TempArea::new("gui-browser-tree-parent");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            subdir.clone(),
        );

        let _ = update(&mut state, Message::BrowserParentRequested);

        let parent = temp.root.canonicalize().expect("canonical parent");
        assert_eq!(
            state.browser.as_ref().expect("browser").sidebar.current_dir,
            parent
        );
        assert_eq!(
            state
                .browser_tree
                .as_ref()
                .expect("tree")
                .root_path()
                .to_path_buf(),
            parent
        );
    }

    #[test]
    fn gui_browser_refresh_picks_up_external_file_creation() {
        let temp = TempArea::new("gui-browser-refresh");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        assert!(!state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "external.txt"));

        fs::write(temp.path("external.txt"), "external\n").expect("write external file");

        let _ = update(&mut state, Message::BrowserRefreshRequested);

        assert!(state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "external.txt"));
        assert_eq!(
            state.status_message,
            format!(
                "refreshed {}",
                temp.root.canonicalize().expect("canonical root").display()
            )
        );
    }

    #[test]
    fn gui_browser_create_file_creates_refreshes_and_opens_new_file() {
        let temp = TempArea::new("gui-browser-create-file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let created = temp.path("created.txt");

        let _ = update(&mut state, Message::BrowserCreateFileRequested);
        assert_eq!(state.path_prompt, Some(GuiPathPrompt::BrowserCreateFile));
        let _ = update(
            &mut state,
            Message::PathPromptChanged("created.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        assert!(created.exists());
        assert_eq!(fs::read_to_string(&created).expect("read created file"), "");
        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, created);
        assert_eq!(state.active_editor().text(), "");
        assert!(state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "created.txt"));
        assert_eq!(
            state.status_message,
            format!("created {}", created.display())
        );
    }

    #[test]
    fn gui_browser_create_file_targets_selected_directory() {
        let temp = TempArea::new("gui-browser-create-file-selected-dir");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "subdir/")
            .expect("subdir entry");
        state.select_browser_entry(index);

        let _ = update(&mut state, Message::BrowserCreateFileRequested);
        let _ = update(
            &mut state,
            Message::PathPromptChanged("nested.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        let created = subdir.join("nested.txt");
        assert!(created.exists());
        assert_eq!(state.workspace.active_tile().document.path, created);
        assert_eq!(
            state.status_message,
            format!("created {}", created.display())
        );
    }

    #[test]
    fn gui_browser_create_directory_targets_selected_directory() {
        let temp = TempArea::new("gui-browser-create-dir-selected-dir");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "subdir/")
            .expect("subdir entry");
        state.select_browser_entry(index);

        let _ = update(&mut state, Message::BrowserCreateDirectoryRequested);
        assert_eq!(
            state.path_prompt,
            Some(GuiPathPrompt::BrowserCreateDirectory)
        );
        let _ = update(&mut state, Message::PathPromptChanged("child".to_string()));
        let _ = update(&mut state, Message::SubmitPathPrompt);

        let created = subdir.join("child");
        assert!(created.is_dir());
        assert_eq!(
            state.status_message,
            format!("created directory {}", created.display())
        );
    }

    #[test]
    fn gui_browser_create_file_targets_tree_selected_nested_directory() {
        let temp = TempArea::new("gui-browser-create-file-tree-selected-dir");
        let subdir = temp.path("subdir");
        let nested = subdir.join("nested");
        fs::create_dir(&subdir).expect("create subdir");
        fs::create_dir(&nested).expect("create nested");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );

        let _ = update(
            &mut state,
            Message::BrowserLocalTreeSelected(nested.clone(), true),
        );
        let _ = update(&mut state, Message::BrowserCreateFileRequested);
        let _ = update(
            &mut state,
            Message::PathPromptChanged("created.txt".to_string()),
        );
        let _ = update(&mut state, Message::SubmitPathPrompt);

        let created = nested.join("created.txt");
        assert!(created.exists());
        assert_eq!(state.workspace.active_tile().document.path, created);
        assert_eq!(
            state.browser_selected_path.as_deref(),
            Some(created.as_path())
        );
    }

    #[test]
    fn gui_browser_delete_file_requires_confirmation() {
        let temp = TempArea::new("gui-browser-delete-file");
        let file = temp.path("delete-me.txt");
        fs::write(&file, "delete\n").expect("write delete file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "delete-me.txt")
            .expect("delete file entry");
        state.select_browser_entry(index);

        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(file.exists());
        assert!(state.status_message.contains("click delete again"));

        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(!file.exists());
        assert_eq!(
            state.status_message,
            format!("deleted file {}", file.display())
        );
    }

    #[test]
    fn gui_browser_delete_targets_tree_selected_nested_file() {
        let temp = TempArea::new("gui-browser-delete-tree-file");
        let subdir = temp.path("subdir");
        fs::create_dir(&subdir).expect("create subdir");
        let file = subdir.join("delete-me.txt");
        fs::write(&file, "delete\n").expect("write delete file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );

        let _ = update(
            &mut state,
            Message::BrowserLocalTreeSelected(file.clone(), false),
        );
        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(file.exists());
        assert!(state.status_message.contains("click delete again"));

        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(!file.exists());
        assert_eq!(
            state.status_message,
            format!("deleted file {}", file.display())
        );
    }

    #[test]
    fn gui_browser_delete_directory_warns_and_removes_tree_after_confirmation() {
        let temp = TempArea::new("gui-browser-delete-dir");
        let directory = temp.path("delete-dir");
        fs::create_dir(&directory).expect("create delete dir");
        fs::create_dir(directory.join("child")).expect("create child dir");
        fs::write(directory.join("child").join("nested.txt"), "nested\n").expect("write nested");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "delete-dir/")
            .expect("delete dir entry");
        state.select_browser_entry(index);

        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(directory.exists());
        assert!(state.status_message.contains("all subdirectories/files"));

        let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
        assert!(!directory.exists());
        assert_eq!(
            state.status_message,
            format!("deleted directory {}", directory.display())
        );
    }

    #[test]
    fn gui_left_panel_switches_between_files_workspaces_and_preferences_without_project_io() {
        let temp = TempArea::new("gui-left-panel-mode");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );

        assert!(state.left_panel.visible);
        assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Files);
        assert!(state.browser_visible);
        let initial_tiles = state.workspace.tiles.len();

        let _ = update(
            &mut state,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
        );

        assert!(state.left_panel.visible);
        assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Workspaces);
        assert!(state.browser_visible);
        assert_eq!(state.workspace.tiles.len(), initial_tiles);
        assert_eq!(state.status_message, "workspaces panel shown");

        let _ = update(
            &mut state,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Preferences),
        );
        assert!(state.left_panel.visible);
        assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Preferences);
        assert!(state.browser_visible);
        assert_eq!(state.workspace.tiles.len(), initial_tiles);
        assert_eq!(state.status_message, "preferences panel shown");

        let _ = update(&mut state, Message::ToggleBrowser);
        assert!(!state.left_panel.visible);
        assert!(!state.browser_visible);
        assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Preferences);

        let _ = update(
            &mut state,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Files),
        );
        assert!(state.left_panel.visible);
        assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Files);
        assert!(state.browser_visible);
        assert_eq!(state.workspace.tiles.len(), initial_tiles);
    }

    #[test]
    fn gui_workspace_panel_saves_current_workspace_project() {
        let temp = TempArea::new("gui-workspace-save-current");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let projects_dir = temp.path("workspaces");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());
        state.workspace_projects.clear();

        let _ = update(
            &mut state,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
        );
        let _ = update(&mut state, Message::SaveCurrentWorkspaceProject);

        let project_path = projects_dir.join("current-workspace.v1");
        let text = fs::read_to_string(project_path).expect("read saved project");
        let project = kfnotepad::parse_gui_workspace_project(&text).expect("parse project");
        assert_eq!(project.name, "current workspace");
        assert_eq!(project.files, vec![first, second]);
        assert_eq!(project.active_ordinal, 1);
        assert!(project.layout.is_some());
        assert_eq!(state.workspace_projects.len(), 1);
        assert_eq!(
            state.workspace_projects[0].project.name,
            "current workspace"
        );
        assert_eq!(
            state.status_message,
            "workspace project saved: current workspace"
        );
    }

    #[test]
    fn gui_restore_last_workspace_toggle_saves_current_workspace_for_relaunch() {
        let temp = TempArea::new("gui-restore-toggle-autosave");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
            Some(config.clone()),
            None,
            None,
            Some(projects_dir.clone()),
        );

        let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        assert!(project_path.exists());
        let restored = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(restored.workspace.tiles.len(), 2);
        assert_eq!(restored.workspace.active_tile().document.path, second);
        assert_eq!(restored.active_editor().text(), "second\n");
        assert!(restored
            .status_message
            .contains("restored last workspace project current workspace"));
    }

    #[test]
    fn gui_restore_last_workspace_updates_snapshot_after_later_file_open() {
        let temp = TempArea::new("gui-restore-open-autosave");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let opened = temp.path("opened.txt");
        fs::write(&opened, "opened later\n").expect("write opened");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config.clone()),
            None,
            None,
            Some(projects_dir.clone()),
        );

        let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));
        assert_eq!(state.active_editor().text(), "");

        assert!(state.open_path_in_new_pane(opened.clone()));

        let restored = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(restored.workspace.tiles.len(), 1);
        assert_eq!(restored.workspace.active_tile().document.path, opened);
        assert_eq!(restored.active_editor().text(), "opened later\n");
        assert!(restored
            .status_message
            .contains("restored last workspace project current workspace"));
    }

    #[test]
    fn gui_restore_last_workspace_updates_snapshot_from_explicit_launch_files() {
        let temp = TempArea::new("gui-restore-explicit-launch-autosave");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first launch\n").expect("write first");
        fs::write(&second, "second launch\n").expect("write second");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");

        let launched = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
            Some(config.clone()),
            None,
            None,
            Some(projects_dir.clone()),
        );
        assert_eq!(launched.workspace.tiles.len(), 2);
        assert_eq!(launched.workspace.active_tile().document.path, second);
        assert!(!launched.status_message.contains("restored last workspace"));

        let restored = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(restored.workspace.tiles.len(), 2);
        assert_eq!(restored.workspace.active_tile().document.path, second);
        assert_eq!(restored.active_editor().text(), "second launch\n");
        assert!(restored
            .status_message
            .contains("restored last workspace project current workspace"));
    }

    #[test]
    fn gui_workspace_panel_saves_named_workspace_project() {
        let temp = TempArea::new("gui-workspace-save-named");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let projects_dir = temp.path("workspaces");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());

        let _ = update(
            &mut state,
            Message::WorkspaceProjectNameChanged("Client Notes".to_string()),
        );
        let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

        let named_path = projects_dir.join("client-notes.v1");
        assert!(named_path.exists());
        let project = kfnotepad::parse_gui_workspace_project(
            &fs::read_to_string(named_path).expect("read named project"),
        )
        .expect("parse named project");
        assert_eq!(project.name, "Client Notes");
        assert_eq!(project.files, vec![first, second]);
        assert_eq!(project.active_ordinal, 1);
        assert!(project.layout.is_some());
        assert!(!projects_dir.join("current-workspace.v1").exists());
        assert_eq!(state.workspace_projects.len(), 1);
        assert_eq!(state.workspace_projects[0].project.name, "Client Notes");
        assert_eq!(
            state.status_message,
            "workspace project saved: Client Notes"
        );
    }

    #[test]
    fn gui_workspace_panel_rejects_empty_or_invalid_project_names_without_writes() {
        let temp = TempArea::new("gui-workspace-save-invalid-name");
        let file = temp.path("file.txt");
        fs::write(&file, "file\n").expect("write file");
        let projects_dir = temp.path("workspaces");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());
        state.workspace_projects.clear();

        let _ = update(
            &mut state,
            Message::WorkspaceProjectNameChanged("   ".to_string()),
        );
        let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

        assert_eq!(
            state.status_message,
            "workspace save failed: project name required"
        );
        assert!(!projects_dir.exists());

        let _ = update(
            &mut state,
            Message::WorkspaceProjectNameChanged("../bad".to_string()),
        );
        let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

        assert_eq!(
            state.status_message,
            "workspace save failed: invalid project name"
        );
        assert!(!projects_dir.exists());
        assert!(state.workspace_projects.is_empty());
    }

    #[test]
    fn gui_workspace_panel_named_save_does_not_change_current_workspace_restore_target() {
        let temp = TempArea::new("gui-workspace-save-named-current");
        let current_file = temp.path("current.txt");
        let named_file = temp.path("named.txt");
        fs::write(&current_file, "current\n").expect("write current");
        fs::write(&named_file, "named\n").expect("write named");
        let projects_dir = temp.path("workspaces");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![current_file.clone()],
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());

        let _ = update(&mut state, Message::SaveCurrentWorkspaceProject);
        state.open_path_in_new_pane(named_file.clone());
        let _ = update(
            &mut state,
            Message::WorkspaceProjectNameChanged("Named Workspace".to_string()),
        );
        let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

        let current = kfnotepad::parse_gui_workspace_project(
            &fs::read_to_string(projects_dir.join("current-workspace.v1"))
                .expect("read current project"),
        )
        .expect("parse current project");
        let named = kfnotepad::parse_gui_workspace_project(
            &fs::read_to_string(projects_dir.join("named-workspace.v1"))
                .expect("read named project"),
        )
        .expect("parse named project");

        assert_eq!(current.name, "current workspace");
        assert_eq!(current.files, vec![current_file]);
        assert_eq!(named.name, "Named Workspace");
        assert_eq!(named.files, vec![current.files[0].clone(), named_file]);
    }

    #[test]
    fn gui_workspace_panel_refresh_lists_projects_in_deterministic_order() {
        let temp = TempArea::new("gui-workspace-refresh");
        let projects_dir = temp.path("workspaces");
        fs::create_dir_all(&projects_dir).expect("create projects dir");
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
        fs::write(projects_dir.join("broken.v1"), "bad").expect("write broken");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir);
        state.workspace_projects.clear();

        let _ = update(&mut state, Message::RefreshWorkspaceProjects);

        assert_eq!(
            state
                .workspace_projects
                .iter()
                .map(|entry| entry.project.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
        assert_eq!(state.status_message, "workspace projects: 2");
    }

    #[test]
    fn gui_workspace_project_delete_requires_confirmation_and_removes_project() {
        let temp = TempArea::new("gui-workspace-delete");
        let projects_dir = temp.path("workspaces");
        let project = GuiWorkspaceProject {
            name: "delete me".to_string(),
            files: vec![temp.path("delete.md")],
            active_ordinal: 0,
            layout: None,
        };
        let project_path = projects_dir.join("delete-me.v1");
        save_gui_workspace_project(&project_path, &project).expect("save project");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());
        let _ = update(&mut state, Message::RefreshWorkspaceProjects);

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert_eq!(state.pending_project_delete, Some(0));
        assert!(project_path.exists());
        assert_eq!(
            state.status_message,
            "delete workspace project delete me? click delete again"
        );

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert_eq!(state.pending_project_delete, None);
        assert!(!project_path.exists());
        assert!(state.workspace_projects.is_empty());
        assert_eq!(state.status_message, "workspace project deleted: delete me");
    }

    #[test]
    fn gui_workspace_project_delete_warns_for_restore_target_when_enabled() {
        let temp = TempArea::new("gui-workspace-delete-current");
        let projects_dir = temp.path("workspaces");
        let project = GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![temp.path("current.md")],
            active_ordinal: 0,
            layout: None,
        };
        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(&project_path, &project).expect("save current project");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir.clone());
        state.settings.gui_restore_last_workspace = true;
        let _ = update(&mut state, Message::RefreshWorkspaceProjects);

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert_eq!(state.pending_project_delete, Some(0));
        assert!(project_path.exists());
        assert_eq!(
            state.status_message,
            "restore target selected; delete again to remove last-workspace restore project"
        );

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert!(!project_path.exists());
        assert!(state.workspace_projects.is_empty());
        assert_eq!(
            state.status_message,
            "workspace project deleted: current workspace"
        );
    }

    #[test]
    fn gui_workspace_project_delete_treats_missing_project_as_removed() {
        let temp = TempArea::new("gui-workspace-delete-missing");
        let projects_dir = temp.path("workspaces");
        fs::create_dir_all(&projects_dir).expect("create projects dir");
        let project_path = projects_dir.join("missing.v1");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir);
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: project_path,
            project: GuiWorkspaceProject {
                name: "missing".to_string(),
                files: vec![temp.path("missing.md")],
                active_ordinal: 0,
                layout: None,
            },
        }];

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));
        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert_eq!(state.pending_project_delete, None);
        assert!(state.workspace_projects.is_empty());
        assert_eq!(
            state.status_message,
            "workspace project already missing: missing"
        );
    }

    #[test]
    fn gui_workspace_project_delete_rejects_paths_outside_project_directory() {
        let temp = TempArea::new("gui-workspace-delete-outside");
        let projects_dir = temp.path("workspaces");
        fs::create_dir_all(&projects_dir).expect("create projects dir");
        let outside_path = temp.path("outside.v1");
        fs::write(&outside_path, "outside\n").expect("write outside");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects_dir = Some(projects_dir);
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: outside_path.clone(),
            project: GuiWorkspaceProject {
                name: "outside".to_string(),
                files: vec![temp.path("outside.md")],
                active_ordinal: 0,
                layout: None,
            },
        }];

        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));
        let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

        assert!(outside_path.exists());
        assert!(state
            .status_message
            .contains("workspace project path is outside the project directory"));
    }

    #[test]
    fn gui_workspace_project_click_opens_clean_project_in_current_window() {
        let temp = TempArea::new("gui-workspace-open-clean");
        let original = temp.path("original.txt");
        let first = temp.path("project-first.txt");
        let second = temp.path("project-second.txt");
        fs::write(&original, "original\n").expect("write original");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original],
            },
            temp.root.clone(),
        );
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: temp.path("workspaces").join("project.v1"),
            project: GuiWorkspaceProject {
                name: "project".to_string(),
                files: vec![first.clone(), second.clone()],
                active_ordinal: 1,
                layout: None,
            },
        }];

        let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(
            state
                .workspace
                .tiles
                .iter()
                .map(|tile| tile.document.path.clone())
                .collect::<Vec<_>>(),
            vec![first, second.clone()]
        );
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.active_editor().text(), "second\n");
        assert_eq!(state.status_message, "opened workspace project project");
    }

    #[test]
    fn gui_workspace_project_click_requires_confirmation_for_dirty_workspace() {
        let temp = TempArea::new("gui-workspace-open-dirty");
        let original = temp.path("original.txt");
        let project_file = temp.path("project.txt");
        fs::write(&original, "original\n").expect("write original");
        fs::write(&project_file, "project\n").expect("write project");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );
        state.panes.get_mut(state.active_pane).expect("pane").editor =
            GuiEditorAdapter::from_text("dirty\n");
        state.sync_active_editor_to_document();
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: temp.path("workspaces").join("project.v1"),
            project: GuiWorkspaceProject {
                name: "project".to_string(),
                files: vec![project_file.clone()],
                active_ordinal: 0,
                layout: None,
            },
        }];

        let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

        assert_eq!(state.workspace.active_tile().document.path, original);
        assert_eq!(state.pending_project_open, Some(0));
        assert_eq!(
            state.status_message,
            "unsaved changes; open project again to replace current workspace"
        );

        let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

        assert_eq!(state.pending_project_open, None);
        assert_eq!(state.workspace.active_tile().document.path, project_file);
        assert_eq!(state.active_editor().text(), "project\n");
    }

    #[test]
    fn gui_workspace_project_click_validates_files_before_replacing_workspace() {
        let temp = TempArea::new("gui-workspace-open-invalid");
        let original = temp.path("original.txt");
        let missing = temp.path("missing.txt");
        fs::write(&original, "original\n").expect("write original");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![original.clone()],
            },
            temp.root.clone(),
        );
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: temp.path("workspaces").join("project.v1"),
            project: GuiWorkspaceProject {
                name: "project".to_string(),
                files: vec![missing.clone()],
                active_ordinal: 0,
                layout: None,
            },
        }];
        state.pending_project_open = Some(0);

        let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.pending_project_open, None);
        assert_eq!(state.workspace.active_tile().document.path, original);
        assert!(state
            .status_message
            .starts_with("workspace project open failed: "));
    }

    #[test]
    fn gui_workspace_project_new_window_spawns_current_executable_with_project_env() {
        let temp = TempArea::new("gui-workspace-new-window");
        let project_path = temp.path("workspaces").join("project.v1");
        let file = temp.path("project.txt");
        fs::write(&file, "project\n").expect("write project");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.workspace_projects = vec![GuiWorkspaceProjectEntry {
            path: project_path.clone(),
            project: GuiWorkspaceProject {
                name: "project".to_string(),
                files: vec![file],
                active_ordinal: 0,
                layout: None,
            },
        }];

        let _ = update(&mut state, Message::WorkspaceProjectNewWindowClicked(0));

        assert_eq!(state.spawned_workspace_project_paths, vec![project_path]);
        assert_eq!(
            state.status_message,
            "opened workspace project project in new window"
        );
    }

    #[test]
    fn gui_workspace_project_launch_command_uses_env_without_shell_arguments() {
        let executable = PathBuf::from("/tmp/kfnotepad-gui");
        let project = PathBuf::from("/tmp/project with spaces.v1");
        let command = workspace_project_launch_command(&executable, &project);

        assert_eq!(command.get_program(), executable.as_os_str());
        assert!(command.get_args().next().is_none());
        assert_eq!(
            command
                .get_envs()
                .find(|(name, _value)| *name == WORKSPACE_PROJECT_ENV)
                .and_then(|(_name, value)| value),
            Some(project.as_os_str())
        );
    }

    #[test]
    fn gui_workspace_project_launch_restores_saved_files_layout_and_active_tile() {
        let temp = TempArea::new("gui-workspace-project-launch");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let project_path = temp.path("workspaces").join("project.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let project = GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![first.clone(), second.clone()],
            active_ordinal: 0,
            layout: Some(GuiLayout {
                browser_visible: false,
                browser_width_px: Some(260),
                root: GuiLayoutNode::Split {
                    axis: GuiLayoutAxis::Horizontal,
                    ratio_per_mille: 300,
                    first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                    second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                },
                minimized_ordinals: vec![1],
            }),
        };
        save_gui_workspace_project(&project_path, &project).expect("save project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            Some(project_path),
            None,
        );

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, first);
        assert!(!state.browser_visible);
        assert_eq!(state.browser_width, 260.0);
        assert!(
            state
                .workspace
                .tiles
                .iter()
                .find(|tile| tile.document.path == second)
                .expect("second tile")
                .minimized
        );
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.minimized_panes.len(), 1);
        assert_eq!(state.minimized_panes[0].tile_id, GuiTileId(1));
        assert!(matches!(state.panes.layout(), pane_grid::Node::Pane(_)));
        assert!(state
            .status_message
            .contains("opened workspace project project"));
    }

    #[test]
    fn gui_workspace_auto_restore_is_disabled_by_default() {
        let temp = TempArea::new("gui-auto-restore-disabled");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let file = temp.path("saved.txt");
        fs::write(&file, "saved\n").expect("write saved file");
        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "current workspace".to_string(),
                files: vec![file.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            Some(projects_dir),
        );

        assert_ne!(state.workspace.active_tile().document.path, file);
        assert_eq!(state.active_editor().text(), "");
        assert_eq!(state.status_message, "started empty GUI document tile");
    }

    #[test]
    fn gui_workspace_auto_restore_opens_current_workspace_when_enabled() {
        let temp = TempArea::new("gui-auto-restore-enabled");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "current workspace".to_string(),
                files: vec![first.clone(), second.clone()],
                active_ordinal: 1,
                layout: Some(GuiLayout {
                    browser_visible: false,
                    browser_width_px: Some(240),
                    root: GuiLayoutNode::Split {
                        axis: GuiLayoutAxis::Vertical,
                        ratio_per_mille: 600,
                        first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                        second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                    },
                    minimized_ordinals: vec![0],
                }),
            },
        )
        .expect("save project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.active_editor().text(), "second\n");
        assert!(!state.browser_visible);
        assert_eq!(state.browser_width, 240.0);
        assert!(
            state
                .workspace
                .tiles
                .iter()
                .find(|tile| tile.document.path == first)
                .expect("first tile")
                .minimized
        );
        assert!(state
            .status_message
            .contains("restored last workspace project current workspace"));
    }

    #[test]
    fn gui_workspace_auto_restore_yields_to_explicit_file_args() {
        let temp = TempArea::new("gui-auto-restore-file-precedence");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let saved = temp.path("saved.txt");
        let explicit = temp.path("explicit.txt");
        fs::write(&saved, "saved\n").expect("write saved");
        fs::write(&explicit, "explicit\n").expect("write explicit");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "current workspace".to_string(),
                files: vec![saved],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![explicit.clone()],
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, explicit);
        assert_eq!(state.active_editor().text(), "explicit\n");
        assert!(!state.status_message.contains("restored last workspace"));
    }

    #[test]
    fn gui_workspace_auto_restore_yields_to_explicit_project_launch() {
        let temp = TempArea::new("gui-auto-restore-project-precedence");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let current = temp.path("current.txt");
        let explicit = temp.path("explicit.txt");
        fs::write(&current, "current\n").expect("write current");
        fs::write(&explicit, "explicit\n").expect("write explicit");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let current_project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(
            &current_project_path,
            &GuiWorkspaceProject {
                name: "current workspace".to_string(),
                files: vec![current],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save current project");
        let explicit_project = temp.path("explicit-project.v1");
        save_gui_workspace_project(
            &explicit_project,
            &GuiWorkspaceProject {
                name: "explicit".to_string(),
                files: vec![explicit.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save explicit project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            Some(explicit_project),
            Some(projects_dir),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, explicit);
        assert_eq!(state.active_editor().text(), "explicit\n");
        assert!(state
            .status_message
            .contains("opened workspace project explicit"));
        assert!(!state.status_message.contains("restored last workspace"));
    }

    #[test]
    fn gui_workspace_auto_restore_invalid_path_falls_back_without_writing_files() {
        let temp = TempArea::new("gui-auto-restore-invalid");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
        let existing = temp.path("existing.txt");
        let missing = temp.path("missing.txt");
        fs::write(&existing, "unchanged\n").expect("write existing");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
        fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
        let project_path =
            gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
        save_gui_workspace_project(
            &project_path,
            &GuiWorkspaceProject {
                name: "current workspace".to_string(),
                files: vec![missing.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            Some(config),
            None,
            None,
            Some(projects_dir),
        );

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.active_editor().text(), "");
        assert!(state
            .status_message
            .contains("workspace auto-restore failed: "));
        assert_eq!(
            fs::read_to_string(existing).expect("read existing"),
            "unchanged\n"
        );
        let repaired_project =
            load_workspace_project_launch(&project_path).expect("read repaired current project");
        assert_eq!(
            repaired_project.files,
            vec![state.workspace.active_tile().document.path.clone()]
        );
        assert_ne!(repaired_project.files, vec![missing]);
    }

    #[test]
    fn gui_browser_clicks_are_ignored_while_workspace_panel_is_active() {
        let temp = TempArea::new("gui-left-panel-ignore-browser");
        let file = temp.path("from-browser.txt");
        fs::write(&file, "browser file\n").expect("write browser file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let initial_path = state.workspace.active_tile().document.path.clone();
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "from-browser.txt")
            .expect("browser file entry");

        let _ = update(
            &mut state,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
        );
        state.activate_browser_entry(index);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, initial_path);
        assert_ne!(state.workspace.active_tile().document.path, file);
    }

    #[test]
    fn gui_browser_directory_single_click_selects_without_navigation() {
        let temp = TempArea::new("gui-browser-nav");
        fs::create_dir(temp.path("subdir")).expect("create subdir");
        fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "subdir/")
            .expect("subdir entry");

        state.select_browser_entry(index);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(
            state.browser.as_ref().expect("browser").sidebar.current_dir,
            temp.root.canonicalize().expect("canonical root")
        );
        assert_eq!(
            state
                .browser
                .as_ref()
                .expect("browser")
                .selected_entry()
                .expect("selected")
                .path,
            temp.path("subdir")
        );
    }

    #[test]
    fn gui_browser_directory_double_click_navigates_without_opening_pane() {
        let temp = TempArea::new("gui-browser-nav-double");
        fs::create_dir(temp.path("subdir")).expect("create subdir");
        fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "subdir/")
            .expect("subdir entry");

        state.activate_browser_entry(index);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(
            state.browser.as_ref().expect("browser").sidebar.current_dir,
            temp.path("subdir")
                .canonicalize()
                .expect("canonical subdir")
        );
        assert!(state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "inside.txt"));
    }

    #[test]
    fn gui_browser_toggle_hides_and_restores_open_behavior() {
        let temp = TempArea::new("gui-browser-toggle");
        let file = temp.path("visible-again.txt");
        fs::write(&file, "visible\n").expect("write file");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
            None,
            None,
            None,
            None,
        );
        let index = state
            .browser
            .as_ref()
            .expect("browser")
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "visible-again.txt")
            .expect("browser file entry");

        let _ = update(&mut state, Message::ToggleBrowser);
        assert!(!state.browser_visible);
        state.activate_browser_entry(index);
        assert_eq!(state.workspace.tiles.len(), 1);

        let _ = update(&mut state, Message::ToggleBrowser);
        assert!(state.browser_visible);
        state.select_browser_entry(index);
        assert_ne!(state.workspace.active_tile().document.path, file);
        state.activate_browser_entry(index);
        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, file);
    }

    #[test]
    fn gui_close_active_clean_pane_removes_tile_and_focuses_fallback() {
        let temp = TempArea::new("gui-close-clean");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, first);
        assert!(state.status_message.starts_with("closed "));
    }

    #[test]
    fn gui_close_dirty_pane_requires_second_request() {
        let temp = TempArea::new("gui-close-dirty");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("discard me\n");

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.panes.len(), 2);
        assert!(state
            .status_message
            .contains("unsaved changes; close again"));

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, first);
        assert_eq!(
            fs::read_to_string(&second).expect("read second"),
            "second\n"
        );
    }

    #[test]
    fn gui_window_close_clean_state_allows_close_without_prompt() {
        let temp = TempArea::new("gui-window-close-clean");
        let file = temp.path("clean.txt");
        fs::write(&file, "clean\n").expect("write clean");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });

        let _task = update(
            &mut state,
            Message::WindowCloseRequested(window::Id::unique()),
        );

        assert!(!state.pending_app_quit);
        assert!(!state
            .status_message
            .contains("close window again to discard"));
    }

    #[test]
    fn gui_window_close_dirty_state_requires_second_request_without_saving() {
        let temp = TempArea::new("gui-window-close-dirty");
        let file = temp.path("dirty.txt");
        fs::write(&file, "original\n").expect("write dirty");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("discard from app close\n");

        let _task = update(
            &mut state,
            Message::WindowCloseRequested(window::Id::unique()),
        );

        assert!(state.pending_app_quit);
        assert_eq!(
            state.status_message,
            "unsaved changes; close window again to discard all dirty tiles"
        );
        assert_eq!(
            fs::read_to_string(&file).expect("read original"),
            "original\n"
        );

        let _task = update(
            &mut state,
            Message::WindowCloseRequested(window::Id::unique()),
        );

        assert!(state.pending_app_quit);
        assert_eq!(
            fs::read_to_string(&file).expect("read original after confirm"),
            "original\n"
        );
    }

    #[test]
    fn gui_ctrl_q_quit_uses_window_close_dirty_confirmation() {
        let temp = TempArea::new("gui-ctrl-q-dirty");
        let file = temp.path("dirty.txt");
        fs::write(&file, "original\n").expect("write dirty");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("dirty from ctrl-q\n");

        let _task = update(&mut state, Message::QuitRequested(window::Id::unique()));

        assert!(state.pending_app_quit);
        assert_eq!(
            state.status_message,
            "unsaved changes; close window again to discard all dirty tiles"
        );
        assert_eq!(
            fs::read_to_string(&file).expect("read original"),
            "original\n"
        );
    }

    #[test]
    fn gui_window_close_pending_confirmation_clears_after_save() {
        let temp = TempArea::new("gui-window-close-save-clears");
        let file = temp.path("dirty.txt");
        fs::write(&file, "original\n").expect("write dirty");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("saved before close\n");

        let _task = update(
            &mut state,
            Message::WindowCloseRequested(window::Id::unique()),
        );
        assert!(state.pending_app_quit);

        let _ = update(&mut state, Message::SaveRequested);

        assert!(!state.pending_app_quit);
        assert_eq!(
            fs::read_to_string(&file).expect("read saved"),
            "saved before close\n"
        );
    }

    #[test]
    fn gui_search_next_and_previous_update_shared_and_editor_cursor() {
        let temp = TempArea::new("gui-search-next-prev");
        let file = temp.path("search.txt");
        fs::write(&file, "alpha\nbeta alpha\n").expect("write search");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });

        let _ = update(&mut state, Message::SearchQueryChanged("alpha".to_string()));
        let _ = update(&mut state, Message::SearchNext);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 5 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 1, column: 5 }
        );
        assert_eq!(state.active_editor().selection().as_deref(), Some("alpha"));
        assert_eq!(state.status_message, "found next: alpha");
        let highlight = state.search_highlight.as_ref().expect("search highlight");
        assert_eq!(highlight.tile_id, state.workspace.active_tile().id);
        assert_eq!(highlight.query, "alpha");

        let _ = update(&mut state, Message::SearchPrevious);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(state.active_editor().selection().as_deref(), Some("alpha"));
        assert_eq!(state.status_message, "found previous: alpha");
        assert_eq!(
            state
                .search_highlight
                .as_ref()
                .map(|highlight| highlight.query.as_str()),
            Some("alpha")
        );

        let _ = update(&mut state, Message::SearchQueryChanged("beta".to_string()));
        assert_eq!(state.search_highlight, None);
    }

    #[test]
    fn gui_find_history_keeps_ten_recent_unique_queries() {
        let temp = TempArea::new("gui-find-history");
        let file = temp.path("search.txt");
        fs::write(&file, "alpha beta gamma delta epsilon\n").expect("write search file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        for query in [
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
            "lambda", "beta",
        ] {
            let _ = update(&mut state, Message::SearchQueryChanged(query.to_string()));
            let _ = update(&mut state, Message::SearchNext);
        }

        assert_eq!(state.search_history.len(), GUI_FIND_HISTORY_LIMIT);
        assert_eq!(
            state.search_history.first().map(String::as_str),
            Some("beta")
        );
        assert_eq!(
            state
                .search_history
                .iter()
                .filter(|entry| entry.as_str() == "beta")
                .count(),
            1
        );
        assert!(!state.search_history.iter().any(|entry| entry == "alpha"));

        let _ = update(&mut state, Message::SearchQueryChanged(String::new()));
        assert!(state.search_history_open);

        let _ = update(
            &mut state,
            Message::SearchHistorySelected("theta".to_string()),
        );
        assert_eq!(state.search_query, "theta");
        assert_eq!(
            state.search_history.first().map(String::as_str),
            Some("theta")
        );
        assert!(!state.search_history_open);
    }

    #[test]
    fn gui_search_reports_missing_query_and_no_match() {
        let temp = TempArea::new("gui-search-status");
        let file = temp.path("search.txt");
        fs::write(&file, "alpha\n").expect("write search");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });

        let _ = update(&mut state, Message::SearchNext);
        assert_eq!(state.status_message, "search query required");

        let _ = update(
            &mut state,
            Message::SearchQueryChanged("missing".to_string()),
        );
        let _ = update(&mut state, Message::SearchNext);
        assert_eq!(state.status_message, "no match: missing");
        assert_eq!(state.search_highlight, None);
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 0 }
        );
    }

    #[test]
    fn gui_document_edge_navigation_updates_editor_cursor() {
        let temp = TempArea::new("gui-edge-navigation");
        let file = temp.path("nav.txt");
        fs::write(&file, "one\ntwo\nthree").expect("write nav");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });

        let _ = update(&mut state, Message::GoDocumentEnd);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 2, column: 5 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 2, column: 5 }
        );
        assert_eq!(state.status_message, "moved to document end");

        let _ = update(&mut state, Message::GoDocumentStart);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(state.status_message, "moved to document start");
    }

    #[test]
    fn gui_go_to_line_updates_shared_and_editor_cursor() {
        let temp = TempArea::new("gui-go-to-line");
        let file = temp.path("goto.txt");
        fs::write(&file, "one\ntwo words\nthree\n").expect("write goto");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor
            .move_to(DocumentCursor { row: 0, column: 99 });

        let _ = update(&mut state, Message::GoToLineQueryChanged("2".to_string()));
        let _ = update(&mut state, Message::GoToLineRequested);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 9 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 1, column: 9 }
        );
        assert_eq!(state.status_message, "Line 2");
    }

    #[test]
    fn gui_go_to_line_reports_validation_without_moving_cursor() {
        let temp = TempArea::new("gui-go-to-line-validation");
        let file = temp.path("goto.txt");
        fs::write(&file, "one\ntwo\n").expect("write goto");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor
            .move_to(DocumentCursor { row: 0, column: 1 });

        for (query, expected_status) in [
            ("", "Line number is empty"),
            ("abc", "Line number is invalid"),
            ("99", "Line out of range: 99"),
        ] {
            let _ = update(&mut state, Message::GoToLineQueryChanged(query.to_string()));
            let _ = update(&mut state, Message::GoToLineRequested);
            assert_eq!(state.status_message, expected_status);
            assert_eq!(
                state.workspace.active_tile().state.cursor,
                DocumentCursor { row: 0, column: 1 }
            );
        }
    }

    #[test]
    fn gui_viewport_scroll_message_routes_to_active_editor() {
        let temp = TempArea::new("gui-viewport-scroll-message");
        let file = temp.path("viewport.txt");
        let text = numbered_lines(100);
        fs::write(&file, &text).expect("write viewport");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::ScrollActiveEditorViewport(2));

        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 2, column: 0 }
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 2, column: 0 }
        );
        assert_eq!(
            state
                .active_editor()
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 3,
                text: gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );
        assert_eq!(state.status_message, "viewport down");

        let _ = update(&mut state, Message::ScrollActiveEditorViewport(-99));

        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 2, column: 0 }
        );
        assert_eq!(
            state
                .active_editor()
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 1,
                text: gui_line_number_gutter_text(1, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );
        assert_eq!(state.status_message, "viewport up");
    }

    #[test]
    fn gui_native_editor_scroll_keeps_gutter_synced_without_dirtying_document() {
        let temp = TempArea::new("gui-native-scroll-gutter-sync");
        let file = temp.path("native-scroll.txt");
        let text = numbered_lines(100);
        fs::write(&file, &text).expect("write native scroll");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;
        let tile_id = state.workspace.active_tile().id;

        let _ = update(
            &mut state,
            Message::Edit(pane, text_editor::Action::Scroll { lines: 5 }),
        );

        assert!(!state
            .workspace
            .tile(tile_id)
            .expect("tile")
            .document
            .buffer
            .is_dirty());
        assert_eq!(
            state
                .active_editor()
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 6,
                text: gui_line_number_gutter_text(6, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );
        assert_eq!(state.status_message, "scrolled");
    }

    #[test]
    fn gui_replacement_editor_wheel_delta_maps_to_viewport_lines() {
        let settings = EditorSettings {
            gui_font_size: 20,
            ..EditorSettings::default()
        };

        assert_eq!(
            gui_editor_replacement_scroll_delta_lines(
                mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
                settings,
            ),
            3
        );
        assert_eq!(
            gui_editor_replacement_scroll_delta_lines(
                mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 },
                settings,
            ),
            -2
        );
        assert_eq!(
            gui_editor_replacement_scroll_delta_lines(
                mouse::ScrollDelta::Pixels {
                    x: 0.0,
                    y: -(20.0 * GUI_EDITOR_LINE_HEIGHT * 2.0),
                },
                settings,
            ),
            2
        );
    }

    #[test]
    fn gui_replacement_editor_wheel_scrolls_tile_without_dirtying_document() {
        let temp = TempArea::new("gui-replacement-wheel-scroll");
        let file = temp.path("replacement-wheel.txt");
        let text = numbered_lines(100);
        fs::write(&file, &text).expect("write wheel file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;
        let tile_id = state.workspace.active_tile().id;

        let _ = update(&mut state, Message::ReplacementEditorWheelScrolled(pane, 4));

        let tile = state.workspace.tile(tile_id).expect("tile");
        assert!(!tile.document.buffer.is_dirty());
        assert_eq!(tile.state.cursor, DocumentCursor { row: 0, column: 0 });
        assert_eq!(
            state
                .panes
                .get(pane)
                .expect("pane")
                .editor
                .document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(
            state
                .panes
                .get(pane)
                .expect("pane")
                .editor
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 5,
                text: gui_line_number_gutter_text(5, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );
        assert_eq!(state.status_message, "viewport down");
    }

    #[test]
    fn gui_menu_surface_lists_primary_command_groups() {
        assert!(gui_menu_uses_iced_aw_menu_tree());
        assert_eq!(
            gui_menu_submenu_policy(),
            "Keep current root command groups flat until a group gains enough depth to justify nested hover submenus."
        );
        assert_eq!(
            gui_menu_groups().map(gui_menu_group_label),
            ["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
        );
        assert_eq!(
            gui_menu_groups()
                .map(gui_menu_group_chrome_label)
                .into_iter()
                .collect::<Vec<_>>(),
            vec!["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
        );
        assert_eq!(
            gui_menu_dropdown_labels(GuiMenuGroup::File),
            vec![
                LABEL_NEW_TILE,
                LABEL_OPEN,
                LABEL_OPEN_PATH,
                LABEL_SAVE,
                LABEL_SAVE_AS,
                LABEL_SAVE_AS_PATH,
                LABEL_CLOSE_TILE,
                LABEL_QUIT,
            ]
        );
        assert_eq!(
            gui_menu_dropdown_labels(GuiMenuGroup::Edit),
            vec![
                LABEL_UNDO,
                LABEL_REDO,
                LABEL_COPY,
                LABEL_CUT,
                LABEL_PASTE,
                LABEL_SELECT_ALL,
                LABEL_FIND_NEXT,
                LABEL_FIND_PREVIOUS,
            ]
        );
        assert_eq!(gui_menu_group_index(GuiMenuGroup::File), 0);
        assert_eq!(gui_menu_group_index(GuiMenuGroup::Tile), 5);
        assert_eq!(gui_menu_group_index(GuiMenuGroup::Help), 6);

        let file_commands: Vec<_> = gui_menu_items(GuiMenuGroup::File)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(
            file_commands,
            vec![
                GuiMenuCommand::NewTile,
                GuiMenuCommand::Open,
                GuiMenuCommand::OpenPath,
                GuiMenuCommand::Save,
                GuiMenuCommand::SaveAs,
                GuiMenuCommand::SaveAsPath,
                GuiMenuCommand::ClosePane,
                GuiMenuCommand::Quit,
            ]
        );

        let edit_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Edit)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(
            edit_commands,
            vec![
                GuiMenuCommand::Undo,
                GuiMenuCommand::Redo,
                GuiMenuCommand::Copy,
                GuiMenuCommand::Cut,
                GuiMenuCommand::Paste,
                GuiMenuCommand::SelectAll,
                GuiMenuCommand::FindNext,
                GuiMenuCommand::FindPrevious,
            ]
        );

        let go_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Go)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(
            go_commands,
            vec![
                GuiMenuCommand::GoToLine,
                GuiMenuCommand::GoDocumentStart,
                GuiMenuCommand::GoDocumentEnd,
            ]
        );

        let notes_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Notes)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(
            notes_commands,
            vec![
                GuiMenuCommand::OpenManagedNote,
                GuiMenuCommand::ListManagedNotes,
            ]
        );

        let tile_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Tile)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(
            tile_commands,
            vec![
                GuiMenuCommand::ToggleMinimize,
                GuiMenuCommand::ToggleMaximize,
                GuiMenuCommand::EqualizeTiles,
                GuiMenuCommand::MoveLeft,
                GuiMenuCommand::MoveRight,
                GuiMenuCommand::MoveUp,
                GuiMenuCommand::MoveDown,
            ]
        );

        let help_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Help)
            .into_iter()
            .map(|item| item.command)
            .collect();
        assert_eq!(help_commands, vec![GuiMenuCommand::OpenHelp]);
    }

    #[test]
    fn gui_help_menu_opens_builtin_help_document_tile() {
        let temp = TempArea::new("gui-help-menu");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));

        let active = state.workspace.active_tile();
        assert_eq!(active.document.path, temp.path(GUI_HELP_DOCUMENT_PATH));
        let help_text = active.document.buffer.to_text();
        assert!(help_text.contains("# kfnotepad help"));
        assert!(help_text.contains("Double-click a file to open it in a tile."));
        assert!(help_text.contains("Ctrl-R, View > Reader mode"));
        assert!(help_text.contains("Search is case-insensitive by default."));
        assert!(help_text.contains("Ctrl-Shift-T cycles the syntax highlighting theme"));
        assert!(help_text.contains("Reader speed is configured in Preferences"));
        assert!(help_text.contains("Notes > Open note creates or opens a managed Markdown note."));
        assert!(help_text.contains("Tile > Equalize tiles arranges open tiles into an even grid."));
        assert!(help_text.contains("Opening a file that is already open focuses or restores"));
        assert!(help_text.contains("Save uses the same atomic local-file adapter"));
        assert!(!active.document.buffer.is_dirty());
        assert!(state.status_message.contains("opened help"));

        let tile_count = state.workspace.tiles.len();
        let help_path = active.document.path.clone();
        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));
        assert_eq!(state.workspace.tiles.len(), tile_count);
        assert_eq!(state.workspace.active_tile().document.path, help_path);
    }

    #[test]
    fn gui_menu_styles_use_app_theme_palette() {
        let palette = gui_theme_palette(EditorThemeId::Nocturne);
        let panel_style = gui_menu_panel_style(palette);

        assert_eq!(
            panel_style.menu_border.radius.top_left,
            GUI_MENU_DROPDOWN_RADIUS
        );
        assert_eq!(panel_style.menu_border.color, palette.primary);
        assert_eq!(panel_style.menu_shadow.offset, Vector::new(0.0, 6.0));
        assert_eq!(panel_style.menu_shadow.blur_radius, 16.0);
        assert_eq!(
            panel_style.path_border.radius.top_left,
            GUI_MENU_ITEM_RADIUS
        );

        let active = gui_menu_item_button_style(palette, iced::widget::button::Status::Active);
        assert_eq!(active.text_color, palette.text);
        assert_eq!(active.border.width, 0.0);
        assert_eq!(active.border.radius.top_left, GUI_MENU_ITEM_RADIUS);

        let root = gui_menu_root_style(palette);
        assert_eq!(root.text_color, Some(palette.text));
        assert!(root.background.is_none());
        assert_eq!(GUI_MENU_ROOT_HORIZONTAL_PADDING, 3.0);
        assert_eq!(GUI_MENU_ROOT_VERTICAL_PADDING, 1.0);
        assert_eq!(GUI_MENU_ROOT_HEIGHT, 24.0);
        assert_eq!(GUI_MENU_BAR_SPACING, 1);
        assert_eq!(GUI_HEADER_ACTION_SPACING, 3);
        assert_eq!(GUI_HEADER_GROUP_SPACING, 6);
        assert_eq!(GUI_HEADER_SPLIT_SPACING, 3);
        assert_eq!(GUI_MENU_ITEM_PADDING, [3, 5]);
        assert_eq!(GUI_ICON_BUTTON_SIDE, 22.0);
        assert_eq!(GUI_TILE_CONTROL_BUTTON_SIDE, 24.0);
        assert_eq!(gui_icon_font(), Font::with_name(GUI_ICON_FONT_NAME));
        assert_eq!(GUI_ICON_LINE_HEIGHT, 1.0);
        assert_eq!(GUI_PANEL_PADDING_LEFT, 2.0);
        assert_eq!(GUI_PANEL_PADDING_RIGHT, 4.0);
        assert_eq!(GUI_PANEL_PADDING_VERTICAL, 6.0);
        assert_eq!(GUI_EDITOR_RENDER_LINE_BUDGET, 512);

        let hovered = gui_menu_item_button_style(palette, iced::widget::button::Status::Hovered);
        assert_eq!(hovered.text_color, palette.background);
        assert_eq!(hovered.border.width, 1.0);
        assert_eq!(hovered.border.color, palette.primary);

        let chrome = gui_chrome_button_style(palette, iced::widget::button::Status::Active);
        assert_eq!(chrome.text_color, palette.background);
        assert_eq!(chrome.border.width, 0.0);
        assert_eq!(chrome.border.radius.top_left, 4.0);
    }

    #[test]
    fn gui_native_editor_and_form_styles_do_not_add_hover_borders() {
        let palette = gui_theme_palette(EditorThemeId::Nocturne);

        let active = gui_native_editor_style(palette, text_editor::Status::Active, false);
        let hovered = gui_native_editor_style(palette, text_editor::Status::Hovered, false);
        let focused = gui_native_editor_style(
            palette,
            text_editor::Status::Focused { is_hovered: true },
            false,
        );
        let search = gui_native_editor_style(palette, text_editor::Status::Active, true);
        assert_eq!(hovered.border, active.border);
        assert_eq!(focused.border, active.border);
        assert_eq!(active.border.width, 0.0);
        assert_eq!(active.border.color, Color::TRANSPARENT);
        assert!(search.selection.a > active.selection.a);
        assert_eq!(search.selection.r, active.selection.r);
        assert_eq!(search.selection.g, active.selection.g);
        assert_eq!(search.selection.b, active.selection.b);

        let input_active = gui_text_input_style(palette, iced::widget::text_input::Status::Active);
        let input_hovered =
            gui_text_input_style(palette, iced::widget::text_input::Status::Hovered);
        assert_eq!(input_hovered.border, input_active.border);
        assert_eq!(input_active.value, palette.text);

        let checkbox_active = gui_checkbox_style(
            palette,
            iced::widget::checkbox::Status::Active { is_checked: true },
        );
        assert_eq!(checkbox_active.text_color, Some(palette.text));
        assert_eq!(checkbox_active.border.color, palette.primary);
    }

    #[test]
    fn gui_chrome_labels_trim_paths_and_keep_tooltip_sources() {
        let path = PathBuf::from("/tmp/kfnotepad/deep/example.md");

        assert_eq!(gui_file_name_label(&path), "example.md");
        assert_eq!(
            gui_tile_title_label(&path, true, "modified"),
            "active | example.md | modified"
        );
        assert_eq!(
            gui_tile_title_label(&path, true, "saved"),
            "active | example.md"
        );
        assert_eq!(gui_tile_title_label(&path, false, "saved"), "example.md");
        assert_eq!(
            gui_icon_label(ICON_FILES, LABEL_FILES),
            format!("{ICON_FILES} Files")
        );
        assert_eq!(gui_icon_only_label(ICON_FILES), ICON_FILES);
        assert!(!gui_icon_only_label(ICON_PREFERENCES).contains(LABEL_PREFERENCES));
        assert_eq!(gui_icon_only_label(ICON_NEW_TILE), ICON_NEW_TILE);
        assert!(!gui_icon_only_label(ICON_NEW_TILE).contains(LABEL_NEW_TILE));
        assert_eq!(gui_icon_only_label(ICON_SAVE), ICON_SAVE);
        assert!(!gui_icon_only_label(ICON_SAVE).contains(LABEL_SAVE));
        assert_eq!(gui_icon_only_label(ICON_THEME), ICON_THEME);
        assert!(!gui_icon_only_label(ICON_THEME).contains(LABEL_THEME));
        assert!(!gui_tile_title_label(&path, true, "modified").contains("/tmp"));
        assert!(gui_tile_title_controls_attached(true));
        assert!(gui_tile_title_controls_attached(false));

        let deep_path = PathBuf::from("/home/example/projects/kfnotepad/docs");
        assert_eq!(
            gui_sidebar_path_label(&path),
            "/tmp/kfnotepad/deep/example.md"
        );
        assert_eq!(gui_sidebar_path_label(&deep_path), ".../docs");
        assert!(gui_sidebar_path_label(&deep_path).len() <= GUI_PANEL_PATH_MAX_CHARS);
    }

    #[test]
    fn gui_tile_window_chrome_uses_compact_gapped_layout() {
        let palette = gui_theme_palette(EditorThemeId::Terminal);
        let active_body = gui_tile_body_style(palette, true);
        let inactive_body = gui_tile_body_style(palette, false);
        let active_title = gui_tile_title_style(palette, true);
        let grid = gui_pane_grid_style(palette);

        assert_eq!(GUI_PANE_GRID_SPACING, 5.0);
        assert_eq!(GUI_EDITOR_PADDING, 2);
        assert_eq!(GUI_TILE_BODY_PADDING, 2);
        assert_eq!(GUI_TILE_TITLE_PADDING, 3);
        assert_eq!(GUI_TILE_CONTROL_SPACING, 1);
        assert_eq!(GUI_PANEL_CONTROL_SPACING, 5);
        assert_eq!(GUI_PANEL_SECTION_SPACING, 6);
        assert_eq!(GUI_PANEL_TREE_TOP_PADDING, 4.0);
        assert_eq!(GUI_CHROME_PADDING, [2, 3]);
        let global_icon_side = GUI_ICON_BUTTON_SIDE;
        let tile_control_side = GUI_TILE_CONTROL_BUTTON_SIDE;
        assert!(global_icon_side < tile_control_side);
        assert_eq!(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 32);
        assert_eq!(GUI_LINE_NUMBER_SEPARATOR_WIDTH, 1.0);
        assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
        assert_eq!(active_body.border.radius.top_left, GUI_TILE_RADIUS);
        assert_eq!(active_body.border.width, 1.0);
        assert_eq!(inactive_body.border.width, 1.0);
        assert_eq!(active_body.border.color, palette.primary);
        assert_eq!(
            inactive_body.border.color,
            Color {
                a: 0.55,
                ..palette.primary
            }
        );
        assert_eq!(active_title.text_color, Some(palette.primary));
        assert_eq!(grid.hovered_region.border.width, 0.0);
        assert_eq!(grid.hovered_region.border.color, Color::TRANSPARENT);
        assert!(matches!(
            grid.hovered_region.background,
            Background::Color(Color::TRANSPARENT)
        ));
        assert_eq!(grid.hovered_split.width, 1.0);
        assert_eq!(grid.picked_split.color, palette.primary);
    }

    #[test]
    fn gui_editor_scrollbar_model_is_thin_and_proportional() {
        let model = gui_editor_scrollbar_model(100, 41, 20, 200.0);

        assert!(model.visible);
        assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
        assert_eq!(model.track_height, 200.0);
        assert_eq!(model.thumb_height, 40.0);
        assert_eq!(model.thumb_top, 80.0);
        assert_eq!(model.page_delta, 20);

        let hidden = gui_editor_scrollbar_model(5, 1, 20, 200.0);
        assert!(!hidden.visible);
        assert_eq!(hidden.thumb_height, 200.0);
    }

    #[test]
    fn gui_equalized_tile_layout_places_remainder_on_right() {
        let three = equalized_tile_layout_node(3).expect("three layout");
        let GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } = three
        else {
            panic!("expected three-tile split");
        };
        assert_eq!(axis, GuiLayoutAxis::Vertical);
        assert_eq!(ratio_per_mille, 500);
        assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 2 });
        let GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } = *first
        else {
            panic!("expected first column row split");
        };
        assert_eq!(axis, GuiLayoutAxis::Horizontal);
        assert_eq!(ratio_per_mille, 500);
        assert_eq!(*first, GuiLayoutNode::Leaf { ordinal: 0 });
        assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 1 });

        let five = equalized_tile_layout_node(5).expect("five layout");
        let GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } = five
        else {
            panic!("expected five-tile split");
        };
        assert_eq!(axis, GuiLayoutAxis::Vertical);
        assert_eq!(ratio_per_mille, 666);
        assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
        assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
    }

    #[test]
    fn gui_menu_tile_equalizes_visible_panes_into_grid() {
        let temp = TempArea::new("gui-equalize-menu");
        let paths = (1..=5)
            .map(|index| {
                let path = temp.path(&format!("{index}.txt"));
                fs::write(&path, format!("{index}\n")).expect("write tile");
                path
            })
            .collect::<Vec<_>>();
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: paths.clone(),
        });
        let active_path = state.workspace.active_tile().document.path.clone();

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
        );

        assert_eq!(state.status_message, "equalized 5 tiles");
        assert_eq!(state.workspace.active_tile().document.path, active_path);
        let Some(layout) = gui_layout_from_state(
            &state.panes,
            &state.workspace,
            state.browser_visible,
            state.browser_width,
        ) else {
            panic!("expected layout");
        };
        let GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } = layout.root
        else {
            panic!("expected equalized root split");
        };
        assert_eq!(axis, GuiLayoutAxis::Vertical);
        assert_eq!(ratio_per_mille, 666);
        assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
        assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
    }

    #[test]
    fn gui_equalize_preserves_wrapped_editor_viewport_and_gutter() {
        let temp = TempArea::new("gui-equalize-renderer-viewport");
        let paths = (1..=5)
            .map(|index| {
                let path = temp.path(&format!("{index}.txt"));
                let text = (1..=120)
                    .map(|line| {
                        format!(
                            "line {line} keeps enough words around to wrap in a narrow pane without changing the source line"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(&path, text).expect("write tile");
                path
            })
            .collect::<Vec<_>>();
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: paths,
        });
        state.settings.show_line_numbers = true;
        state.settings.wrap_lines = true;

        let _ = update(&mut state, Message::ScrollActiveEditorViewport(40));
        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
        );

        let surface = gui_editor_surface_model(
            state.settings,
            &state.workspace.active_tile().document,
            state.active_editor(),
            &state.syntax_highlighter,
            state.syntax_caches.get(&state.workspace.active_tile().id),
        );
        let line_numbers = surface.line_numbers.expect("visible line numbers");
        let visual_rows = gui_editor_read_only_visual_rows(
            &surface.viewport_slice.lines,
            surface.viewport_slice.first_line,
            surface.wrapping,
            34,
        );

        assert_eq!(surface.viewport_slice.first_line, 41);
        assert_eq!(line_numbers.gutter_start, 41);
        assert!(line_numbers.text.starts_with("41\n42\n43"));
        assert_eq!(visual_rows.first().map(|row| row.line.number), Some(41));
        assert!(visual_rows.iter().any(|row| !row.show_line_number));
        assert!(visual_rows.iter().all(|row| row.line.number >= 41));
        assert_eq!(state.status_message, "equalized 5 tiles");
    }

    #[test]
    fn gui_open_replaces_initial_blank_tile_but_not_dirty_or_real_tiles() {
        let temp = TempArea::new("gui-open-replace-blank");
        let file = temp.path("opened.txt");
        fs::write(&file, "opened\n").expect("write opened file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::OpenDialogSelected(Some(file.clone())));

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, file);
        assert_eq!(state.active_editor().text(), "opened\n");
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );

        let second = temp.path("second.txt");
        fs::write(&second, "second\n").expect("write second file");
        let _ = update(
            &mut state,
            Message::OpenDialogSelected(Some(second.clone())),
        );

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, second);
    }

    #[test]
    fn gui_open_focuses_existing_file_instead_of_duplicate_tile() {
        let temp = TempArea::new("gui-open-existing-file");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first file");
        fs::write(&second, "second\n").expect("write second file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
        );

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, second);

        let _ = update(&mut state, Message::OpenDialogSelected(Some(first.clone())));

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.workspace.active_tile().document.path, first);
        assert_eq!(state.active_editor().text(), "first\n");
        assert!(state.status_message.starts_with("already open: "));
    }

    #[test]
    fn gui_tile_title_controls_are_attached_for_hover_reveal_on_every_pane() {
        assert!(gui_tile_title_controls_attached(true));
        assert!(gui_tile_title_controls_attached(false));
    }

    #[test]
    fn gui_header_layout_splits_global_actions_on_narrow_windows() {
        assert_eq!(
            gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH),
            GuiHeaderLayoutMode::SingleRow
        );
        assert_eq!(
            gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH + 1.0),
            GuiHeaderLayoutMode::SingleRow
        );
        assert_eq!(
            gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH - 1.0),
            GuiHeaderLayoutMode::SplitActions
        );
    }

    #[test]
    fn gui_search_layout_splits_controls_on_narrow_windows() {
        assert_eq!(
            gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH),
            GuiSearchLayoutMode::SingleRow
        );
        assert_eq!(
            gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH + 1.0),
            GuiSearchLayoutMode::SingleRow
        );
        assert_eq!(
            gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH - 1.0),
            GuiSearchLayoutMode::SplitRows
        );
    }

    #[test]
    fn gui_core_actions_keep_descriptive_labels_and_access_paths() {
        let actions = gui_action_descriptors();
        assert!(actions.len() >= 14);

        for action in &actions {
            assert!(
                action.label.chars().count() > 1,
                "GUI action label must be descriptive: {}",
                action.label
            );
            assert!(
                action.shortcut.is_some() || action.menu_group.is_some(),
                "GUI action must have keyboard or menu access: {}",
                action.label
            );
        }

        let menu_labels = gui_menu_groups()
            .into_iter()
            .flat_map(gui_menu_items)
            .map(|item| item.label)
            .collect::<Vec<_>>();
        for action in actions.iter().filter(|action| action.menu_group.is_some()) {
            assert!(
                menu_labels.contains(&action.label),
                "menu-backed GUI action missing from menu: {}",
                action.label
            );
        }
    }

    #[test]
    fn gui_focus_order_keeps_global_controls_before_browser_search_and_tiles() {
        let focus_order = gui_focus_order_descriptors(true, false);
        let labels = focus_order
            .iter()
            .map(|step| step.label)
            .collect::<Vec<_>>();

        assert_eq!(
            &labels[..10],
            [
                "File",
                "Edit",
                "View",
                "Nav",
                "Notes",
                "Tile",
                "Help",
                LABEL_NEW_TILE,
                "Hide Files",
                LABEL_THEME,
            ]
        );

        assert!(
            labels
                .iter()
                .position(|label| *label == "File browser entries")
                < labels.iter().position(|label| *label == "Find field")
        );
        assert!(
            labels.iter().position(|label| *label == LABEL_DOCUMENT_END)
                < labels.iter().position(|label| *label == LABEL_MOVE_LEFT)
        );
        assert_eq!(labels.last(), Some(&"Active editor"));

        for step in &focus_order {
            assert!(
                !step.area.trim().is_empty() && !step.label.trim().is_empty(),
                "focus step must have visible context and label"
            );
        }

        for label in [
            "Hide Files",
            LABEL_NEW_TILE,
            LABEL_THEME,
            LABEL_SAVE,
            "Find field",
            LABEL_FIND_PREVIOUS,
            LABEL_FIND_NEXT,
            "Line field",
            LABEL_GO,
            LABEL_DOCUMENT_START,
            LABEL_DOCUMENT_END,
            LABEL_MOVE_LEFT,
            LABEL_MOVE_RIGHT,
            LABEL_MOVE_UP,
            LABEL_MOVE_DOWN,
            LABEL_MINIMIZE,
            LABEL_MAXIMIZE,
            LABEL_CLOSE_TILE,
        ] {
            let step = focus_order
                .iter()
                .find(|step| step.label == label && !matches!(step.area, "menu"))
                .expect("focus step should exist");
            assert!(
                step.keyboard.is_some(),
                "focus step should keep a keyboard equivalent: {label}"
            );
        }
    }

    #[test]
    fn gui_focus_order_reflects_browser_and_minimized_tile_visibility() {
        let browser_hidden = gui_focus_order_descriptors(false, false);
        let hidden_labels = browser_hidden
            .iter()
            .map(|step| step.label)
            .collect::<Vec<_>>();
        assert!(hidden_labels.contains(&"Show Files"));
        assert!(!hidden_labels.contains(&"File browser entries"));
        assert!(hidden_labels.contains(&"Active editor"));

        let minimized = gui_focus_order_descriptors(true, true);
        let minimized_labels = minimized.iter().map(|step| step.label).collect::<Vec<_>>();
        assert!(minimized_labels.contains(&LABEL_RESTORE));
        assert!(!minimized_labels.contains(&LABEL_MINIMIZE));
        assert!(!minimized_labels.contains(&"Active editor"));
    }

    #[test]
    fn gui_menu_file_and_view_commands_route_existing_actions() {
        let temp = TempArea::new("gui-menu-file-view");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let config = temp.path("config.toml");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("saved by menu\n");

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Save));

        assert_eq!(
            fs::read_to_string(&second).expect("read second"),
            "saved by menu\n"
        );

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ToggleBrowser),
        );
        assert!(!state.browser_visible);

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::CycleTheme));
        assert_eq!(state.settings.theme_id, EditorThemeId::Abyss);
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::ClosePane));
        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, first);
    }

    #[test]
    fn gui_file_menu_new_tile_routes_to_new_tile_creation() {
        let temp = TempArea::new("gui-menu-new-tile");
        let first = temp.path("first.txt");
        fs::write(&first, "first\n").expect("write first");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![first],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::NewTile));

        assert_eq!(state.workspace.tiles.len(), 2);
        assert_eq!(
            state.workspace.active_tile().document.path,
            temp.path("untitled.txt")
        );
        assert!(!temp.path("untitled.txt").exists());
    }

    #[test]
    fn gui_menu_edit_go_and_tile_commands_route_existing_actions() {
        let temp = TempArea::new("gui-menu-edit-go-tile");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "alpha\nbeta alpha\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second],
        });

        let _ = update(&mut state, Message::SearchQueryChanged("alpha".to_string()));
        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::FindNext));
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 5 }
        );

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::GoDocumentStart),
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 0 }
        );

        let second_pane = state.active_pane;
        let second_tile = state.panes.get(second_pane).expect("second pane").tile_id;
        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ToggleMinimize),
        );
        assert!(
            state
                .workspace
                .tile(second_tile)
                .expect("second tile")
                .minimized
        );
        assert_eq!(state.workspace.active_tile().document.path, first);
    }

    #[test]
    fn gui_menu_undo_redo_route_replacement_editor_history() {
        let temp = TempArea::new("gui-menu-undo-redo");
        let file = temp.path("edit.txt");
        fs::write(&file, "hello\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        let _ = update(
            &mut state,
            Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
        );
        assert_eq!(state.active_editor().text(), "Xhello\n");

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Undo));
        assert_eq!(state.active_editor().text(), "hello\n");
        assert_eq!(state.status_message, "undo");

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Redo));
        assert_eq!(state.active_editor().text(), "Xhello\n");
        assert_eq!(state.status_message, "redo");
    }

    #[test]
    fn gui_insert_key_toggles_overwrite_for_replacement_editor() {
        let temp = TempArea::new("gui-insert-overwrite");
        let file = temp.path("edit.txt");
        fs::write(&file, "abc\n").expect("write file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        let _ = update(&mut state, Message::ToggleReplacementOverwriteMode);
        assert!(state.replacement_overwrite_mode);
        let _ = update(
            &mut state,
            Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
        );
        assert_eq!(state.active_editor().text(), "Xbc\n");

        let _ = update(&mut state, Message::ToggleReplacementOverwriteMode);
        assert!(!state.replacement_overwrite_mode);
        let _ = update(
            &mut state,
            Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('Y')]),
        );
        assert_eq!(state.active_editor().text(), "XYbc\n");
    }

    #[test]
    fn gui_menu_clipboard_commands_route_editor_actions() {
        let temp = TempArea::new("gui-menu-clipboard");
        let file = temp.path("clip.txt");
        fs::write(&file, "alpha\n").expect("write file");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Copy));
        assert_eq!(state.status_message, "nothing selected");
        assert_eq!(
            fs::read_to_string(&file).expect("read file"),
            "alpha\n",
            "clipboard menu actions should not write files"
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SelectAll));
        assert_eq!(state.status_message, "selected all");
        assert_eq!(
            state.active_editor().selection().as_deref(),
            Some("alpha\n")
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Copy));
        assert_eq!(state.status_message, "copied selection");
        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "alpha\n"
        );

        let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Cut));
        assert_eq!(state.status_message, "cut selection");
        assert_eq!(state.active_editor().text(), "");
        assert_eq!(state.workspace.active_tile().document.buffer.to_text(), "");
        assert_eq!(
            state.workspace.active_tile().save_status(),
            GuiTileSaveStatus::Modified
        );
        assert_eq!(
            fs::read_to_string(&file).expect("read file after cut"),
            "alpha\n",
            "cut should not save implicitly"
        );

        let _ = update(
            &mut state,
            Message::ClipboardPasted(Some("omega".to_string())),
        );
        assert_eq!(state.status_message, "pasted clipboard");
        assert_eq!(state.active_editor().text(), "omega");
        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "omega"
        );

        let _ = update(&mut state, Message::ClipboardPasted(None));
        assert_eq!(state.status_message, "clipboard is empty");
        assert_eq!(state.active_editor().text(), "omega");
    }

    #[test]
    fn gui_close_only_clean_pane_resets_to_blank_tile() {
        let temp = TempArea::new("gui-close-only");
        let file = temp.path("only.txt");
        fs::write(&file, "only\n").expect("write only");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(
            state.workspace.active_tile().document.path,
            temp.path("untitled.txt")
        );
        assert_eq!(state.active_editor().text(), "");
        assert_eq!(
            fs::read_to_string(&file).expect("original file unchanged"),
            "only\n"
        );
        assert!(state.status_message.starts_with("new blank tile "));
    }

    #[test]
    fn gui_close_only_dirty_pane_requires_confirmation_before_blank_reset() {
        let temp = TempArea::new("gui-close-only-dirty");
        let file = temp.path("only.txt");
        fs::write(&file, "only\n").expect("write only");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file.clone()],
        });
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor = GuiEditorAdapter::from_text("dirty\n");

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, file);
        assert_eq!(
            state.status_message,
            "unsaved changes; close again to discard this tile"
        );

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(
            state.workspace.active_tile().document.path,
            temp.path("untitled.txt")
        );
        assert_eq!(state.active_editor().text(), "");
        assert_eq!(
            fs::read_to_string(&file).expect("original file unchanged"),
            "only\n"
        );
    }

    #[test]
    fn gui_minimize_active_pane_hides_tile_and_focuses_visible_fallback() {
        let temp = TempArea::new("gui-minimize-active");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        let second_pane = state.active_pane;
        let second_tile_id = state.panes.get(second_pane).expect("second pane").tile_id;
        state
            .panes
            .get_mut(second_pane)
            .expect("second pane")
            .editor = GuiEditorAdapter::from_text("dirty second\n");

        let _ = update(&mut state, Message::ToggleActiveMinimize);

        assert!(
            state
                .workspace
                .tile(second_tile_id)
                .expect("second tile")
                .minimized
        );
        assert_eq!(
            state
                .workspace
                .tile(second_tile_id)
                .expect("second tile")
                .document
                .buffer
                .to_text(),
            "dirty second\n"
        );
        assert_eq!(state.workspace.active_tile().document.path, first);
        assert_eq!(state.status_message, "minimized tile");
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.minimized_panes.len(), 1);
        assert_eq!(state.minimized_panes[0].tile_id, second_tile_id);

        let _ = update(&mut state, Message::RestoreMinimizedTile(second_tile_id));

        assert!(
            !state
                .workspace
                .tile(second_tile_id)
                .expect("second tile")
                .minimized
        );
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.active_editor().text(), "dirty second\n");
        assert_eq!(state.status_message, "restored tile");
        assert_eq!(state.panes.len(), 2);
        assert!(state.minimized_panes.is_empty());
    }

    #[test]
    fn gui_minimize_refuses_only_visible_tile() {
        let temp = TempArea::new("gui-minimize-only-visible");
        let file = temp.path("only.txt");
        fs::write(&file, "only\n").expect("write only");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![file],
        });
        let active_tile_id = state.workspace.active;

        let _ = update(&mut state, Message::ToggleActiveMinimize);

        assert!(
            !state
                .workspace
                .tile(active_tile_id)
                .expect("active tile")
                .minimized
        );
        assert_eq!(
            state.status_message,
            "cannot minimize the only visible tile"
        );
    }

    #[test]
    fn gui_close_last_visible_tile_promotes_minimized_tile() {
        let temp = TempArea::new("gui-close-last-visible-promotes-minimized");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        let second_tile_id = state
            .panes
            .get(state.active_pane)
            .expect("second pane")
            .tile_id;

        let _ = update(&mut state, Message::ToggleActiveMinimize);
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.minimized_panes.len(), 1);
        assert_eq!(state.workspace.active_tile().document.path, first);

        let _ = update(&mut state, Message::CloseActivePane);

        assert_eq!(state.workspace.tiles.len(), 1);
        assert_eq!(state.panes.len(), 1);
        assert!(state.minimized_panes.is_empty());
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert!(
            !state
                .workspace
                .tile(second_tile_id)
                .expect("second tile")
                .minimized
        );
        assert_eq!(state.active_editor().text(), "second\n");
        assert_eq!(state.status_message, format!("closed {}", first.display()));
    }

    #[test]
    fn gui_maximize_active_pane_toggles_without_changing_saved_geometry() {
        let temp = TempArea::new("gui-maximize-active");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        let active = state.active_pane;

        let _ = update(&mut state, Message::ToggleActiveMaximize);

        assert_eq!(state.panes.maximized(), Some(active));
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.status_message, "maximized tile");
        let pane_grid::Node::Split { axis, ratio, .. } = state.panes.layout() else {
            panic!("expected split layout to remain available");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical);
        assert_eq!(*ratio, 0.5);

        let _ = update(&mut state, Message::ToggleMaximizePane(active));

        assert_eq!(state.panes.maximized(), None);
        assert_eq!(state.status_message, "restored tile layout");
    }

    #[test]
    fn gui_maximized_focus_moves_to_clicked_or_opened_pane() {
        let temp = TempArea::new("gui-maximize-focus");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let third = temp.path("third.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::write(&third, "third\n").expect("write third");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        let first_pane = pane_for_path(&state, &first);

        let _ = update(&mut state, Message::ToggleActiveMaximize);
        assert_eq!(state.panes.maximized(), Some(state.active_pane));

        let _ = update(&mut state, Message::PaneClicked(first_pane));

        assert_eq!(state.active_pane, first_pane);
        assert_eq!(state.panes.maximized(), Some(first_pane));
        assert_eq!(state.workspace.active_tile().document.path, first);

        state.open_path_in_new_pane(third.clone());

        assert_eq!(state.panes.maximized(), Some(state.active_pane));
        assert_eq!(state.workspace.active_tile().document.path, third);
    }

    #[test]
    fn gui_menu_tile_can_maximize_and_restore_active_pane() {
        let temp = TempArea::new("gui-maximize-menu");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first, second],
        });
        let active = state.active_pane;

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ToggleMaximize),
        );

        assert_eq!(state.panes.maximized(), Some(active));
        assert_eq!(state.status_message, "maximized tile");

        let _ = update(
            &mut state,
            Message::MenuCommand(GuiMenuCommand::ToggleMaximize),
        );

        assert_eq!(state.panes.maximized(), None);
        assert_eq!(state.status_message, "restored tile layout");
    }

    #[test]
    fn gui_move_active_pane_swaps_with_adjacent_pane() {
        let temp = TempArea::new("gui-move-active");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        });
        let active = state.active_pane;
        let first_pane = pane_for_path(&state, &first);

        assert_eq!(
            state.panes.adjacent(active, pane_grid::Direction::Left),
            Some(first_pane)
        );

        let _ = update(
            &mut state,
            Message::MoveActivePane(pane_grid::Direction::Left),
        );

        assert_eq!(
            state.panes.adjacent(active, pane_grid::Direction::Right),
            Some(first_pane)
        );
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.status_message, "moved active tile");
    }

    #[test]
    fn gui_drag_drop_moves_pane_to_edge() {
        let temp = TempArea::new("gui-drag-edge");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first, second.clone()],
        });
        let active = state.active_pane;
        let before = pane_x(&state, active);
        assert!(before > 0.0);

        let _ = update(
            &mut state,
            Message::PaneDragged(pane_grid::DragEvent::Dropped {
                pane: active,
                target: pane_grid::Target::Edge(pane_grid::Edge::Left),
            }),
        );

        assert_eq!(pane_x(&state, active), 0.0);
        assert_eq!(state.workspace.active_tile().document.path, second);
        assert_eq!(state.status_message, "moved tile");
    }

    #[test]
    fn gui_resize_updates_pane_grid_split_ratio() {
        let temp = TempArea::new("gui-resize");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![first, second],
        });
        let split = *state.panes.layout().splits().next().expect("split");

        let _ = update(
            &mut state,
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio: 0.3 }),
        );

        let pane_grid::Node::Split { ratio, .. } = state.panes.layout() else {
            panic!("expected split layout");
        };
        assert_eq!(*ratio, 0.3);
    }

    #[test]
    fn gui_restores_valid_layout_without_storing_paths() {
        let temp = TempArea::new("gui-layout-restore");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
        fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = false\nroot = 0\nnode.0 = split horizontal 250 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\nminimized = 0\n",
        )
        .expect("write layout");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second],
            },
            temp.root.clone(),
            None,
            Some(layout_path),
            None,
            None,
        );

        assert!(!state.browser_visible);
        assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_DEFAULT);
        assert!(
            state
                .workspace
                .tile(GuiTileId(0))
                .expect("first tile")
                .minimized
        );
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.minimized_panes.len(), 1);
        assert_eq!(state.minimized_panes[0].tile_id, GuiTileId(0));
        assert!(matches!(state.panes.layout(), pane_grid::Node::Pane(_)));
        assert!(state.status_message.contains("restored GUI layout"));
    }

    #[test]
    fn gui_restores_and_clamps_persisted_browser_width() {
        let temp = TempArea::new("gui-layout-browser-width");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
        fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = true\nbrowser_width_px = 999\nroot = 0\nnode.0 = split vertical 500 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\nminimized =\n",
        )
        .expect("write layout");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first, second],
            },
            temp.root.clone(),
            None,
            Some(layout_path),
            None,
            None,
        );

        assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MAX);
        assert!(state.status_message.contains("restored GUI layout"));
    }

    #[test]
    fn gui_restores_nested_layout_for_three_panes() {
        let temp = TempArea::new("gui-layout-restore-nested");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let third = temp.path("third.txt");
        let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::write(&third, "third\n").expect("write third");
        fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
        fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = true\nroot = 0\nnode.0 = split horizontal 400 1 2\nnode.1 = leaf 0\nnode.2 = split vertical 700 3 4\nnode.3 = leaf 1\nnode.4 = leaf 2\nminimized =\n",
        )
        .expect("write layout");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone(), third.clone()],
            },
            temp.root.clone(),
            None,
            Some(layout_path),
            None,
            None,
        );

        let pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } = state.panes.layout()
        else {
            panic!("expected restored root split");
        };
        assert_eq!(*axis, pane_grid::Axis::Horizontal);
        assert_eq!(*ratio, 0.4);
        assert_eq!(node_path(&state, a), Some(first));
        let pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } = &**b
        else {
            panic!("expected nested split");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical);
        assert_eq!(*ratio, 0.7);
        assert_eq!(node_path(&state, a), Some(second));
        assert_eq!(node_path(&state, b), Some(third));
    }

    #[test]
    fn gui_ignores_invalid_layout_and_uses_default_launch_layout() {
        let temp = TempArea::new("gui-layout-invalid");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
        fs::write(&layout_path, "version = 99\nroot = 0\nnode.0 = leaf 0\n")
            .expect("write invalid layout");

        let state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second],
            },
            temp.root.clone(),
            None,
            Some(layout_path),
            None,
            None,
        );

        assert!(state.browser_visible);
        let pane_grid::Node::Split { axis, ratio, .. } = state.panes.layout() else {
            panic!("expected default split layout");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical);
        assert_eq!(*ratio, 0.5);
        assert!(!state.status_message.contains("restored GUI layout"));
        assert_eq!(pane_x(&state, pane_for_path(&state, &first)), 0.0);
    }

    #[test]
    fn gui_saves_geometry_only_layout_after_layout_changes() {
        let temp = TempArea::new("gui-layout-save-runtime");
        let first = temp.path("first.txt");
        let second = temp.path("second.txt");
        let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
        fs::write(&first, "first\n").expect("write first");
        fs::write(&second, "second\n").expect("write second");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![first.clone(), second.clone()],
            },
            temp.root.clone(),
            None,
            Some(layout_path.clone()),
            None,
            None,
        );
        let split = *state.panes.layout().splits().next().expect("split");

        let _ = update(
            &mut state,
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio: 0.3 }),
        );
        let _ = update(&mut state, Message::ToggleBrowser);
        let _ = update(&mut state, Message::BrowserWidthChanged(270.0));
        let _ = update(&mut state, Message::ToggleActiveMinimize);

        let text = fs::read_to_string(&layout_path).expect("read saved layout");
        let layout = parse_gui_layout(&text, 2).expect("parse saved layout");
        assert!(!layout.browser_visible);
        assert_eq!(layout.browser_width_px, Some(270));
        assert_eq!(layout.minimized_ordinals, vec![1]);
        assert!(text.contains("node.0 = split vertical 500"));
        assert!(text.contains("browser_width_px = 270"));
        assert!(!text.contains(first.to_string_lossy().as_ref()));
        assert!(!text.contains(second.to_string_lossy().as_ref()));
        assert!(!text.contains("first\n"));
        assert!(!text.contains("second\n"));
    }

    #[test]
    fn gui_theme_palettes_cover_existing_presets() {
        let pastel = gui_theme_palette(EditorThemeId::Paper);
        assert_eq!(pastel.background, color(245, 226, 244));
        assert_eq!(pastel.text, color(34, 24, 48));

        let terminal = gui_theme_palette(EditorThemeId::Terminal);
        assert_eq!(terminal.background, color(0, 18, 7));
        assert_eq!(terminal.primary, color(72, 255, 112));

        let abyss = gui_theme_palette(EditorThemeId::Abyss);
        assert_eq!(abyss.background, color(3, 7, 18));
        assert_eq!(abyss.danger, color(255, 64, 96));

        let terror = gui_theme_palette(EditorThemeId::Terror);
        assert_eq!(terror.background, color(24, 0, 30));
        assert_eq!(terror.primary, color(255, 42, 160));

        for theme_id in [
            EditorThemeId::Nocturne,
            EditorThemeId::Aurora,
            EditorThemeId::Paper,
            EditorThemeId::Terminal,
            EditorThemeId::Abyss,
            EditorThemeId::Terror,
        ] {
            assert_eq!(gui_theme(theme_id).palette(), gui_theme_palette(theme_id));
        }
    }

    #[test]
    fn gui_pastel_syntax_colors_are_darkened_for_readability() {
        let pale = syntect::highlighting::Color {
            r: 220,
            g: 226,
            b: 232,
            a: 255,
        };
        let normal = gui_color_from_syntect(pale, EditorThemeId::Nocturne);
        let pastel = gui_color_from_syntect(pale, EditorThemeId::Paper);

        assert_eq!(normal, Color::from_rgb8(213, 224, 246));
        assert!(pastel.r < normal.r);
        assert!(pastel.g < normal.g);
        assert!(pastel.b < normal.b);
        assert_eq!(pastel, Color::from_rgb8(80, 67, 91));
    }

    #[test]
    fn gui_syntax_theme_colors_keep_readable_contrast() {
        let samples = [
            (220, 226, 232),
            (190, 90, 120),
            (90, 170, 130),
            (120, 140, 230),
        ];

        for theme_id in [
            EditorThemeId::Nocturne,
            EditorThemeId::Aurora,
            EditorThemeId::Paper,
            EditorThemeId::Terminal,
            EditorThemeId::Abyss,
            EditorThemeId::Terror,
        ] {
            let background = gui_color_to_rgb(gui_theme_palette(theme_id).background);
            for (red, green, blue) in samples {
                let foreground = gui_syntax_rgb_for_theme(red, green, blue, theme_id);
                assert!(
                    gui_contrast_ratio(foreground, background) >= 4.5,
                    "{theme_id:?} foreground {foreground:?} lacks readable contrast on {background:?}"
                );
            }
        }
    }

    #[test]
    fn gui_syntax_theme_role_palettes_stay_varied() {
        let samples = [
            (220, 226, 232),
            (100, 110, 120),
            (220, 80, 120),
            (220, 130, 70),
            (210, 190, 70),
            (80, 190, 120),
            (70, 190, 210),
            (80, 120, 220),
            (170, 95, 230),
        ];

        for theme_id in [
            EditorThemeId::Nocturne,
            EditorThemeId::Aurora,
            EditorThemeId::Paper,
            EditorThemeId::Terminal,
            EditorThemeId::Abyss,
            EditorThemeId::Terror,
        ] {
            let colors = samples
                .into_iter()
                .map(|(red, green, blue)| gui_syntax_rgb_for_theme(red, green, blue, theme_id))
                .collect::<HashSet<_>>();

            assert!(
                colors.len() >= 7,
                "{theme_id:?} syntax palette collapsed into {colors:?}"
            );
        }
    }

    #[test]
    fn gui_highlighter_uses_shared_syntax_tokens_and_preset_theme_mapping() {
        let temp = TempArea::new("gui-syntax-highlight");
        let rust_path = temp.path("main.rs");
        let text_path = temp.path("note.txt");
        fs::write(&rust_path, "fn main() {}\n").expect("write rust");
        fs::write(&text_path, "plain\n").expect("write text");
        let state = KfnotepadGui::new(GuiLaunch {
            requested_paths: vec![rust_path.clone(), text_path.clone()],
        });
        let rust_tile = state
            .workspace
            .tiles
            .iter()
            .find(|tile| tile.document.path == rust_path)
            .expect("rust tile");
        let text_tile = state
            .workspace
            .tiles
            .iter()
            .find(|tile| tile.document.path == text_path)
            .expect("text tile");

        assert_eq!(
            state
                .syntax_highlighter
                .syntax_token_for_document(&rust_tile.document),
            "rs"
        );
        assert_eq!(
            state
                .syntax_highlighter
                .syntax_token_for_document(&text_tile.document),
            "txt"
        );
        assert_eq!(
            gui_highlighter_theme(EditorThemeId::Paper),
            highlighter::Theme::InspiredGitHub
        );
        assert_eq!(
            gui_highlighter_theme(EditorThemeId::Terror),
            highlighter::Theme::Base16Eighties
        );
    }

    #[test]
    fn gui_theme_cycle_updates_status_and_persists_existing_config_format() {
        let temp = TempArea::new("gui-theme-cycle");
        let config = temp.path("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::CycleTheme);

        assert_eq!(state.settings.theme_id, EditorThemeId::Abyss);
        assert_eq!(state.status_message, "theme: abyss");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
    }

    #[test]
    fn gui_syntax_theme_cycles_separately_from_app_theme_and_persists() {
        let temp = TempArea::new("gui-syntax-theme-cycle");
        let config = temp.path("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            syntax_theme_id: EditorThemeId::Nocturne,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::CycleSyntaxTheme);

        assert_eq!(state.settings.theme_id, EditorThemeId::Terminal);
        assert_eq!(state.settings.syntax_theme_id, EditorThemeId::Aurora);
        assert_eq!(state.status_message, "syntax theme: aurora");
        let saved = fs::read_to_string(&config).expect("read config");
        assert!(saved.contains("theme = \"terminal\"\n"));
        assert!(saved.contains("syntax_theme = \"aurora\"\n"));
    }

    #[test]
    fn gui_search_defaults_case_insensitive_and_can_toggle_sensitive() {
        let temp = TempArea::new("gui-search-case");
        let config = temp.path("config.toml");
        let file_path = temp.path("note.txt");
        fs::write(&file_path, "Heading\nAlpha only\n").expect("write note");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![file_path],
            },
            temp.root.clone(),
            Some(config.clone()),
            None,
            None,
            None,
        );
        state.search_query = "alpha".to_string();

        let _ = update(&mut state, Message::SearchNext);
        assert_eq!(state.status_message, "found next: alpha");
        assert_eq!(state.workspace.active_tile().state.cursor.row, 1);
        assert_eq!(state.workspace.active_tile().state.cursor.column, 0);

        let _ = update(&mut state, Message::SearchCaseSensitiveChanged(true));
        assert!(state.settings.search_case_sensitive);
        let _ = update(&mut state, Message::SearchNext);
        assert_eq!(state.status_message, "no match: alpha");
        assert!(fs::read_to_string(&config)
            .expect("read config")
            .contains("search_case_sensitive = true\n"));
    }

    #[test]
    fn gui_reader_mode_ticks_scroll_active_document_and_persists_speed() {
        let temp = TempArea::new("gui-reader-mode");
        let config = temp.path("config.toml");
        let file_path = temp.path("long.txt");
        fs::write(
            &file_path,
            (1..=80)
                .map(|line| format!("line {line}\n"))
                .collect::<String>(),
        )
        .expect("write long note");
        let mut state = KfnotepadGui::new_with_paths(
            GuiLaunch {
                requested_paths: vec![file_path],
            },
            temp.root.clone(),
            Some(config.clone()),
            None,
            None,
            None,
        );

        let _ = update(&mut state, Message::ReaderSpeedChanged(120));
        let _ = update(&mut state, Message::ReaderModeChanged(true));
        let before = state
            .panes
            .get(state.active_pane)
            .expect("active pane")
            .editor
            .viewport
            .first_line;
        let _ = update(&mut state, Message::ReaderScrollTick);
        let after = state
            .panes
            .get(state.active_pane)
            .expect("active pane")
            .editor
            .viewport
            .first_line;

        assert!(after > before);
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 0 }
        );
        assert_eq!(
            state
                .panes
                .get(state.active_pane)
                .expect("active pane")
                .editor
                .document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );
        assert!(state.settings.gui_reader_mode_enabled);
        assert_eq!(state.settings.gui_reader_lines_per_minute, 120);
        let saved = fs::read_to_string(&config).expect("read config");
        assert!(saved.contains("gui_reader_mode_enabled = true\n"));
        assert!(saved.contains("gui_reader_lines_per_minute = 120\n"));
    }

    #[test]
    fn gui_preferences_panel_toggles_line_numbers_and_wrap_in_config() {
        let temp = TempArea::new("gui-preferences-toggle");
        let config = temp.path("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

        assert!(!state.settings.show_line_numbers);
        assert_eq!(state.status_message, "line numbers: off");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

        let _ = update(&mut state, Message::WrapLinesChanged(true));

        assert!(state.settings.wrap_lines);
        assert_eq!(state.status_message, "wrap text: on");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
    }

    #[test]
    fn gui_preferences_panel_cycles_font_family_and_size_in_config() {
        let temp = TempArea::new("gui-preferences-font");
        let config = temp.path("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::CycleGuiFontFamily);

        assert_eq!(state.settings.gui_font_family, GuiFontFamily::SansSerif);
        assert_eq!(state.status_message, "font: Sans serif");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

        let _ = update(&mut state, Message::GuiFontSizeChanged(20));

        assert_eq!(state.settings.gui_font_size, 20);
        assert_eq!(
            state.settings.gui_ui_font_size,
            kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
        );
        assert_eq!(state.status_message, "editor font size: 20");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 14\n"
        );

        let _ = update(&mut state, Message::GuiUiFontSizeChanged(18));

        assert_eq!(state.settings.gui_font_size, 20);
        assert_eq!(state.settings.gui_ui_font_size, 18);
        assert_eq!(state.status_message, "ui font size: 18");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 18\n"
        );
    }

    #[test]
    fn gui_preferences_panel_rejects_out_of_range_font_size() {
        let temp = TempArea::new("gui-preferences-font-size-bounds");
        let config = temp.path("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(
            &mut state,
            Message::GuiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
        );

        assert_eq!(state.settings.gui_font_size, DEFAULT_GUI_FONT_SIZE);
        assert_eq!(state.status_message, "editor font size must be 10-32");
        assert!(!config.exists());

        let _ = update(
            &mut state,
            Message::GuiUiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
        );

        assert_eq!(
            state.settings.gui_ui_font_size,
            kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
        );
        assert_eq!(state.status_message, "ui font size must be 10-32");
        assert!(!config.exists());
    }

    #[test]
    fn gui_preferences_panel_toggle_rolls_back_on_config_save_failure() {
        let temp = TempArea::new("gui-preferences-toggle-failure");
        let blocked_parent = temp.path("blocked");
        fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(blocked_parent.join("config.toml"));
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

        assert!(state.settings.show_line_numbers);
        assert!(!state.settings.wrap_lines);
        assert!(state.status_message.starts_with("settings save failed: "));
        assert_eq!(
            fs::read_to_string(&blocked_parent).expect("read blocked parent"),
            "not a directory\n"
        );
    }

    #[test]
    fn gui_preferences_panel_font_change_rolls_back_on_config_save_failure() {
        let temp = TempArea::new("gui-preferences-font-failure");
        let blocked_parent = temp.path("blocked");
        fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(blocked_parent.join("config.toml"));
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::CycleGuiFontFamily);

        assert_eq!(state.settings.gui_font_family, GuiFontFamily::Monospace);
        assert!(state.status_message.starts_with("settings save failed: "));
        assert_eq!(
            fs::read_to_string(&blocked_parent).expect("read blocked parent"),
            "not a directory\n"
        );
    }

    #[test]
    fn gui_editor_font_maps_font_presets() {
        assert_eq!(gui_editor_font(GuiFontFamily::Monospace), Font::MONOSPACE);
        assert_eq!(gui_editor_font(GuiFontFamily::SansSerif), Font::DEFAULT);
        assert_eq!(
            gui_editor_font(GuiFontFamily::Serif).family,
            iced::font::Family::Serif
        );
        assert_eq!(
            gui_editor_font(GuiFontFamily::JetBrainsMono),
            Font::with_name("JetBrains Mono")
        );
        assert_eq!(
            gui_editor_font(GuiFontFamily::FiraCode),
            Font::with_name("Fira Code")
        );
    }

    #[test]
    fn gui_ui_font_size_helpers_use_separate_chrome_setting() {
        let settings = EditorSettings {
            gui_font_size: 30,
            gui_ui_font_size: 18,
            ..EditorSettings::default()
        };

        assert_eq!(
            EditorSettings::default().gui_ui_font_size,
            kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
        );
        assert_eq!(gui_ui_text_size(settings), 18);
        assert_eq!(gui_ui_small_text_size(settings), 16);
        assert_eq!(gui_ui_heading_text_size(settings), 22);
        assert_eq!(gui_ui_icon_text_size(settings), 19);
        assert_eq!(gui_ui_tooltip_text_size(settings), 16);
    }

    #[test]
    fn gui_file_tree_rows_use_gui_ui_font_size() {
        let temp = TempArea::new("gui-file-tree-ui-size");
        fs::create_dir(temp.path("src")).expect("create src");
        fs::write(temp.path("README.md"), "readme\n").expect("write readme");
        let root = temp.root.canonicalize().expect("canonical root");
        let mut expanded = HashSet::new();
        expanded.insert(root.clone());
        let settings = EditorSettings {
            gui_font_size: 30,
            gui_ui_font_size: 19,
            ..EditorSettings::default()
        };

        let rows = gui_file_tree_rows(&root, &expanded, None);

        assert_eq!(gui_file_tree_text_size(settings), 19);
        assert_eq!(gui_file_tree_icon_size(settings), 20);
        assert!(rows.iter().any(|row| row.path == root && row.expanded));
        assert!(rows
            .iter()
            .any(|row| row.label == "src" && row.kind == FileSidebarEntryKind::Directory));
        assert!(rows
            .iter()
            .any(|row| row.label == "README.md" && row.kind == FileSidebarEntryKind::File));
    }

    #[test]
    fn gui_file_tree_rows_mark_exact_selected_nested_path() {
        let temp = TempArea::new("gui-file-tree-selected-nested");
        let src = temp.path("src");
        fs::create_dir(&src).expect("create src");
        let nested = src.join("lib.rs");
        fs::write(&nested, "pub fn demo() {}\n").expect("write nested file");
        let root = temp.root.canonicalize().expect("canonical root");
        let src = src.canonicalize().expect("canonical src");
        let nested = nested.canonicalize().expect("canonical nested");
        let mut expanded = HashSet::new();
        expanded.insert(root.clone());
        expanded.insert(src.clone());

        let rows = gui_file_tree_rows(&root, &expanded, Some(nested.as_path()));

        assert!(rows.iter().any(|row| row.path == nested && row.selected));
        assert!(rows.iter().any(|row| row.path == src && !row.selected));
    }

    #[test]
    fn gui_file_tree_selected_rows_use_selection_foreground() {
        let palette = gui_theme_palette(EditorThemeId::Abyss);

        assert_eq!(
            gui_file_tree_row_text_color(palette, true, false),
            palette.background
        );
        assert_eq!(
            gui_file_tree_row_text_color(palette, false, false),
            palette.text
        );
        assert_ne!(
            gui_file_tree_row_text_color(palette, true, false),
            palette.text
        );
    }

    #[test]
    fn gui_primary_icons_come_from_nerd_font_symbol_constants() {
        assert_eq!(ICON_VIEW_MENU, nf::cod::COD_EYE);
        assert_eq!(ICON_NEW_TILE, nf::fa::FA_PLUS);
        assert_eq!(ICON_SAVE, nf::cod::COD_SAVE);
        assert_eq!(ICON_FILES, nf::fa::FA_FOLDER);
        assert_eq!(ICON_WORKSPACES, nf::cod::COD_MULTIPLE_WINDOWS);
        assert_eq!(ICON_PREFERENCES, nf::cod::COD_SETTINGS_GEAR);
        assert_eq!(ICON_THEME, nf::cod::COD_SYMBOL_COLOR);
        assert_eq!(ICON_REFRESH, nf::cod::COD_REFRESH);
        assert_eq!(ICON_CREATE_FILE, nf::cod::COD_NEW_FILE);
        assert_eq!(ICON_PARENT_DIR, nf::cod::COD_ARROW_UP);
        assert_eq!(ICON_FIND_PREVIOUS, nf::cod::COD_ARROW_LEFT);
        assert_eq!(ICON_FIND_NEXT, nf::cod::COD_ARROW_RIGHT);
        assert_eq!(ICON_GO_TO_LINE, nf::cod::COD_DEBUG_LINE_BY_LINE);
        assert_eq!(ICON_DOCUMENT_START, nf::oct::OCT_HOME);
        assert_eq!(ICON_DOCUMENT_END, nf::oct::OCT_MOVE_TO_END);
        assert_eq!(ICON_MOVE_LEFT, nf::cod::COD_ARROW_SMALL_LEFT);
        assert_eq!(ICON_MOVE_RIGHT, nf::cod::COD_ARROW_SMALL_RIGHT);
        assert_eq!(ICON_MOVE_UP, nf::cod::COD_ARROW_SMALL_UP);
        assert_eq!(ICON_MOVE_DOWN, nf::cod::COD_ARROW_SMALL_DOWN);
        assert_eq!(ICON_MINIMIZE, nf::fa::FA_WINDOW_MINIMIZE);
        assert_eq!(ICON_RESTORE, nf::fa::FA_WINDOW_RESTORE);
        assert_eq!(ICON_MAXIMIZE, nf::fa::FA_WINDOW_MAXIMIZE);
        assert_eq!(ICON_CLOSE, nf::cod::COD_CHROME_CLOSE);
        assert_eq!(ICON_DELETE, nf::fa::FA_TRASH);
    }

    #[test]
    fn gui_file_tree_icons_use_nerd_font_symbol_constants() {
        let cases = [
            (IconRole::FolderClosed, nf::cod::COD_FOLDER),
            (IconRole::FolderOpen, nf::cod::COD_FOLDER_OPENED),
            (IconRole::File, nf::cod::COD_FILE),
            (IconRole::Error, nf::cod::COD_ERROR),
            (IconRole::CaretRight, nf::oct::OCT_CHEVRON_RIGHT),
            (IconRole::CaretDown, nf::oct::OCT_CHEVRON_DOWN),
        ];

        for (role, glyph) in cases {
            let spec = gui_tree_icon_spec(role);
            assert_eq!(spec.glyph.as_ref(), glyph);
            assert_eq!(spec.size, Some(13.0));
            assert_eq!(spec.font, None);
        }
    }

    #[test]
    fn gui_editor_rendering_helpers_reflect_line_number_and_wrap_preferences() {
        assert_eq!(gui_editor_wrapping(false), Wrapping::None);
        assert_eq!(gui_editor_wrapping(true), Wrapping::WordOrGlyph);
        assert_eq!(
            gui_editor_effective_wrapping(true, false),
            Wrapping::WordOrGlyph
        );
        assert_eq!(
            gui_editor_effective_wrapping(true, true),
            Wrapping::WordOrGlyph
        );
        assert_eq!(gui_editor_effective_wrapping(false, true), Wrapping::None);
        assert_eq!(gui_line_number_gutter_text(1, 0, 3), "1");
        assert_eq!(gui_line_number_gutter_text(3, 10, 4), "3\n4\n5\n6");
        assert_eq!(gui_line_number_gutter_text(99, 10, 4), "10");
        assert!(gui_line_number_gutter_width(100, 16) > gui_line_number_gutter_width(9, 16));
        assert!(gui_line_number_gutter_width(236, 16) < 40.0);
        assert!(gui_line_number_gutter_width(236, 20) > gui_line_number_gutter_width(236, 16));
        assert_eq!(gui_left_panel_width(false, 260.0), 0.0);
        assert_eq!(gui_left_panel_width(true, 260.0), 260.0);
        assert_eq!(gui_left_panel_width(true, 999.0), GUI_BROWSER_WIDTH_MAX);
    }

    #[test]
    fn gui_editor_surface_model_captures_backend_replacement_inputs() {
        let document = TextDocument {
            path: PathBuf::from("surface.rs"),
            buffer: TextBuffer::from_text("fn main() {}\nsecond\n"),
        };
        let mut adapter = GuiEditorAdapter::from_text("fn main() {}\nsecond\n");
        adapter.move_to(DocumentCursor { row: 1, column: 0 });
        let settings = EditorSettings {
            show_line_numbers: true,
            wrap_lines: true,
            gui_font_family: GuiFontFamily::FiraCode,
            gui_font_size: 18,
            theme_id: EditorThemeId::Terror,
            ..EditorSettings::default()
        };
        let highlighter = SyntaxHighlighter::default();

        let cache = gui_test_syntax_cache_for_document(&highlighter, &document, 8);
        let surface =
            gui_editor_surface_model(settings, &document, &adapter, &highlighter, Some(&cache));

        assert_eq!(surface.content.text(), "fn main() {}\nsecond\n");
        assert_eq!(surface.editor_size, 18);
        assert_eq!(surface.wrapping, Wrapping::WordOrGlyph);
        assert_eq!(surface.syntax_token, "rs");
        assert_eq!(
            surface.line_numbers,
            Some(GuiEditorLineNumberSnapshot {
                line_count: 3,
                gutter_start: 1,
                text: "1\n2\n3".to_string(),
                width: gui_line_number_gutter_width(3, 18),
            })
        );
        let mut viewport_without_syntax = surface.viewport_slice.clone();
        assert!(viewport_without_syntax.lines[0].syntax_segments.is_some());
        for line in &mut viewport_without_syntax.lines {
            line.syntax_segments = None;
        }
        assert_eq!(
            viewport_without_syntax,
            GuiEditorViewportSlice {
                line_count: 3,
                first_line: 1,
                lines: vec![
                    GuiEditorViewportLine {
                        number: 1,
                        text: "fn main() {}".to_string(),
                        cursor_column: None,
                        selection: None,
                        syntax_segments: None,
                    },
                    GuiEditorViewportLine {
                        number: 2,
                        text: "second".to_string(),
                        cursor_column: Some(0),
                        selection: None,
                        syntax_segments: None,
                    },
                    GuiEditorViewportLine {
                        number: 3,
                        text: String::new(),
                        cursor_column: None,
                        selection: None,
                        syntax_segments: None,
                    },
                ],
            }
        );

        let hidden_numbers = gui_editor_surface_model(
            EditorSettings {
                show_line_numbers: false,
                ..settings
            },
            &document,
            &adapter,
            &highlighter,
            Some(&cache),
        );
        assert_eq!(hidden_numbers.line_numbers, None);
        assert_eq!(hidden_numbers.wrapping, Wrapping::WordOrGlyph);
    }

    #[test]
    fn gui_editor_surface_model_renders_beyond_stale_logical_viewport_height() {
        let text = numbered_lines(100);
        let document = TextDocument {
            path: PathBuf::from("surface-long.txt"),
            buffer: TextBuffer::from_text(&text),
        };
        let mut adapter = GuiEditorAdapter::from_text(&text);
        adapter.apply(GuiEditorCommand::ScrollViewportLines(30));
        let highlighter = SyntaxHighlighter::default();

        let surface = gui_editor_surface_model(
            EditorSettings::default(),
            &document,
            &adapter,
            &highlighter,
            None,
        );

        assert_eq!(
            adapter.viewport.visible_lines,
            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
        );
        assert_eq!(surface.viewport_slice.first_line, 31);
        assert_eq!(
            surface.viewport_slice.lines.first().map(|line| line.number),
            Some(31)
        );
        assert_eq!(
            surface.viewport_slice.lines.last().map(|line| line.number),
            Some(100)
        );
        assert_eq!(surface.viewport_slice.lines.len(), 70);
        assert_eq!(
            surface
                .line_numbers
                .as_ref()
                .map(|numbers| numbers.text.clone()),
            Some(gui_line_number_gutter_text(
                31,
                100,
                GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
            ))
        );
    }

    #[test]
    fn gui_editor_surface_model_bounds_large_document_source_slice() {
        let text = numbered_lines(2_000);
        let document = TextDocument {
            path: PathBuf::from("surface-large.txt"),
            buffer: TextBuffer::from_text(&text),
        };
        let mut adapter = GuiEditorAdapter::from_text(&text);
        adapter.apply(GuiEditorCommand::ScrollViewportLines(1_200));
        let highlighter = SyntaxHighlighter::default();

        let surface = gui_editor_surface_model(
            EditorSettings::default(),
            &document,
            &adapter,
            &highlighter,
            None,
        );

        assert_eq!(surface.viewport_slice.first_line, 1_201);
        assert_eq!(
            surface.viewport_slice.lines.len(),
            GUI_EDITOR_RENDER_LINE_BUDGET
        );
        assert_eq!(
            surface.viewport_slice.lines.last().map(|line| line.number),
            Some(1_200 + GUI_EDITOR_RENDER_LINE_BUDGET)
        );
    }

    #[test]
    fn gui_syntax_cache_extends_to_visible_large_document_scroll() {
        let temp = TempArea::new("gui-syntax-cache-scroll");
        let path = temp.path("large.rs");
        let text = (0..2_000)
            .map(|index| format!("fn function_{index}() -> usize {{ {index} }}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, text).expect("write large rust file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![path],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        let initial_until = state
            .syntax_caches
            .get(&tile_id)
            .map(|cache| cache.highlighted_until)
            .expect("initial syntax cache");

        assert_eq!(initial_until, GUI_EDITOR_RENDER_LINE_BUDGET);
        assert!(
            state
                .syntax_caches
                .get(&tile_id)
                .and_then(|cache| cache.lines.first())
                .and_then(|line| line.as_ref())
                .is_some(),
            "Rust file should keep cached syntax segments"
        );

        let _ = update(&mut state, Message::ScrollActiveEditorViewport(80));

        let cache = state
            .syntax_caches
            .get(&tile_id)
            .expect("extended syntax cache");
        assert_eq!(cache.highlighted_until, initial_until + 80);
        assert_eq!(cache.lines.len(), initial_until + 80);
    }

    #[test]
    fn gui_syntax_cache_scrolls_real_large_source_incrementally() {
        let source_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/kfnotepad-gui.rs");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![source_path],
            },
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        );
        let tile_id = state.workspace.active_tile().id;
        let line_count = state.workspace.active_tile().document.buffer.line_count();
        let initial_until = state
            .syntax_caches
            .get(&tile_id)
            .map(|cache| cache.highlighted_until)
            .expect("initial syntax cache");
        let started = Instant::now();

        for _ in 0..50 {
            let _ = update(&mut state, Message::ScrollActiveEditorViewport(20));
        }

        let elapsed = started.elapsed();
        let cache = state
            .syntax_caches
            .get(&tile_id)
            .expect("extended syntax cache");
        let expected_until = (initial_until + 1_000).min(line_count);
        assert_eq!(cache.highlighted_until, expected_until);
        assert!(cache.highlighted_until < line_count);
        eprintln!(
            "large source incremental scroll: {} cached lines of {line_count} in {:?}",
            cache.highlighted_until, elapsed
        );
    }

    #[test]
    fn gui_syntax_cache_rebuilds_after_replacement_edit() {
        let temp = TempArea::new("gui-syntax-cache-edit");
        let path = temp.path("large.rs");
        let text = (0..700)
            .map(|index| format!("fn function_{index}() -> usize {{ {index} }}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, text).expect("write rust file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![path],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;
        let original_line_count = state.workspace.active_tile().document.buffer.line_count();

        let _ = update(
            &mut state,
            Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertNewline]),
        );

        let cache = state
            .syntax_caches
            .get(&tile_id)
            .expect("rebuilt syntax cache");
        assert_eq!(cache.line_count, original_line_count + 1);
        assert_eq!(cache.highlighted_until, GUI_EDITOR_RENDER_LINE_BUDGET);
    }

    #[test]
    fn gui_editor_adapter_exposes_parity_boundary_without_changing_backend() {
        let mut adapter = GuiEditorAdapter::from_text("one\ntwo\nthree\n");

        assert_eq!(adapter.text(), "one\ntwo\nthree\n");
        assert_eq!(adapter.line_count(), 4);
        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );

        adapter.apply(GuiEditorCommand::MoveTo(DocumentCursor {
            row: 1,
            column: 2,
        }));
        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 1, column: 2 }
        );

        let render = adapter.render_state(3, 16);
        assert_eq!(render.content.text(), "one\ntwo\nthree\n");
        assert_eq!(
            render.line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 4,
                gutter_start: 1,
                text: "1\n2\n3".to_string(),
                width: gui_line_number_gutter_width(4, 16),
            }
        );

        adapter.move_to(DocumentCursor { row: 1, column: 0 });
        adapter.select_right_chars(3);
        assert_eq!(adapter.selection().as_deref(), Some("two"));

        adapter.apply(GuiEditorCommand::SelectAll);
        assert_eq!(adapter.selection().as_deref(), Some("one\ntwo\nthree\n"));

        adapter.apply(GuiEditorCommand::Paste("alpha".to_string()));
        assert_eq!(adapter.text(), "alpha");
        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 0, column: 5 }
        );

        adapter.apply(GuiEditorCommand::MoveTo(DocumentCursor {
            row: 0,
            column: 0,
        }));
        adapter.apply(GuiEditorCommand::SelectRightChars(1));
        adapter.apply(GuiEditorCommand::Delete);
        assert_eq!(adapter.text(), "lpha");
    }

    #[test]
    fn gui_editor_adapter_viewport_keeps_cursor_visible_for_gutter() {
        let mut adapter = GuiEditorAdapter::from_text("one\ntwo\nthree\nfour\nfive");

        assert_eq!(
            adapter.render_state(3, 16).line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 5,
                gutter_start: 1,
                text: "1\n2\n3".to_string(),
                width: gui_line_number_gutter_width(5, 16),
            }
        );

        for _ in 0..4 {
            adapter.apply(GuiEditorCommand::IcedAction(text_editor::Action::Move(
                text_editor::Motion::Down,
            )));
        }
        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 4, column: 0 }
        );
        assert_eq!(
            adapter.render_state(3, 16).line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 5,
                gutter_start: 3,
                text: "3\n4\n5".to_string(),
                width: gui_line_number_gutter_width(5, 16),
            }
        );

        for _ in 0..3 {
            adapter.apply(GuiEditorCommand::IcedAction(text_editor::Action::Move(
                text_editor::Motion::Up,
            )));
        }
        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 1, column: 0 }
        );
        assert_eq!(
            adapter.render_state(3, 16).line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 5,
                gutter_start: 1,
                text: "1\n2\n3".to_string(),
                width: gui_line_number_gutter_width(5, 16),
            }
        );
    }

    #[test]
    fn gui_editor_adapter_scrolls_viewport_and_clamps_cursor_to_visible_lines() {
        let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

        adapter.apply(GuiEditorCommand::ScrollViewportLines(2));

        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 2, column: 0 }
        );
        assert_eq!(
            adapter
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 3,
                text: gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );

        adapter.apply(GuiEditorCommand::ScrollViewportLines(99));

        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 68, column: 0 }
        );
        assert_eq!(
            adapter
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 69,
                text: gui_line_number_gutter_text(69, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );

        adapter.apply(GuiEditorCommand::ScrollViewportLines(-99));

        assert_eq!(
            adapter.document_cursor(),
            DocumentCursor { row: 31, column: 0 }
        );
        assert_eq!(
            adapter
                .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
                .line_numbers,
            GuiEditorLineNumberSnapshot {
                line_count: 100,
                gutter_start: 1,
                text: gui_line_number_gutter_text(1, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
                width: gui_line_number_gutter_width(100, 16),
            }
        );
    }

    #[test]
    fn gui_editor_viewport_slice_uses_same_viewport_as_gutter() {
        let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

        adapter.apply(GuiEditorCommand::ScrollViewportLines(2));
        let render = adapter.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16);

        assert_eq!(
            render.line_numbers.gutter_start,
            render.viewport_slice.first_line
        );
        assert_eq!(render.viewport_slice.line_count, 100);
        assert_eq!(
            render.viewport_slice.lines.first(),
            Some(&GuiEditorViewportLine {
                number: 3,
                text: "3".to_string(),
                cursor_column: Some(0),
                selection: None,
                syntax_segments: None,
            })
        );
        assert_eq!(
            render.viewport_slice.lines.last(),
            Some(&GuiEditorViewportLine {
                number: 34,
                text: "34".to_string(),
                cursor_column: None,
                selection: None,
                syntax_segments: None,
            })
        );
        assert_eq!(
            render.viewport_slice.lines.len(),
            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
        );
    }

    #[test]
    fn gui_editor_viewport_slice_preserves_trailing_blank_line() {
        let adapter = GuiEditorAdapter::from_text("one\ntwo\n");
        let render = adapter.render_state(5, 16);

        assert_eq!(
            render.viewport_slice,
            GuiEditorViewportSlice {
                line_count: 3,
                first_line: 1,
                lines: vec![
                    GuiEditorViewportLine {
                        number: 1,
                        text: "one".to_string(),
                        cursor_column: Some(0),
                        selection: None,
                        syntax_segments: None,
                    },
                    GuiEditorViewportLine {
                        number: 2,
                        text: "two".to_string(),
                        cursor_column: None,
                        selection: None,
                        syntax_segments: None,
                    },
                    GuiEditorViewportLine {
                        number: 3,
                        text: String::new(),
                        cursor_column: None,
                        selection: None,
                        syntax_segments: None,
                    },
                ],
            }
        );
    }

    #[test]
    fn gui_editor_viewport_slice_exposes_replacement_selection_spans() {
        let text = "zero\none\ntwo\nthree";
        let line_count = TextBuffer::from_text(text).line_count();
        let render = gui_editor_viewport_slice(
            text,
            line_count,
            GuiEditorViewportState {
                first_line: 2,
                visible_lines: 3,
            },
            DocumentCursor { row: 1, column: 0 },
            Some(GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 3, column: 2 },
                focus: DocumentCursor { row: 1, column: 1 },
            }),
        );

        assert_eq!(render.first_line, 2);
        assert_eq!(
            render
                .lines
                .iter()
                .map(|line| (line.number, line.selection))
                .collect::<Vec<_>>(),
            vec![
                (
                    2,
                    Some(GuiEditorSelectionSpan {
                        start_column: 1,
                        end_column: 3,
                    }),
                ),
                (
                    3,
                    Some(GuiEditorSelectionSpan {
                        start_column: 0,
                        end_column: 3,
                    }),
                ),
                (
                    4,
                    Some(GuiEditorSelectionSpan {
                        start_column: 0,
                        end_column: 2,
                    }),
                ),
            ]
        );
    }

    #[test]
    fn gui_editor_viewport_selection_span_ignores_empty_and_offscreen_ranges() {
        let document = TextDocument {
            path: PathBuf::from("selection-render.txt"),
            buffer: TextBuffer::from_text("alpha\n\nomega"),
        };
        let cursor = DocumentCursor { row: 0, column: 0 };
        let viewport = GuiEditorViewportState::new(2);

        let empty = gui_editor_viewport_slice(
            &document.buffer.to_text(),
            document.buffer.line_count(),
            viewport,
            cursor,
            Some(GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 2 },
                focus: DocumentCursor { row: 0, column: 2 },
            }),
        );
        assert!(empty.lines.iter().all(|line| line.selection.is_none()));

        let offscreen = gui_editor_viewport_slice(
            &document.buffer.to_text(),
            document.buffer.line_count(),
            viewport,
            cursor,
            Some(GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 2, column: 1 },
                focus: DocumentCursor { row: 2, column: 4 },
            }),
        );
        assert!(offscreen.lines.iter().all(|line| line.selection.is_none()));

        let blank_line = gui_editor_viewport_slice(
            &document.buffer.to_text(),
            document.buffer.line_count(),
            viewport,
            cursor,
            Some(GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 3 },
                focus: DocumentCursor { row: 2, column: 2 },
            }),
        );
        assert_eq!(
            blank_line.lines[1].selection,
            Some(GuiEditorSelectionSpan {
                start_column: 0,
                end_column: 0,
            })
        );
    }

    #[test]
    fn gui_editor_read_only_line_segments_mark_selected_text() {
        let line = GuiEditorViewportLine {
            number: 1,
            text: "abcdef".to_string(),
            cursor_column: None,
            selection: Some(GuiEditorSelectionSpan {
                start_column: 2,
                end_column: 5,
            }),
            syntax_segments: None,
        };

        assert_eq!(
            gui_editor_read_only_line_segments(&line),
            vec![
                GuiEditorReadOnlyLineSegment {
                    text: "ab".to_string(),
                    selected: false,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: "cde".to_string(),
                    selected: true,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: "f".to_string(),
                    selected: false,
                    syntax_color: None,
                },
            ]
        );
    }

    #[test]
    fn gui_editor_read_only_line_segments_paint_blank_selection_cell() {
        let line = GuiEditorViewportLine {
            number: 2,
            text: String::new(),
            cursor_column: None,
            selection: Some(GuiEditorSelectionSpan {
                start_column: 0,
                end_column: 0,
            }),
            syntax_segments: None,
        };

        assert_eq!(
            gui_editor_read_only_line_segments(&line),
            vec![GuiEditorReadOnlyLineSegment {
                text: " ".to_string(),
                selected: true,
                syntax_color: None,
            }]
        );
    }

    #[test]
    fn gui_editor_read_only_line_segments_paint_cursor_cell() {
        let line = GuiEditorViewportLine {
            number: 1,
            text: "abc".to_string(),
            cursor_column: Some(1),
            selection: None,
            syntax_segments: None,
        };

        assert_eq!(
            gui_editor_read_only_line_segments(&line),
            vec![
                GuiEditorReadOnlyLineSegment {
                    text: "a".to_string(),
                    selected: false,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: "b".to_string(),
                    selected: true,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: "c".to_string(),
                    selected: false,
                    syntax_color: None,
                },
            ]
        );

        let end_cursor = GuiEditorViewportLine {
            number: 1,
            text: "abc".to_string(),
            cursor_column: Some(3),
            selection: None,
            syntax_segments: None,
        };
        assert_eq!(
            gui_editor_read_only_line_segments(&end_cursor),
            vec![
                GuiEditorReadOnlyLineSegment {
                    text: "abc".to_string(),
                    selected: false,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: " ".to_string(),
                    selected: true,
                    syntax_color: None,
                },
            ]
        );
    }

    #[test]
    fn gui_editor_read_only_line_segments_preserve_syntax_colors_until_overlay() {
        let keyword = Color::from_rgb8(255, 0, 80);
        let plain = Color::from_rgb8(120, 200, 255);
        let line = GuiEditorViewportLine {
            number: 1,
            text: "let x".to_string(),
            cursor_column: None,
            selection: None,
            syntax_segments: Some(vec![
                GuiEditorSyntaxSegment {
                    text: "let".to_string(),
                    color: keyword,
                },
                GuiEditorSyntaxSegment {
                    text: " x".to_string(),
                    color: plain,
                },
            ]),
        };

        assert_eq!(
            gui_editor_read_only_line_segments(&line),
            vec![
                GuiEditorReadOnlyLineSegment {
                    text: "let".to_string(),
                    selected: false,
                    syntax_color: Some(keyword),
                },
                GuiEditorReadOnlyLineSegment {
                    text: " x".to_string(),
                    selected: false,
                    syntax_color: Some(plain),
                },
            ]
        );

        let selected = GuiEditorViewportLine {
            selection: Some(GuiEditorSelectionSpan {
                start_column: 1,
                end_column: 4,
            }),
            ..line
        };
        assert_eq!(
            gui_editor_read_only_line_segments(&selected),
            vec![
                GuiEditorReadOnlyLineSegment {
                    text: "l".to_string(),
                    selected: false,
                    syntax_color: Some(keyword),
                },
                GuiEditorReadOnlyLineSegment {
                    text: "et ".to_string(),
                    selected: true,
                    syntax_color: None,
                },
                GuiEditorReadOnlyLineSegment {
                    text: "x".to_string(),
                    selected: false,
                    syntax_color: Some(plain),
                },
            ]
        );
    }

    #[test]
    fn gui_editor_read_only_line_spans_carry_color_and_overlay_highlight() {
        let palette = gui_theme_palette(EditorThemeId::Abyss);
        let keyword = Color::from_rgb8(255, 0, 80);
        let line = GuiEditorViewportLine {
            number: 1,
            text: "let".to_string(),
            cursor_column: Some(1),
            selection: None,
            syntax_segments: Some(vec![GuiEditorSyntaxSegment {
                text: "let".to_string(),
                color: keyword,
            }]),
        };

        let spans = gui_editor_read_only_line_spans(&line, palette, false);

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].color, Some(keyword));
        assert!(spans[0].highlight.is_none());
        assert_eq!(spans[1].color, Some(palette.background));
        assert!(spans[1].highlight.is_some());
        assert_eq!(spans[2].color, Some(keyword));
        assert!(spans[2].highlight.is_none());
    }

    #[test]
    fn gui_editor_read_only_line_spans_use_stronger_search_highlight() {
        let palette = gui_theme_palette(EditorThemeId::Nocturne);
        let line = GuiEditorViewportLine {
            number: 1,
            text: "match".to_string(),
            cursor_column: None,
            selection: Some(GuiEditorSelectionSpan {
                start_column: 0,
                end_column: 5,
            }),
            syntax_segments: None,
        };

        let normal = gui_editor_read_only_line_spans(&line, palette, false);
        let search = gui_editor_read_only_line_spans(&line, palette, true);
        let normal_background = match normal[0].highlight.expect("normal highlight").background {
            Background::Color(color) => color,
            _ => panic!("expected normal color highlight"),
        };
        let search_background = match search[0].highlight.expect("search highlight").background {
            Background::Color(color) => color,
            _ => panic!("expected search color highlight"),
        };

        assert!(search_background.a > normal_background.a);
        assert_eq!(search_background.r, normal_background.r);
        assert_eq!(search_background.g, normal_background.g);
        assert_eq!(search_background.b, normal_background.b);
        assert_eq!(search[0].color, Some(palette.background));
    }

    #[test]
    fn gui_editor_word_wrap_ranges_prefer_words_then_long_word_breaks() {
        assert_eq!(
            gui_editor_word_wrap_ranges("hello world again", 8),
            vec![(0, 6), (6, 12), (12, 17)]
        );
        assert_eq!(
            gui_editor_word_wrap_ranges("abcdefgh", 3),
            vec![(0, 3), (3, 6), (6, 8)]
        );
    }

    #[test]
    fn gui_editor_word_wrap_ranges_use_display_width_for_wide_text_and_tabs() {
        assert_eq!(
            gui_editor_word_wrap_ranges("ab界cd", 4),
            vec![(0, 3), (3, 5)]
        );
        assert_eq!(gui_editor_word_wrap_ranges("界界", 2), vec![(0, 1), (1, 2)]);
        assert_eq!(
            gui_editor_word_wrap_ranges("a\tbc", 4),
            vec![(0, 1), (1, 2), (2, 4)]
        );
    }

    #[test]
    fn gui_editor_visual_row_mouse_mapping_uses_display_width() {
        let settings = EditorSettings::default();
        let character_width = gui_editor_replacement_character_width(settings);

        assert_eq!(
            gui_editor_replacement_mouse_point_from_visual_row_point(
                iced::Point::new(character_width * 0.2, 0.0),
                2,
                5,
                "a界b",
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 5,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_visual_row_point(
                iced::Point::new(character_width * 3.1, 0.0),
                2,
                5,
                "a界b",
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 7,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_visual_row_point(
                iced::Point::new(character_width * 3.8, 0.0),
                2,
                5,
                "a界b",
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 8,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_visual_row_point(
                iced::Point::new(character_width * 2.0, 0.0),
                2,
                5,
                "a\tb",
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 6,
            }
        );
    }

    #[test]
    fn gui_editor_replacement_row_height_is_fixed_from_editor_font_size() {
        let mut settings = EditorSettings {
            gui_font_size: 16,
            ..EditorSettings::default()
        };

        assert_eq!(gui_editor_replacement_row_height(settings), 21.0);

        settings.gui_font_size = 17;
        assert_eq!(gui_editor_replacement_row_height(settings), 23.0);
    }

    #[test]
    fn gui_editor_visible_row_budget_avoids_partial_row_overflow() {
        assert_eq!(gui_editor_visible_row_budget(8.0, 20.0), 1);
        assert_eq!(gui_editor_visible_row_budget(40.0, 20.0), 2);
        assert_eq!(gui_editor_visible_row_budget(47.9, 20.0), 2);
        assert_eq!(gui_editor_visible_row_budget(60.0, 20.0), 3);
    }

    #[test]
    fn gui_editor_read_only_visual_rows_keep_gutter_and_cursor_snug() {
        let line = GuiEditorViewportLine {
            number: 7,
            text: "alpha beta gamma".to_string(),
            cursor_column: Some(8),
            selection: Some(GuiEditorSelectionSpan {
                start_column: 6,
                end_column: 11,
            }),
            syntax_segments: None,
        };

        let rows = gui_editor_read_only_visual_rows(&[line], 7, Wrapping::Word, 6);

        assert_eq!(
            rows.iter()
                .map(|row| (row.line.text.as_str(), row.show_line_number))
                .collect::<Vec<_>>(),
            vec![("alpha ", true), ("beta ", false), ("gamma", false)]
        );
        assert_eq!(rows[1].viewport_row, 0);
        assert_eq!(rows[1].source_column_start, 6);
        assert_eq!(rows[1].line.cursor_column, Some(2));
        assert_eq!(
            rows[1].line.selection,
            Some(GuiEditorSelectionSpan {
                start_column: 0,
                end_column: 5,
            })
        );
    }

    #[test]
    fn gui_editor_read_only_render_model_scrolls_text_and_gutter_together() {
        let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

        adapter.apply(GuiEditorCommand::ScrollViewportLines(2));
        let render = adapter.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16);
        let model = gui_editor_read_only_render_model(&render.viewport_slice);

        assert_eq!(model.first_line, 3);
        assert_eq!(model.line_count, 100);
        assert_eq!(
            model.gutter_text,
            gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES)
        );
        assert!(model.body_text.starts_with("3\n4\n5\n"));
        assert!(model.body_text.ends_with("\n34"));
        assert_eq!(model.cursor_row_in_view, Some(0));
        assert_eq!(model.cursor_column, Some(0));
    }

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
            gui_editor_replacement_selected_text(&document, selection.expect("selection"))
                .as_deref(),
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
    }

    #[test]
    fn gui_editor_replacement_mouse_point_maps_viewport_to_clamped_cursor() {
        let document = TextDocument {
            path: PathBuf::from("mouse.txt"),
            buffer: TextBuffer::from_text("one\ntwø\nthree"),
        };
        let mut viewport = GuiEditorViewportState::new(2);
        viewport.scroll_by(1, document.buffer.line_count());

        assert_eq!(
            gui_editor_replacement_cursor_from_mouse_point(
                &document.buffer,
                viewport,
                GuiEditorReplacementMousePoint {
                    viewport_row: 0,
                    column: 99,
                },
            ),
            DocumentCursor { row: 1, column: 3 }
        );
        assert_eq!(
            gui_editor_replacement_cursor_from_mouse_point(
                &document.buffer,
                viewport,
                GuiEditorReplacementMousePoint {
                    viewport_row: 99,
                    column: 2,
                },
            ),
            DocumentCursor { row: 2, column: 2 }
        );
    }

    #[test]
    fn gui_editor_replacement_line_point_accounts_for_wrapped_visual_row() {
        let settings = EditorSettings::default();
        let character_width = f32::from(settings.gui_font_size) * 0.62;
        let line_height = f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT;
        let body_width = character_width * 8.0;

        assert_eq!(
            gui_editor_replacement_mouse_point_from_line_point(
                iced::Point::new(character_width * 2.2, line_height * 1.2),
                3,
                settings,
                body_width,
                Wrapping::None,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 3,
                column: 2,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_line_point(
                iced::Point::new(character_width * 2.2, line_height * 1.2),
                3,
                settings,
                body_width,
                Wrapping::WordOrGlyph,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 3,
                column: 10,
            }
        );
    }

    #[test]
    fn gui_editor_replacement_body_point_subtracts_gutter_before_text_column_mapping() {
        let settings = EditorSettings::default();
        let character_width = gui_editor_replacement_character_width(settings);
        let row_height = gui_editor_replacement_row_height(settings);
        let snug_line =
            "See `docs/06-SECURITY.md` for the project's working threat model and release gate.";
        let lines = vec![
            "alpha".to_string(),
            "beta".to_string(),
            snug_line.to_string(),
        ];
        let viewport = GuiEditorViewportState {
            first_line: 1,
            visible_lines: 3,
        };
        let slice = gui_editor_viewport_slice_from_lines(
            &lines,
            lines.len(),
            viewport,
            DocumentCursor { row: 0, column: 0 },
            None,
        );
        let gutter_width = 42.0;
        let hit_test = GuiEditorBodyHitTest {
            columns: 120,
            visible_rows: 3,
            text_origin_x: gutter_width,
        };

        assert_eq!(
            gui_editor_replacement_mouse_point_from_body_point(
                iced::Point::new(gutter_width + 1.0, row_height * 0.25),
                &slice.lines,
                slice.first_line,
                Wrapping::WordOrGlyph,
                hit_test,
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 0,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_body_point(
                iced::Point::new(gutter_width + character_width * 2.2, row_height * 1.25),
                &slice.lines,
                slice.first_line,
                Wrapping::WordOrGlyph,
                hit_test,
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 1,
                column: 2,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_body_point(
                iced::Point::new(gutter_width - 8.0, row_height * 0.25),
                &slice.lines,
                slice.first_line,
                Wrapping::WordOrGlyph,
                hit_test,
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 0,
            }
        );
        assert_eq!(
            gui_editor_replacement_mouse_point_from_body_point(
                iced::Point::new(
                    gutter_width + character_width * (snug_line.chars().count() as f32 - 0.1),
                    row_height * 2.25,
                ),
                &slice.lines,
                slice.first_line,
                Wrapping::WordOrGlyph,
                hit_test,
                settings,
            ),
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: snug_line.chars().count(),
            }
        );
    }

    #[test]
    fn gui_editor_replacement_mouse_click_moves_cursor_and_clears_selection() {
        let document = TextDocument {
            path: PathBuf::from("mouse-click.txt"),
            buffer: TextBuffer::from_text("alpha\nbeta"),
        };
        let mut cursor = DocumentCursor { row: 0, column: 0 };
        let mut viewport = GuiEditorViewportState::new(3);
        let mut selection = Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 0, column: 1 },
            focus: DocumentCursor { row: 1, column: 2 },
        });

        gui_editor_replacement_mouse_click(
            &document,
            &mut cursor,
            &mut viewport,
            &mut selection,
            GuiEditorReplacementMousePoint {
                viewport_row: 1,
                column: 99,
            },
        );

        assert_eq!(cursor, DocumentCursor { row: 1, column: 4 });
        assert_eq!(selection, None);
        assert_eq!(viewport.first_line, 1);
    }

    #[test]
    fn gui_editor_replacement_mouse_click_preserves_visible_viewport() {
        let document = TextDocument {
            path: PathBuf::from("mouse-click-scroll.txt"),
            buffer: TextBuffer::from_text("one\ntwo\nthree\nfour\nfive"),
        };
        let mut cursor = DocumentCursor { row: 0, column: 0 };
        let mut viewport = GuiEditorViewportState::new(3);
        viewport.scroll_by(2, document.buffer.line_count());
        let mut selection = None;

        gui_editor_replacement_mouse_click(
            &document,
            &mut cursor,
            &mut viewport,
            &mut selection,
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 1,
            },
        );

        assert_eq!(cursor, DocumentCursor { row: 4, column: 1 });
        assert_eq!(viewport.first_line, 3);
        assert_eq!(selection, None);
    }

    #[test]
    fn gui_editor_replacement_mouse_drag_sets_selection_and_cursor() {
        let document = TextDocument {
            path: PathBuf::from("mouse-drag.txt"),
            buffer: TextBuffer::from_text("zero\none\ntwo\nthree"),
        };
        let mut cursor = DocumentCursor { row: 0, column: 0 };
        let mut viewport = GuiEditorViewportState::new(3);
        viewport.scroll_by(1, document.buffer.line_count());
        let mut selection = None;

        gui_editor_replacement_mouse_drag(
            &document,
            &mut cursor,
            &mut viewport,
            &mut selection,
            DocumentCursor { row: 3, column: 5 },
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 1,
            },
        );

        assert_eq!(cursor, DocumentCursor { row: 1, column: 1 });
        assert_eq!(
            selection.map(GuiEditorReplacementSelection::normalized),
            Some((
                DocumentCursor { row: 1, column: 1 },
                DocumentCursor { row: 3, column: 5 },
            ))
        );
        assert_eq!(
            gui_editor_replacement_selected_text(&document, selection.expect("selection"))
                .as_deref(),
            Some("ne\ntwo\nthree")
        );
        assert_eq!(viewport.first_line, 2);
    }

    #[test]
    fn gui_editor_replacement_pointer_click_updates_live_tile_cursor_without_dirtying() {
        let temp = TempArea::new("gui-replacement-pointer-click");
        let file = temp.path("pointer-click.txt");
        fs::write(&file, "alpha\nbeta").expect("write pointer click");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;

        state.replacement_editor_pointer_moved(
            pane,
            GuiEditorReplacementMousePoint {
                viewport_row: 1,
                column: 2,
            },
        );
        state.replacement_editor_pointer_pressed(pane);
        state.replacement_editor_pointer_released(pane);

        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 2 }
        );
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 1, column: 2 }
        );
        assert_eq!(state.active_editor().replacement_selection, None);
        assert!(!state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(state.status_message, "cursor moved");
    }

    #[test]
    fn gui_editor_replacement_pointer_drag_updates_live_selection() {
        let temp = TempArea::new("gui-replacement-pointer-drag");
        let file = temp.path("pointer-drag.txt");
        fs::write(&file, "zero\none\ntwo\nthree").expect("write pointer drag");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;
        let line_count = state.workspace.active_tile().document.buffer.line_count();
        state
            .panes
            .get_mut(pane)
            .expect("pane")
            .editor
            .viewport
            .visible_lines = 3;
        state
            .panes
            .get_mut(pane)
            .expect("pane")
            .editor
            .viewport
            .scroll_by(1, line_count);
        assert_eq!(
            state
                .panes
                .get(pane)
                .expect("pane")
                .editor
                .viewport
                .first_line,
            2
        );

        state.replacement_editor_pointer_moved(
            pane,
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 1,
            },
        );
        state.replacement_editor_pointer_pressed(pane);
        state.replacement_editor_pointer_moved(
            pane,
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 2,
            },
        );
        state.replacement_editor_pointer_released(pane);

        let selection = state
            .active_editor()
            .replacement_selection
            .expect("live selection");
        assert_eq!(
            selection.normalized(),
            (
                DocumentCursor { row: 1, column: 1 },
                DocumentCursor { row: 3, column: 2 },
            )
        );
        assert_eq!(
            gui_editor_replacement_selected_text(
                &state.workspace.active_tile().document,
                selection
            )
            .as_deref(),
            Some("ne\ntwo\nth")
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 3, column: 2 }
        );
        assert_eq!(
            state
                .panes
                .get(pane)
                .expect("pane")
                .editor
                .viewport
                .first_line,
            2
        );
        assert!(!state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(state.status_message, "selected text");
    }

    #[test]
    fn gui_editor_replacement_edge_drag_scrolls_and_extends_selection() {
        let temp = TempArea::new("gui-replacement-edge-drag");
        let file = temp.path("edge-drag.txt");
        let text = (1..=12)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file, text).expect("write edge drag");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;
        if let Some(pane_state) = state.panes.get_mut(pane) {
            pane_state.editor.viewport.visible_lines = 3;
            pane_state.editor.viewport.first_line = 1;
        }

        state.replacement_editor_pointer_moved(
            pane,
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 0,
            },
        );
        state.replacement_editor_pointer_pressed(pane);
        state.replacement_editor_body_pointer_moved(
            pane,
            GuiEditorReplacementMousePoint {
                viewport_row: 2,
                column: 4,
            },
            GuiEditorDragEdge {
                pane,
                direction: 1,
                column: 4,
            },
        );
        state.replacement_editor_drag_tick();

        let pane_state = state.panes.get(pane).expect("pane");
        assert_eq!(pane_state.editor.viewport.first_line, 2);
        assert_eq!(
            pane_state.editor.document_cursor(),
            DocumentCursor { row: 3, column: 4 }
        );
        assert_eq!(
            pane_state
                .editor
                .replacement_selection
                .map(GuiEditorReplacementSelection::normalized),
            Some((
                DocumentCursor { row: 0, column: 0 },
                DocumentCursor { row: 3, column: 4 },
            ))
        );
        assert_eq!(state.status_message, "selected text");
    }

    #[test]
    fn gui_editor_scrollbar_thumb_drag_updates_viewport_without_cursor_move() {
        let temp = TempArea::new("gui-scrollbar-thumb-drag");
        let file = temp.path("scrollbar.txt");
        let text = (1..=100)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file, text).expect("write scrollbar file");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let pane = state.active_pane;
        if let Some(pane_state) = state.panes.get_mut(pane) {
            pane_state.editor.viewport.visible_lines = 10;
            pane_state.editor.viewport.first_line = 1;
        }

        let model = gui_editor_scrollbar_model(100, 1, 10, 200.0);
        state.replacement_editor_scrollbar_moved(pane, model.thumb_top + 2.0, model);
        state.replacement_editor_scrollbar_pressed(pane);
        state.replacement_editor_scrollbar_moved(pane, 170.0, model);

        let expected = gui_editor_scrollbar_first_line_from_thumb_y(model, 170.0, 2.0);
        let pane_state = state.panes.get(pane).expect("pane");
        assert_eq!(pane_state.editor.viewport.first_line, expected);
        assert_eq!(
            pane_state.editor.document_cursor(),
            DocumentCursor { row: 0, column: 0 }
        );
        assert!(!pane_state.editor.viewport_tracks_cursor);
        assert_eq!(state.status_message, "viewport scrolled");
    }

    fn replacement_key_event(
        key: Key,
        modifiers: keyboard::Modifiers,
        text: Option<&str>,
    ) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: text.map(Into::into),
            repeat: false,
        }
    }

    #[test]
    fn gui_editor_replacement_keyboard_bridge_maps_text_and_navigation() {
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Character("a".into()),
                keyboard::Modifiers::NONE,
                Some("a"),
            )),
            vec![GuiEditorReplacementInput::InsertChar('a')]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Character("A".into()),
                keyboard::Modifiers::SHIFT,
                Some("A"),
            )),
            vec![GuiEditorReplacementInput::InsertChar('A')]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Character("x".into()),
                keyboard::Modifiers::CTRL,
                Some("x"),
            )),
            Vec::<GuiEditorReplacementInput>::new()
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Character("a".into()),
                keyboard::Modifiers::CTRL,
                Some("a"),
            )),
            vec![GuiEditorReplacementInput::SelectAll]
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("c".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            Some(GuiMenuCommand::Copy)
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("x".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            Some(GuiMenuCommand::Cut)
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("v".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            Some(GuiMenuCommand::Paste)
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("z".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            Some(GuiMenuCommand::Undo)
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("z".into()),
                keyboard::Modifiers::CTRL.union(keyboard::Modifiers::SHIFT),
                None,
            )),
            Some(GuiMenuCommand::Redo)
        );
        assert_eq!(
            gui_editor_clipboard_shortcut_command(&replacement_key_event(
                Key::Character("y".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            Some(GuiMenuCommand::Redo)
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Enter),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::InsertNewline]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Backspace),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::DeleteBackward]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Delete),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::DeleteForward]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Escape),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::ClearSelection]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Home),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::MoveLineStart]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::End),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::MoveLineEnd]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::ArrowDown),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down)]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::PageDown),
                keyboard::Modifiers::NONE,
                None,
            )),
            vec![GuiEditorReplacementInput::ScrollViewportLines(
                GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32
            )]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::ArrowLeft),
                keyboard::Modifiers::CTRL,
                None,
            )),
            vec![GuiEditorReplacementInput::Move(
                kfnotepad::CursorMove::WordLeft
            )]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::ArrowRight),
                keyboard::Modifiers::CTRL,
                None,
            )),
            vec![GuiEditorReplacementInput::Move(
                kfnotepad::CursorMove::WordRight
            )]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Backspace),
                keyboard::Modifiers::CTRL,
                None,
            )),
            vec![GuiEditorReplacementInput::DeletePreviousWord]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Named(Named::Delete),
                keyboard::Modifiers::CTRL,
                None,
            )),
            vec![GuiEditorReplacementInput::DeleteNextWord]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
                Key::Character("k".into()),
                keyboard::Modifiers::CTRL,
                None,
            )),
            vec![GuiEditorReplacementInput::DeleteToLineEnd]
        );
    }

    #[test]
    fn gui_editor_replacement_ime_commit_maps_to_text_insertion() {
        assert_eq!(
            gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
                "かな".to_string()
            )),
            vec![
                GuiEditorReplacementInput::InsertChar('か'),
                GuiEditorReplacementInput::InsertChar('な'),
            ]
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Preedit(
                "か".to_string(),
                Some(0..3),
            )),
            Vec::<GuiEditorReplacementInput>::new()
        );
        assert_eq!(
            gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Closed),
            Vec::<GuiEditorReplacementInput>::new()
        );
    }

    #[test]
    fn gui_editor_ime_preedit_renders_at_cursor_without_mutating_line() {
        let line = GuiEditorViewportLine {
            number: 1,
            text: "start end".to_string(),
            cursor_column: Some(6),
            selection: None,
            syntax_segments: Some(vec![GuiEditorSyntaxSegment {
                text: "start end".to_string(),
                color: Color::WHITE,
            }]),
        };
        let preedit = GuiImePreedit {
            tile_id: GuiTileId(1),
            content: "かな".to_string(),
            selection: Some(0..3),
        };

        let rendered = gui_editor_viewport_line_with_ime_preedit(line.clone(), Some(&preedit));

        assert_eq!(line.text, "start end");
        assert_eq!(rendered.text, "start かなend");
        assert_eq!(rendered.cursor_column, None);
        assert_eq!(
            rendered.selection,
            Some(GuiEditorSelectionSpan {
                start_column: 6,
                end_column: 7,
            })
        );
        assert_eq!(rendered.syntax_segments, None);
    }

    #[test]
    fn gui_editor_ime_request_cursor_rect_tracks_visual_row_and_gutter() {
        let request = GuiImeInputMethodRequest {
            visual_row: 3,
            cursor_column: 5,
            gutter_width: 42.0,
            character_width: 9.0,
            row_height: 18.0,
            preedit: Some(input_method::Preedit {
                content: "かな".to_string(),
                selection: Some(0..3),
                text_size: Some(Pixels(16.0)),
            }),
        };

        assert_eq!(
            request.cursor_rect(Rectangle::new(
                iced::Point::new(10.0, 20.0),
                Size::new(500.0, 300.0)
            )),
            Rectangle::new(iced::Point::new(97.0, 74.0), Size::new(1.0, 18.0))
        );
        assert_eq!(
            request
                .preedit
                .as_ref()
                .map(|preedit| preedit.content.as_str()),
            Some("かな")
        );
    }

    #[test]
    fn gui_editor_replacement_keyboard_bridge_applies_when_explicitly_routed() {
        let temp = TempArea::new("gui-replacement-bridge");
        let file = temp.path("bridge.txt");
        fs::write(&file, "one\ntwo").expect("write bridge");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        state.apply_replacement_editor_inputs_to_active_tile(vec![
            GuiEditorReplacementInput::InsertChar('X'),
            GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
            GuiEditorReplacementInput::InsertNewline,
        ]);

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "Xo\nne\ntwo"
        );
        assert!(state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 0 }
        );
        assert_eq!(state.active_editor().text(), "Xo\nne\ntwo");
        assert_eq!(
            state.active_editor().document_cursor(),
            DocumentCursor { row: 1, column: 0 }
        );
        assert_eq!(state.status_message, "replacement edit");
    }

    #[test]
    fn gui_editor_replacement_home_end_move_within_current_line() {
        let temp = TempArea::new("gui-replacement-home-end");
        let file = temp.path("home-end.txt");
        fs::write(&file, "abc\ndefgh\n").expect("write home end");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        state
            .panes
            .get_mut(state.active_pane)
            .expect("active pane")
            .editor
            .move_to(DocumentCursor { row: 1, column: 2 });
        state.sync_active_editor_to_document();

        state.apply_replacement_editor_inputs_to_active_tile(vec![
            GuiEditorReplacementInput::MoveLineEnd,
        ]);
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 5 }
        );

        state.apply_replacement_editor_inputs_to_active_tile(vec![
            GuiEditorReplacementInput::MoveLineStart,
        ]);
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 1, column: 0 }
        );
        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "abc\ndefgh\n"
        );
        assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    }

    #[test]
    fn gui_editor_replacement_ime_commit_applies_when_explicitly_routed() {
        let temp = TempArea::new("gui-replacement-ime");
        let file = temp.path("ime.txt");
        fs::write(&file, "start").expect("write ime");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        state.apply_replacement_editor_inputs_to_active_tile(
            gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
                "かな".to_string(),
            )),
        );

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "かなstart"
        );
        assert!(state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 2 }
        );
        assert_eq!(state.active_editor().text(), "かなstart");
    }

    #[test]
    fn gui_editor_replacement_ime_preedit_is_transient_until_commit() {
        let temp = TempArea::new("gui-replacement-ime-preedit");
        let file = temp.path("ime-preedit.txt");
        fs::write(&file, "start").expect("write ime preedit");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );
        let tile_id = state.workspace.active_tile().id;

        let _ = update(
            &mut state,
            Message::ReplacementEditorIme(input_method::Event::Opened),
        );
        let _ = update(
            &mut state,
            Message::ReplacementEditorIme(input_method::Event::Preedit(
                "かな".to_string(),
                Some(0..3),
            )),
        );

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "start"
        );
        assert!(!state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(
            state.replacement_ime_preedit,
            Some(GuiImePreedit {
                tile_id,
                content: "かな".to_string(),
                selection: Some(0..3),
            })
        );

        let _ = update(
            &mut state,
            Message::ReplacementEditorIme(input_method::Event::Commit("かな".to_string())),
        );

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "かなstart"
        );
        assert!(state.workspace.active_tile().document.buffer.is_dirty());
        assert_eq!(state.replacement_ime_preedit, None);
    }

    #[test]
    fn gui_editor_replacement_selection_persists_until_next_active_tile_edit() {
        let temp = TempArea::new("gui-replacement-selection");
        let file = temp.path("selection.txt");
        fs::write(&file, "alpha beta").expect("write selection");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        state.apply_replacement_editor_inputs_to_active_tile(vec![
            GuiEditorReplacementInput::SelectRange {
                anchor: DocumentCursor { row: 0, column: 6 },
                focus: DocumentCursor { row: 0, column: 10 },
            },
        ]);

        assert_eq!(
            state
                .panes
                .get(state.active_pane)
                .and_then(|pane| pane.editor.replacement_selection),
            Some(GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 6 },
                focus: DocumentCursor { row: 0, column: 10 },
            })
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 10 }
        );
        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "alpha beta"
        );

        state.apply_replacement_editor_inputs_to_active_tile(vec![
            GuiEditorReplacementInput::InsertChar('X'),
        ]);

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "alpha X"
        );
        assert_eq!(
            state
                .panes
                .get(state.active_pane)
                .and_then(|pane| pane.editor.replacement_selection),
            None
        );
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 7 }
        );
        assert_eq!(state.active_editor().text(), "alpha X");
    }

    #[test]
    fn gui_editor_replacement_message_edits_active_tile_when_renderer_is_live() {
        let temp = TempArea::new("gui-replacement-live");
        let file = temp.path("live.txt");
        fs::write(&file, "unchanged").expect("write live");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: vec![file],
            },
            temp.root.clone(),
        );

        let _ = update(
            &mut state,
            Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
        );

        assert_eq!(
            state.workspace.active_tile().document.buffer.to_text(),
            "Xunchanged"
        );
        assert_eq!(state.active_editor().text(), "Xunchanged");
        assert_eq!(state.status_message, "replacement edit");
    }

    #[test]
    fn gui_restore_last_workspace_toggle_persists_config() {
        let temp = TempArea::new("gui-restore-toggle");
        let config = temp.path("config").join("kfnotepad").join("config.toml");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(config.clone());
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

        assert!(state.settings.gui_restore_last_workspace);
        assert_eq!(state.status_message, "restore last workspace: on");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

        let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(false));

        assert!(!state.settings.gui_restore_last_workspace);
        assert_eq!(state.status_message, "restore last workspace: off");
        assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
    }

    #[test]
    fn gui_restore_last_workspace_toggle_rolls_back_on_config_save_failure() {
        let temp = TempArea::new("gui-restore-toggle-failure");
        let blocked_parent = temp.path("blocked");
        fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
        let mut state = KfnotepadGui::new_with_current_dir(
            GuiLaunch {
                requested_paths: Vec::new(),
            },
            temp.root.clone(),
        );
        state.config_path = Some(blocked_parent.join("config.toml"));
        state.settings = EditorSettings {
            theme_id: EditorThemeId::Terminal,
            show_line_numbers: true,
            wrap_lines: false,
            gui_restore_last_workspace: false,
            ..EditorSettings::default()
        };

        let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

        assert!(!state.settings.gui_restore_last_workspace);
        assert!(state.status_message.starts_with("settings save failed: "));
        assert_eq!(
            fs::read_to_string(&blocked_parent).expect("read blocked parent"),
            "not a directory\n"
        );
    }

    struct TempArea {
        root: PathBuf,
    }

    impl TempArea {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos();
            let root =
                env::temp_dir().join(format!("kfnotepad-{label}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&root).expect("create temp dir");
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

    fn pane_for_path(state: &KfnotepadGui, path: &PathBuf) -> pane_grid::Pane {
        state
            .panes
            .iter()
            .find_map(|(pane, pane_state)| {
                state
                    .workspace
                    .tile(pane_state.tile_id)
                    .and_then(|tile| (tile.document.path == *path).then_some(*pane))
            })
            .expect("pane for path")
    }

    fn pane_x(state: &KfnotepadGui, pane: pane_grid::Pane) -> f32 {
        state
            .panes
            .layout()
            .pane_regions(
                GUI_PANE_GRID_SPACING,
                GUI_PANE_GRID_MIN_SIZE,
                GUI_PANE_GRID_REFERENCE_SIZE,
            )
            .get(&pane)
            .expect("pane region")
            .x
    }

    fn node_path(state: &KfnotepadGui, node: &pane_grid::Node) -> Option<PathBuf> {
        let pane_grid::Node::Pane(pane) = node else {
            return None;
        };
        let tile_id = state.panes.get(*pane)?.tile_id;
        Some(state.workspace.tile(tile_id)?.document.path.clone())
    }

    fn layout_leaf_ordinals(node: &GuiLayoutNode) -> Vec<usize> {
        match node {
            GuiLayoutNode::Leaf { ordinal } => vec![*ordinal],
            GuiLayoutNode::Split { first, second, .. } => {
                let mut ordinals = layout_leaf_ordinals(first);
                ordinals.extend(layout_leaf_ordinals(second));
                ordinals
            }
        }
    }
}
