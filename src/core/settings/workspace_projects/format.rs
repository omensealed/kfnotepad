pub fn parse_gui_workspace_project(text: &str) -> Option<GuiWorkspaceProject> {
    let mut version = None;
    let mut name = None;
    let mut active_ordinal = 0usize;
    let mut files = HashMap::new();
    let mut layout_lines = Vec::new();

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
        } else if key == "name_hex" {
            name = String::from_utf8(hex_to_bytes(value)?).ok();
        } else if key == "active" {
            active_ordinal = value.parse::<usize>().ok()?;
        } else if let Some(raw_ordinal) = key.strip_prefix("file.") {
            let ordinal = raw_ordinal.parse::<usize>().ok()?;
            if files.insert(ordinal, path_from_hex(value)?).is_some() {
                return None;
            }
        } else if let Some(layout_key) = key.strip_prefix("layout.") {
            layout_lines.push(format!("{layout_key} = {value}"));
        } else {
            continue;
        }
    }

    if version != Some(1) {
        return None;
    }

    let mut ordered_files = Vec::new();
    for ordinal in 0..files.len() {
        ordered_files.push(files.remove(&ordinal)?);
    }
    if !files.is_empty() || ordered_files.is_empty() || active_ordinal >= ordered_files.len() {
        return None;
    }

    let layout = if layout_lines.is_empty() {
        None
    } else {
        Some(parse_gui_layout(
            &layout_lines.join("\n"),
            ordered_files.len(),
        )?)
    };

    Some(GuiWorkspaceProject {
        name: name?,
        files: ordered_files,
        active_ordinal,
        layout,
    })
}

pub fn serialize_gui_workspace_project(project: &GuiWorkspaceProject) -> Option<String> {
    if project.name.is_empty()
        || project.files.is_empty()
        || project.active_ordinal >= project.files.len()
    {
        return None;
    }
    if let Some(layout) = &project.layout {
        parse_gui_layout(&serialize_gui_layout(layout), project.files.len())?;
    }

    let mut lines = vec![
        "version = 1".to_string(),
        format!("name_hex = {}", bytes_to_hex(project.name.as_bytes())),
        format!("active = {}", project.active_ordinal),
    ];
    for (ordinal, path) in project.files.iter().enumerate() {
        lines.push(format!("file.{ordinal} = {}", path_to_hex(path)));
    }
    if let Some(layout) = &project.layout {
        for line in serialize_gui_layout(layout).lines() {
            lines.push(format!("layout.{line}"));
        }
    }
    let mut text = lines.join("\n");
    text.push('\n');
    Some(text)
}
