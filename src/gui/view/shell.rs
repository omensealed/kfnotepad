//! Responsive root window composition for the complete GUI surface.

use super::*;

pub(in crate::gui::app::state) fn view(state: &KfnotepadGui) -> Element<'_, Message> {
    responsive(move |viewport_size| view_sized(state, viewport_size)).into()
}

fn view_sized(state: &KfnotepadGui, viewport_size: Size) -> Element<'_, Message> {
    let header = gui_view_header(state, viewport_size);
    let path_prompt = gui_path_prompt_panel(state);
    let notes_panel = gui_notes_panel(state);
    let startup_help = startup_help_panel(state);
    let sidebar = gui_left_panel_view(state);
    let search_controls = gui_search_controls(state, viewport_size.width);
    let panes = gui_pane_grid_view(state);

    let editor = container(
        widget::column![
            search_controls,
            panes,
            gui_status_bar(&state.status_message, state.settings),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(GUI_EDITOR_PADDING);

    let minimized_tray = gui_minimized_tray(state);
    let mut body = widget::column![].spacing(8);
    if let Some(startup_help) = startup_help {
        body = body.push(startup_help);
    }
    if let Some(path_prompt) = path_prompt {
        body = body.push(path_prompt);
    }
    if let Some(notes_panel) = notes_panel {
        body = body.push(notes_panel);
    }
    let content = if let Some(sidebar) = sidebar {
        row![sidebar, editor].height(Length::Fill)
    } else {
        row![editor].height(Length::Fill)
    };
    body = body.push(content);

    let mut root = widget::column![header].spacing(8);
    if let Some(minimized_tray) = minimized_tray {
        root = root.push(minimized_tray);
    }
    root = root.push(body);
    container(root)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(GUI_ROOT_PADDING)
        .into()
}
