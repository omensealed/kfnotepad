use super::*;

#[path = "render_model/ime.rs"]
mod ime;
#[path = "render_model/line_segments.rs"]
mod line_segments;
#[path = "render_model/line_slicing.rs"]
mod line_slicing;
#[path = "render_model/selection_and_model.rs"]
mod selection_and_model;
#[path = "render_model/syntax_colors.rs"]
mod syntax_colors;
#[path = "render_model/wrapping_and_width.rs"]
mod wrapping_and_width;

pub(in crate::gui::app::state) use self::ime::*;
pub(in crate::gui::app::state) use self::line_segments::*;
pub(in crate::gui::app::state) use self::line_slicing::*;
pub(in crate::gui::app::state) use self::selection_and_model::*;
pub(in crate::gui::app::state) use self::syntax_colors::*;
pub(in crate::gui::app::state) use self::wrapping_and_width::*;
