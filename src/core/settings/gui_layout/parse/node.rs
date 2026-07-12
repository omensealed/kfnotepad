//! Recursive layout node parsing with cycle and duplicate-leaf rejection.

use super::*;

pub(super) fn parse_gui_layout_node(
    node_id: usize,
    pane_count: usize,
    node_specs: &HashMap<usize, String>,
    seen_nodes: &mut HashSet<usize>,
    seen_ordinals: &mut HashSet<usize>,
) -> Option<GuiLayoutNode> {
    if !seen_nodes.insert(node_id) {
        return None;
    }

    let spec = node_specs.get(&node_id)?;
    let parts = spec.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["leaf", raw_ordinal] => {
            let ordinal = raw_ordinal.parse::<usize>().ok()?;
            if ordinal >= pane_count || !seen_ordinals.insert(ordinal) {
                return None;
            }
            Some(GuiLayoutNode::Leaf { ordinal })
        }
        ["split", raw_axis, raw_ratio, raw_first, raw_second] => {
            let axis = match *raw_axis {
                "horizontal" => GuiLayoutAxis::Horizontal,
                "vertical" => GuiLayoutAxis::Vertical,
                _ => return None,
            };
            let ratio_per_mille = raw_ratio.parse::<u16>().ok()?;
            if !(1..=999).contains(&ratio_per_mille) {
                return None;
            }
            let first_id = raw_first.parse::<usize>().ok()?;
            let second_id = raw_second.parse::<usize>().ok()?;
            Some(GuiLayoutNode::Split {
                axis,
                ratio_per_mille,
                first: Box::new(parse_gui_layout_node(
                    first_id,
                    pane_count,
                    node_specs,
                    seen_nodes,
                    seen_ordinals,
                )?),
                second: Box::new(parse_gui_layout_node(
                    second_id,
                    pane_count,
                    node_specs,
                    seen_nodes,
                    seen_ordinals,
                )?),
            })
        }
        _ => None,
    }
}
