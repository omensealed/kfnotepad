//! Pane-grid creation, minimized-pane extraction, and split-axis selection.

use super::*;

pub(in crate::gui::app::state) fn load_gui_layout(
    path: &std::path::Path,
    pane_count: usize,
) -> Option<GuiLayout> {
    let text = fs::read_to_string(path).ok()?;
    parse_gui_layout(&text, pane_count)
}

pub(in crate::gui::app::state) fn default_panes(
    mut pane_states: Vec<GuiPane>,
) -> (pane_grid::State<GuiPane>, pane_grid::Pane) {
    let first = pane_states.remove(0);
    let (mut panes, mut active_pane) = pane_grid::State::new(first);
    for pane_state in pane_states {
        let split_axis = split_axis_for_pane(&panes, active_pane);
        if let Some((pane, _split)) = panes.split(split_axis, active_pane, pane_state) {
            active_pane = pane;
        }
    }
    (panes, active_pane)
}

pub(in crate::gui::app::state) fn close_minimized_panes_into_tray(
    mut panes: pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
    mut active_pane: pane_grid::Pane,
) -> (pane_grid::State<GuiPane>, Vec<GuiPane>, pane_grid::Pane) {
    let minimized = panes
        .iter()
        .filter_map(|(pane, pane_state)| {
            workspace
                .tile(pane_state.tile_id)
                .and_then(|tile| tile.minimized.then_some(*pane))
        })
        .collect::<Vec<_>>();
    let mut tray = Vec::new();

    for pane in minimized {
        if panes.len() <= 1 {
            break;
        }
        if let Some((pane_state, sibling)) = panes.close(pane) {
            if active_pane == pane {
                active_pane = sibling;
            }
            tray.push(pane_state);
        }
    }

    (panes, tray, active_pane)
}

pub(in crate::gui::app::state) fn split_axis_for_pane(
    panes: &pane_grid::State<GuiPane>,
    pane: pane_grid::Pane,
) -> pane_grid::Axis {
    let Some(region) = panes
        .layout()
        .pane_regions(
            GUI_PANE_GRID_SPACING,
            GUI_PANE_GRID_MIN_SIZE,
            GUI_PANE_GRID_REFERENCE_SIZE,
        )
        .get(&pane)
        .copied()
    else {
        return pane_grid::Axis::Vertical;
    };

    if region.height > region.width {
        pane_grid::Axis::Horizontal
    } else {
        pane_grid::Axis::Vertical
    }
}
