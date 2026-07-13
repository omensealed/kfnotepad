use super::*;

pub(in crate::gui::app::state) fn gui_left_panel_width(
    visible: bool,
    configured_width: f32,
) -> f32 {
    if visible {
        clamp_browser_width(configured_width)
    } else {
        0.0
    }
}
