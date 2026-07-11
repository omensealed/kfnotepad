//! TUI runtime event loop extracted from the monolithic terminal app module.

use std::io;
use std::io::Write;
use std::time::Duration;

use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event};
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType};
use kfnotepad::{EditorSettings, EditorTab, EditorWorkspace, SyntaxHighlighter, TabStripItem};

use super::super::terminal_session::TerminalSession;
use crate::tui::input::{
    apply_reader_tick, autosave_tui_current_workspace, current_editor_config_path,
    current_workspace_projects_dir, handle_command_palette_key_event, handle_key_event,
    handle_workspace_key_event, handle_workspace_manager_key_event,
    handle_workspace_menu_key_event, handle_workspace_mouse_event,
    handle_workspace_prompt_key_event, handle_workspace_sidebar_key_event, insert_paste,
    EditorRuntime, InputResult, MouseContext, TUI_READER_TICK_MS,
};
use crate::tui::theme::EditorTheme;
use crate::tui::{render, sidebar};

include!("event_loop/types.rs");
include!("event_loop/runtime_setup.rs");
include!("event_loop/frame.rs");
include!("event_loop/event_read.rs");
include!("event_loop/dispatch.rs");
include!("event_loop/run.rs");
