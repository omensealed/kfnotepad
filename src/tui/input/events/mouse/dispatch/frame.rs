//! Render-frame construction for mouse hit testing.

use super::*;

pub(super) fn mouse_render_frame(runtime: &EditorRuntime, context: MouseContext) -> RenderFrame {
    RenderFrame {
        theme: EditorTheme::for_id(runtime.settings.theme_id),
        gutter_width: context.gutter_width,
        terminal_width: context
            .terminal_width
            .saturating_sub(context.sidebar_width)
            .max(1),
        origin_column: context.sidebar_width as u16,
        body_top: context.body_top,
        no_color: false,
    }
}
