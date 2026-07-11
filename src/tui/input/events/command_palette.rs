pub(crate) fn open_menu(runtime: &mut EditorRuntime) {
    runtime.command_palette = None;
    runtime.menu = Some(MenuState::default());
    runtime.status = String::from("Menu: File");
}

pub(crate) fn open_command_palette(runtime: &mut EditorRuntime) {
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.workspace_prompt = None;
    runtime.workspace_manager = None;
    runtime.sidebar_prompt = None;
    runtime.command_palette = Some(CommandPaletteState::default());
    runtime.status = String::from("Command palette");
}

pub(crate) fn command_palette_candidates(query: &str) -> Vec<CommandPaletteEntry> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .collect();
    let mut entries = Vec::new();

    for group in MENU_GROUPS {
        for item in group.items() {
            if item.command == MenuCommand::HelpOnly {
                continue;
            }
            let searchable = format!(
                "{} {} {}",
                group.label(),
                item.label,
                item.shortcut.unwrap_or("")
            )
            .to_ascii_lowercase();
            if tokens.iter().all(|token| searchable.contains(token)) {
                entries.push(CommandPaletteEntry {
                    group,
                    label: item.label,
                    shortcut: item.shortcut,
                    command: item.command,
                });
            }
        }
    }

    entries
}

pub(crate) fn selected_command_palette_entry(
    runtime: &EditorRuntime,
) -> Option<CommandPaletteEntry> {
    let palette = runtime.command_palette.as_ref()?;
    command_palette_candidates(&palette.query)
        .get(palette.selected)
        .copied()
}

pub(crate) fn move_command_palette_selection(runtime: &mut EditorRuntime, delta: isize) {
    let Some(palette) = runtime.command_palette.as_ref() else {
        return;
    };
    let len = command_palette_candidates(&palette.query).len();
    if len == 0 {
        set_command_palette_selection(runtime, 0);
        return;
    }
    let current = palette.selected.min(len.saturating_sub(1));
    let next = current
        .saturating_add_signed(delta)
        .min(len.saturating_sub(1));
    set_command_palette_selection(runtime, next);
}

pub(crate) fn set_command_palette_selection(runtime: &mut EditorRuntime, selected: usize) {
    let Some(palette) = runtime.command_palette.as_mut() else {
        return;
    };
    let len = command_palette_candidates(&palette.query).len();
    palette.selected = if len == 0 {
        0
    } else {
        selected.min(len.saturating_sub(1))
    };
    let visible_rows = 8usize;
    if palette.selected < palette.scroll {
        palette.scroll = palette.selected;
    } else if palette.selected >= palette.scroll.saturating_add(visible_rows) {
        palette.scroll = palette.selected.saturating_sub(visible_rows - 1);
    }
    runtime.status = palette_status(palette, len);
}

pub(crate) fn normalize_command_palette_selection(runtime: &mut EditorRuntime) {
    set_command_palette_selection(runtime, 0);
}

pub(crate) fn palette_status(palette: &CommandPaletteState, len: usize) -> String {
    if palette.query.is_empty() {
        format!("Command palette: {len} commands")
    } else {
        format!("Command palette: {} match(es)", len)
    }
}
