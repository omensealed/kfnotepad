//! Event-loop lifecycle and redraw orchestration.

use std::io;

use kfnotepad::{EditorSettings, EditorWorkspace, SyntaxHighlighter};

use super::dispatch::handle_terminal_event;
use super::event_read::read_event_or_apply_reader_tick;
use super::frame::render_frame;
use super::runtime_setup::{build_editor_runtime, clear_terminal_when_not_using_alternate_screen};
use super::LoopLayout;
use crate::tui::input::{autosave_tui_current_workspace, InputResult};
use crate::tui::render::{
    line_number_width, terminal_width, visible_editor_rows, TuiSyntaxHighlightCache,
};
use crate::tui::terminal_session::TerminalSession;

pub(crate) fn run_editor_workspace(
    mut workspace: EditorWorkspace<'_>,
    loaded_settings: Option<EditorSettings>,
    initial_status: Option<String>,
) -> io::Result<()> {
    let highlighter = SyntaxHighlighter::default();
    let mut runtime = build_editor_runtime(loaded_settings, initial_status);
    let mut terminal = TerminalSession::enter()?;
    clear_terminal_when_not_using_alternate_screen(&mut terminal)?;
    let mut redraw = true;
    let mut layout = LoopLayout {
        visible_rows: visible_editor_rows(0),
        terminal_width: terminal_width(),
        gutter_width: line_number_width(workspace.active_tab().document.as_ref()),
    };
    let mut syntax_cache = TuiSyntaxHighlightCache::default();
    autosave_tui_current_workspace(&workspace, &mut runtime);

    loop {
        if redraw {
            render_frame(
                &mut terminal.stdout,
                &mut workspace,
                &mut runtime,
                &highlighter,
                &mut syntax_cache,
                &mut layout,
            )?;
            redraw = false;
        }

        let (event, tick_redraw) =
            read_event_or_apply_reader_tick(&mut workspace, &mut runtime, layout.visible_rows)?;
        redraw |= tick_redraw;
        let Some(event) = event else {
            continue;
        };

        match handle_terminal_event(event, &mut workspace, &mut runtime, &layout) {
            InputResult::Quit => break,
            InputResult::Handled => redraw = true,
            InputResult::Ignored => {}
        }
    }

    Ok(())
}
