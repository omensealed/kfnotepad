//! Pane-grid content, title controls, and editor body composition.

#[path = "panes/body.rs"]
mod body;
#[path = "panes/controls.rs"]
mod controls;
#[path = "panes/grid.rs"]
mod grid;

pub(super) use body::gui_pane_body;
pub(super) use controls::gui_pane_controls;
pub(super) use grid::gui_pane_grid_view;
