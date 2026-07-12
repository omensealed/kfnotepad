//! Sidebar mouse routing.

use super::*;

pub(super) fn handle_sidebar_mouse_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> Option<InputResult> {
    if runtime.sidebar.is_none() || (event.column as usize) >= context.sidebar_width {
        return None;
    }

    Some(match event.kind {
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
    })
}
