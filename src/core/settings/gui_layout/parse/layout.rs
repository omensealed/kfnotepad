pub fn parse_gui_layout(text: &str, pane_count: usize) -> Option<GuiLayout> {
    if pane_count == 0 {
        return None;
    }

    let mut version = None;
    let mut browser_visible = true;
    let mut browser_width_px = None;
    let mut root_id = None;
    let mut minimized_ordinals = Vec::new();
    let mut node_specs = HashMap::new();

    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        if line.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = line.split_once('=')?;
        let key = raw_key.trim();
        let value = raw_value.trim();

        if key == "version" {
            version = value.parse::<usize>().ok();
        } else if key == "browser_visible" {
            browser_visible = parse_config_bool(value)?;
        } else if key == "browser_width_px" {
            let parsed_width = value.parse::<u16>().ok()?;
            if parsed_width == 0 {
                return None;
            }
            browser_width_px = Some(parsed_width);
        } else if key == "root" {
            root_id = value.parse::<usize>().ok();
        } else if key == "minimized" {
            minimized_ordinals = parse_layout_ordinals(value)?;
        } else if let Some(raw_id) = key.strip_prefix("node.") {
            let id = raw_id.parse::<usize>().ok()?;
            node_specs.insert(id, value.to_string());
        } else {
            continue;
        }
    }

    if version != Some(1) {
        return None;
    }

    let mut seen_nodes = HashSet::new();
    let mut seen_ordinals = HashSet::new();
    let root = parse_gui_layout_node(
        root_id?,
        pane_count,
        &node_specs,
        &mut seen_nodes,
        &mut seen_ordinals,
    )?;
    if seen_ordinals.len() != pane_count {
        return None;
    }

    let mut minimized_seen = HashSet::new();
    for ordinal in &minimized_ordinals {
        if *ordinal >= pane_count || !minimized_seen.insert(*ordinal) {
            return None;
        }
    }

    Some(GuiLayout {
        browser_visible,
        browser_width_px,
        root,
        minimized_ordinals,
    })
}
