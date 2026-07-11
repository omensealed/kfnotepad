pub(super) fn panes_from_gui_layout(
    layout: GuiLayout,
    pane_states: Vec<GuiPane>,
) -> (pane_grid::State<GuiPane>, pane_grid::Pane) {
    let mut pane_states = pane_states.into_iter().map(Some).collect::<Vec<_>>();
    let first_ordinal = first_layout_ordinal(&layout.root);
    let first_state = pane_states
        .get_mut(first_ordinal)
        .and_then(Option::take)
        .expect("parsed GUI layout root ordinal must match pane states");
    let (mut panes, first_pane) = pane_grid::State::new(first_state);
    apply_gui_layout_node(&layout.root, first_pane, &mut pane_states, &mut panes);
    for pane_state in pane_states {
        assert!(
            pane_state.is_none(),
            "parsed GUI layout must use every pane"
        );
    }
    (panes, first_pane)
}

pub(super) fn first_layout_ordinal(node: &GuiLayoutNode) -> usize {
    match node {
        GuiLayoutNode::Leaf { ordinal } => *ordinal,
        GuiLayoutNode::Split { first, .. } => first_layout_ordinal(first),
    }
}

pub(super) fn apply_gui_layout_node(
    node: &GuiLayoutNode,
    pane: pane_grid::Pane,
    pane_states: &mut [Option<GuiPane>],
    panes: &mut pane_grid::State<GuiPane>,
) -> pane_grid::Pane {
    match node {
        GuiLayoutNode::Leaf { .. } => pane,
        GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } => {
            let second_ordinal = first_layout_ordinal(second);
            let second_state = pane_states
                .get_mut(second_ordinal)
                .and_then(Option::take)
                .expect("parsed GUI layout child ordinal must match pane states");
            let (second_pane, split) = panes
                .split(iced_axis(*axis), pane, second_state)
                .expect("parsed GUI layout split target must exist");
            panes.resize(split, f32::from(*ratio_per_mille) / 1000.0);
            apply_gui_layout_node(first, pane, pane_states, panes);
            apply_gui_layout_node(second, second_pane, pane_states, panes);
            pane
        }
    }
}

pub(super) fn iced_axis(axis: GuiLayoutAxis) -> pane_grid::Axis {
    match axis {
        GuiLayoutAxis::Horizontal => pane_grid::Axis::Horizontal,
        GuiLayoutAxis::Vertical => pane_grid::Axis::Vertical,
    }
}
