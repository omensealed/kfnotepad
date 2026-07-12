//! Runtime settings initialization and initial terminal clearing.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType};
use kfnotepad::EditorSettings;

use crate::tui::input::{
    current_editor_config_path, current_workspace_projects_dir, EditorRuntime,
};
use crate::tui::terminal_session::TerminalSession;

pub(super) fn build_editor_runtime(
    loaded_settings: Option<EditorSettings>,
    initial_status: Option<String>,
) -> EditorRuntime {
    let config_path = current_editor_config_path();
    let workspace_projects_dir = current_workspace_projects_dir();
    let mut runtime = EditorRuntime {
        settings: loaded_settings.unwrap_or_else(|| {
            config_path
                .as_deref()
                .map(kfnotepad::load_editor_settings)
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
    runtime
}

pub(super) fn clear_terminal_when_not_using_alternate_screen(
    terminal: &mut TerminalSession,
) -> io::Result<()> {
    if !terminal.uses_alternate_screen() {
        queue!(terminal.stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        terminal.stdout.flush()?;
    }
    Ok(())
}
