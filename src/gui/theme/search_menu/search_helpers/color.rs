use super::*;

pub(in crate::gui::app::state) fn color(red: u8, green: u8, blue: u8) -> Color {
    Color::from_rgb8(red, green, blue)
}
