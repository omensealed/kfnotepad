pub(super) fn gui_path_prompt_label(prompt: GuiPathPrompt) -> &'static str {
    match prompt {
        GuiPathPrompt::Open => "Open path",
        GuiPathPrompt::SaveAs => "Save as path",
        GuiPathPrompt::ManagedNote => "Note title",
        GuiPathPrompt::BrowserCreateFile => "File name",
        GuiPathPrompt::BrowserCreateDirectory => "Folder name",
    }
}

#[cfg(test)]
pub(super) fn gui_icon_label(icon: &str, label: &str) -> String {
    format!("{icon} {label}")
}

pub(super) fn gui_icon_only_label(icon: &str) -> String {
    icon.to_string()
}

pub(super) fn gui_file_name_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

pub(super) fn gui_paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

pub(super) fn gui_sidebar_path_label(path: &Path) -> String {
    let label = path.display().to_string();
    if label.chars().count() <= GUI_PANEL_PATH_MAX_CHARS {
        return label;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if !file_name.is_empty() {
        let suffix = format!(".../{file_name}");
        if suffix.chars().count() <= GUI_PANEL_PATH_MAX_CHARS {
            return suffix;
        }
    }

    let keep = GUI_PANEL_PATH_MAX_CHARS.saturating_sub(3);
    format!(
        "...{}",
        label
            .chars()
            .rev()
            .take(keep)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    )
}

pub(super) fn gui_header_layout_mode(viewport_width: f32) -> GuiHeaderLayoutMode {
    if viewport_width < GUI_HEADER_SPLIT_WIDTH {
        GuiHeaderLayoutMode::SplitActions
    } else {
        GuiHeaderLayoutMode::SingleRow
    }
}

pub(super) fn gui_search_layout_mode(viewport_width: f32) -> GuiSearchLayoutMode {
    if viewport_width < GUI_SEARCH_SPLIT_WIDTH {
        GuiSearchLayoutMode::SplitRows
    } else {
        GuiSearchLayoutMode::SingleRow
    }
}

pub(super) fn gui_tile_title_label(path: &Path, active: bool, save_status: &str) -> String {
    let file_name = gui_file_name_label(path);
    let label = if save_status == "saved" {
        file_name
    } else {
        format!("{file_name} | {save_status}")
    };

    if active {
        format!("active | {label}")
    } else {
        label
    }
}

pub(super) fn gui_tile_title_controls_attached(_active: bool) -> bool {
    true
}

pub(super) fn gui_tile_border_color(palette: iced::theme::Palette, active: bool) -> Color {
    if active {
        palette.primary
    } else {
        Color {
            a: 0.55,
            ..palette.primary
        }
    }
}
