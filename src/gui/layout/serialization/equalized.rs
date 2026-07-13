//! Balanced grid-like saved layout construction.

use super::super::*;

pub(in crate::gui::app::state) fn equalized_tile_layout_node(
    count: usize,
) -> Option<GuiLayoutNode> {
    if count == 0 {
        return None;
    }
    let columns = (count as f64).sqrt().ceil() as usize;
    let rows = count.div_ceil(columns);
    let mut next_ordinal = 0usize;
    let mut column_nodes = Vec::new();

    for _column in 0..columns {
        let remaining = count.saturating_sub(next_ordinal);
        if remaining == 0 {
            break;
        }
        let column_len = remaining.min(rows);
        let row_nodes = (0..column_len)
            .map(|_| {
                let ordinal = next_ordinal;
                next_ordinal += 1;
                GuiLayoutNode::Leaf { ordinal }
            })
            .collect::<Vec<_>>();
        column_nodes.push(equalized_axis_node(GuiLayoutAxis::Horizontal, row_nodes));
    }

    Some(equalized_axis_node(GuiLayoutAxis::Vertical, column_nodes))
}

pub(in crate::gui::app::state) fn equalized_axis_node(
    axis: GuiLayoutAxis,
    mut nodes: Vec<GuiLayoutNode>,
) -> GuiLayoutNode {
    assert!(
        !nodes.is_empty(),
        "equalized layout axis needs at least one child"
    );
    if nodes.len() == 1 {
        return nodes.remove(0);
    }

    let right = nodes.pop().expect("checked non-empty nodes");
    let left_count = nodes.len();
    let total = left_count + 1;
    let ratio_per_mille = ((left_count * 1000) / total).clamp(1, 999) as u16;
    GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first: Box::new(equalized_axis_node(axis, nodes)),
        second: Box::new(right),
    }
}
