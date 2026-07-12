//! Top-level workspace mouse event routing.

use super::*;

pub(crate) fn handle_workspace_mouse_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> InputResult {
    if let Some(result) = handle_sidebar_mouse_event(workspace, runtime, event, context) {
        return result;
    }

    if let Some(result) = handle_editor_scroll_mouse_event(workspace, runtime, event, context) {
        return result;
    }

    if event.kind != MouseEventKind::Down(MouseButton::Left) {
        return InputResult::Ignored;
    }

    handle_workspace_left_click(
        workspace,
        runtime,
        event,
        context,
        mouse_render_frame(runtime, context),
    )
}
