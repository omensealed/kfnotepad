use super::*;

pub(in crate::gui::app::state) fn gui_editor_replacement_character_width(
    settings: EditorSettings,
) -> f32 {
    (f32::from(settings.gui_font_size) * 0.62).max(1.0)
}

pub(in crate::gui::app::state) fn gui_editor_replacement_row_height(
    settings: EditorSettings,
) -> f32 {
    (f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT)
        .ceil()
        .max(1.0)
}

pub(in crate::gui::app::state) fn gui_editor_visible_row_budget(
    surface_height: f32,
    row_height: f32,
) -> usize {
    (surface_height.max(row_height) / row_height.max(1.0))
        .floor()
        .max(1.0) as usize
}

pub(in crate::gui::app::state) fn gui_editor_replacement_scroll_delta_lines(
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
