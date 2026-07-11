pub(super) fn gui_layout_from_state(
    panes: &pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
    browser_visible: bool,
    browser_width: f32,
) -> Option<GuiLayout> {
    let minimized_ordinals = workspace
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(ordinal, tile)| tile.minimized.then_some(ordinal))
        .collect::<Vec<_>>();
    let visible_root = gui_layout_node_from_iced(panes.layout(), panes, workspace)?;
    let root = gui_layout_with_minimized_leaves(visible_root, &minimized_ordinals);

    Some(GuiLayout {
        browser_visible,
        browser_width_px: Some(persisted_browser_width(browser_width)),
        root,
        minimized_ordinals,
    })
}

pub(super) fn gui_layout_with_minimized_leaves(
    mut root: GuiLayoutNode,
    minimized_ordinals: &[usize],
) -> GuiLayoutNode {
    for ordinal in minimized_ordinals {
        root = GuiLayoutNode::Split {
            axis: GuiLayoutAxis::Vertical,
            ratio_per_mille: 500,
            first: Box::new(root),
            second: Box::new(GuiLayoutNode::Leaf { ordinal: *ordinal }),
        };
    }
    root
}

pub(super) fn clamp_browser_width(width: f32) -> f32 {
    width.clamp(GUI_BROWSER_WIDTH_MIN, GUI_BROWSER_WIDTH_MAX)
}

pub(super) fn persisted_browser_width(width: f32) -> u16 {
    clamp_browser_width(width).round() as u16
}

pub(super) fn gui_layout_node_from_iced(
    node: &pane_grid::Node,
    panes: &pane_grid::State<GuiPane>,
    workspace: &GuiWorkspace,
) -> Option<GuiLayoutNode> {
    match node {
        pane_grid::Node::Pane(pane) => {
            let tile_id = panes.get(*pane)?.tile_id;
            let ordinal = workspace.tiles.iter().position(|tile| tile.id == tile_id)?;
            Some(GuiLayoutNode::Leaf { ordinal })
        }
        pane_grid::Node::Split {
            axis, ratio, a, b, ..
        } => Some(GuiLayoutNode::Split {
            axis: gui_layout_axis(*axis),
            ratio_per_mille: ((*ratio * 1000.0).round() as u16).clamp(1, 999),
            first: Box::new(gui_layout_node_from_iced(a, panes, workspace)?),
            second: Box::new(gui_layout_node_from_iced(b, panes, workspace)?),
        }),
    }
}

pub(super) fn gui_layout_axis(axis: pane_grid::Axis) -> GuiLayoutAxis {
    match axis {
        pane_grid::Axis::Horizontal => GuiLayoutAxis::Horizontal,
        pane_grid::Axis::Vertical => GuiLayoutAxis::Vertical,
    }
}

pub(super) fn pane_for_tile_id(
    panes: &pane_grid::State<GuiPane>,
    tile_id: GuiTileId,
) -> Option<pane_grid::Pane> {
    panes
        .iter()
        .find_map(|(pane, pane_state)| (pane_state.tile_id == tile_id).then_some(*pane))
}
