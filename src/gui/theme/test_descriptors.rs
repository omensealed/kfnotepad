use super::*;

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_action_descriptors() -> Vec<GuiActionDescriptor> {
    vec![
        GuiActionDescriptor {
            label: LABEL_NEW_TILE,
            shortcut: Some("Ctrl-N"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_OPEN,
            shortcut: Some("Ctrl-O"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_SAVE,
            shortcut: Some("Ctrl-S"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_SAVE_AS,
            shortcut: Some("Ctrl-Shift-S"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_CLOSE_TILE,
            shortcut: Some("Ctrl-F4"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_QUIT,
            shortcut: Some("Ctrl-Q"),
            menu_group: Some(GuiMenuGroup::File),
        },
        GuiActionDescriptor {
            label: LABEL_FIND_NEXT,
            shortcut: Some("F3"),
            menu_group: Some(GuiMenuGroup::Edit),
        },
        GuiActionDescriptor {
            label: LABEL_FIND_PREVIOUS,
            shortcut: Some("Shift-F3"),
            menu_group: Some(GuiMenuGroup::Edit),
        },
        GuiActionDescriptor {
            label: LABEL_FILES,
            shortcut: Some("Ctrl-B"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_THEME,
            shortcut: Some("Ctrl-T"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_SYNTAX_THEME,
            shortcut: Some("Ctrl-Shift-T"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_READER_MODE,
            shortcut: Some("Ctrl-R"),
            menu_group: Some(GuiMenuGroup::View),
        },
        GuiActionDescriptor {
            label: LABEL_GO_TO_LINE,
            shortcut: Some("Ctrl-G"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: LABEL_DOCUMENT_START,
            shortcut: Some("Ctrl-Home"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: LABEL_DOCUMENT_END,
            shortcut: Some("Ctrl-End"),
            menu_group: Some(GuiMenuGroup::Go),
        },
        GuiActionDescriptor {
            label: "Scroll viewport up",
            shortcut: Some("Ctrl-PageUp"),
            menu_group: None,
        },
        GuiActionDescriptor {
            label: "Scroll viewport down",
            shortcut: Some("Ctrl-PageDown"),
            menu_group: None,
        },
        GuiActionDescriptor {
            label: LABEL_OPEN_NOTE,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Notes),
        },
        GuiActionDescriptor {
            label: LABEL_LIST_NOTES,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Notes),
        },
        GuiActionDescriptor {
            label: LABEL_MINIMIZE,
            shortcut: Some("Ctrl-M"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MAXIMIZE,
            shortcut: Some("Ctrl-Shift-M"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_LEFT,
            shortcut: Some("Ctrl-Shift-Left"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_RIGHT,
            shortcut: Some("Ctrl-Shift-Right"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_UP,
            shortcut: Some("Ctrl-Shift-Up"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_MOVE_DOWN,
            shortcut: Some("Ctrl-Shift-Down"),
            menu_group: Some(GuiMenuGroup::Tile),
        },
        GuiActionDescriptor {
            label: LABEL_OPEN_HELP,
            shortcut: None,
            menu_group: Some(GuiMenuGroup::Help),
        },
    ]
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_focus_order_descriptors(
    browser_visible: bool,
    tile_minimized: bool,
) -> Vec<GuiFocusStep> {
    let mut steps = gui_menu_groups()
        .into_iter()
        .map(|group| GuiFocusStep {
            area: "menu",
            label: gui_menu_group_label(group),
            keyboard: None,
        })
        .collect::<Vec<_>>();

    steps.extend([
        GuiFocusStep {
            area: "header",
            label: LABEL_NEW_TILE,
            keyboard: Some("Ctrl-N"),
        },
        GuiFocusStep {
            area: "header",
            label: if browser_visible {
                "Hide Files"
            } else {
                "Show Files"
            },
            keyboard: Some("Ctrl-B"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_THEME,
            keyboard: Some("Ctrl-T"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_SYNTAX_THEME,
            keyboard: Some("Ctrl-Shift-T"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_READER_MODE,
            keyboard: Some("Ctrl-R"),
        },
        GuiFocusStep {
            area: "header",
            label: LABEL_SAVE,
            keyboard: Some("Ctrl-S"),
        },
    ]);

    if browser_visible {
        steps.extend([
            GuiFocusStep {
                area: "file browser",
                label: "Parent directory",
                keyboard: None,
            },
            GuiFocusStep {
                area: "file browser",
                label: LABEL_REFRESH,
                keyboard: None,
            },
            GuiFocusStep {
                area: "file browser",
                label: LABEL_CREATE_FILE,
                keyboard: None,
            },
        ]);
        steps.push(GuiFocusStep {
            area: "file browser",
            label: "File browser entries",
            keyboard: None,
        });
    }

    steps.extend([
        GuiFocusStep {
            area: "search",
            label: "Find field",
            keyboard: Some("Ctrl-F"),
        },
        GuiFocusStep {
            area: "search",
            label: "Case-sensitive search",
            keyboard: None,
        },
        GuiFocusStep {
            area: "search",
            label: LABEL_FIND_PREVIOUS,
            keyboard: Some("Shift-F3"),
        },
        GuiFocusStep {
            area: "search",
            label: LABEL_FIND_NEXT,
            keyboard: Some("F3"),
        },
        GuiFocusStep {
            area: "navigation",
            label: "Line field",
            keyboard: Some("Ctrl-G"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_GO,
            keyboard: Some("Enter in line field"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_DOCUMENT_START,
            keyboard: Some("Ctrl-Home"),
        },
        GuiFocusStep {
            area: "navigation",
            label: LABEL_DOCUMENT_END,
            keyboard: Some("Ctrl-End"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_LEFT,
            keyboard: Some("Ctrl-Shift-Left"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_RIGHT,
            keyboard: Some("Ctrl-Shift-Right"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_UP,
            keyboard: Some("Ctrl-Shift-Up"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MOVE_DOWN,
            keyboard: Some("Ctrl-Shift-Down"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: if tile_minimized {
                LABEL_RESTORE
            } else {
                LABEL_MINIMIZE
            },
            keyboard: Some("Ctrl-M"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_MAXIMIZE,
            keyboard: Some("Ctrl-Shift-M"),
        },
        GuiFocusStep {
            area: "tile controls",
            label: LABEL_CLOSE_TILE,
            keyboard: Some("Ctrl-F4"),
        },
    ]);

    if !tile_minimized {
        steps.push(GuiFocusStep {
            area: "editor",
            label: "Active editor",
            keyboard: None,
        });
    }

    steps
}
