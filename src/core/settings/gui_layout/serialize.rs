pub fn serialize_gui_layout(layout: &GuiLayout) -> String {
    let mut lines = vec![
        "version = 1".to_string(),
        format!("browser_visible = {}", layout.browser_visible),
        "root = 0".to_string(),
    ];
    if let Some(browser_width_px) = layout.browser_width_px {
        lines.insert(2, format!("browser_width_px = {browser_width_px}"));
    }
    let mut next_id = 1;
    write_gui_layout_node(&layout.root, 0, &mut next_id, &mut lines);
    let minimized = layout
        .minimized_ordinals
        .iter()
        .map(|ordinal: &usize| ordinal.to_string())
        .collect::<Vec<_>>()
        .join(",");
    lines.push(format!("minimized = {minimized}"));
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn write_gui_layout_node(
    node: &GuiLayoutNode,
    node_id: usize,
    next_id: &mut usize,
    lines: &mut Vec<String>,
) {
    match node {
        GuiLayoutNode::Leaf { ordinal } => {
            lines.push(format!("node.{node_id} = leaf {ordinal}"));
        }
        GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } => {
            let first_id = *next_id;
            *next_id += 1;
            let second_id = *next_id;
            *next_id += 1;
            let axis = match axis {
                GuiLayoutAxis::Horizontal => "horizontal",
                GuiLayoutAxis::Vertical => "vertical",
            };
            lines.push(format!(
                "node.{node_id} = split {axis} {ratio_per_mille} {first_id} {second_id}"
            ));
            write_gui_layout_node(first, first_id, next_id, lines);
            write_gui_layout_node(second, second_id, next_id, lines);
        }
    }
}
