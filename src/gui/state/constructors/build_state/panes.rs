fn build_workspace_and_pane_states(
    mut documents: Vec<TextDocument>,
) -> (GuiWorkspace, Vec<GuiPane>) {
    let first_document = documents.remove(0);
    let mut workspace = GuiWorkspace::from_document(first_document);
    let mut pane_states = vec![GuiPane::new(
        workspace.active,
        text_editor::Content::with_text(&workspace.active_tile().document.buffer.to_text()),
    )];
    for document in documents {
        let editor = text_editor::Content::with_text(&document.buffer.to_text());
        let tile_id = workspace.open_tile(document);
        pane_states.push(GuiPane::new(tile_id, editor));
    }
    (workspace, pane_states)
}

fn build_gui_panes(
    workspace: &mut GuiWorkspace,
    pane_states: Vec<GuiPane>,
    restored_layout: Option<GuiLayout>,
    project_active_ordinal: Option<usize>,
    status_messages: &mut Vec<String>,
) -> GuiPaneBuild {
    let (panes, mut active_pane) = if let Some(layout) = restored_layout.as_ref() {
        let (panes, pane) = panes_from_gui_layout(layout.clone(), pane_states);
        for ordinal in &layout.minimized_ordinals {
            if let Some(tile_id) = workspace.tiles.get(*ordinal).map(|tile| tile.id) {
                workspace.set_tile_minimized(tile_id, true);
            }
        }
        status_messages.push("restored GUI layout".to_string());
        (panes, pane)
    } else {
        default_panes(pane_states)
    };

    let (panes, minimized_panes, active_pane_after_minimize) =
        close_minimized_panes_into_tray(panes, workspace, active_pane);
    active_pane = active_pane_after_minimize;
    if let Some(active) = pane_for_tile_id(&panes, workspace.active) {
        active_pane = active;
    }
    if let Some(active_tile_id) = panes.get(active_pane).map(|pane| pane.tile_id) {
        workspace.focus_tile(active_tile_id);
    }
    if let Some(active_ordinal) = project_active_ordinal {
        if let Some(active_tile_id) = workspace.tiles.get(active_ordinal).map(|tile| tile.id) {
            workspace.focus_tile(active_tile_id);
            if let Some(pane) = pane_for_tile_id(&panes, active_tile_id) {
                active_pane = pane;
            }
        }
    }

    GuiPaneBuild {
        panes,
        minimized_panes,
        active_pane,
    }
}
