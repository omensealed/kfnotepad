fn gui_workspaces_panel<'a>(
    state: &'a KfnotepadGui,
    panel_tabs: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut projects = widget::column![
        panel_tabs,
        text(LABEL_WORKSPACES).size(gui_ui_heading_text_size(state.settings)),
        row![
            gui_icon_tooltip_button(
                ICON_SAVE,
                LABEL_SAVE,
                Message::SaveCurrentWorkspaceProject,
                "Save current workspace project",
                state.settings,
            ),
            gui_icon_tooltip_button(
                ICON_REFRESH,
                "Refresh",
                Message::RefreshWorkspaceProjects,
                "Refresh saved workspace projects",
                state.settings,
            ),
        ]
        .spacing(6),
        text_input("Project name", &state.workspace_project_name)
            .on_input(Message::WorkspaceProjectNameChanged)
            .on_submit(Message::SaveNamedWorkspaceProject)
            .size(gui_ui_text_size(state.settings))
            .style(move |_theme, status| {
                gui_text_input_style(gui_theme_palette(state.settings.theme_id), status)
            }),
        gui_icon_tooltip_button(
            ICON_SAVE,
            LABEL_SAVE,
            Message::SaveNamedWorkspaceProject,
            "Save workspace project with this name",
            state.settings,
        ),
        checkbox(state.settings.gui_restore_last_workspace)
            .label("Restore last workspace")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| {
                gui_checkbox_style(gui_theme_palette(state.settings.theme_id), status)
            })
            .on_toggle(Message::RestoreLastWorkspaceChanged),
    ]
    .spacing(5);

    if state.workspace_projects.is_empty() {
        projects =
            projects.push(text("No saved workspace projects").size(gui_ui_text_size(state.settings)));
    } else {
        for (index, entry) in state.workspace_projects.iter().enumerate() {
            projects = projects.push(
                row![
                    button(
                        widget::column![
                            text(&entry.project.name).size(gui_ui_text_size(state.settings)),
                            text(format!("{} files", entry.project.files.len()))
                                .size(gui_ui_small_text_size(state.settings)),
                        ]
                        .spacing(2),
                    )
                    .width(Length::Fill)
                    .padding(6)
                    .on_press(Message::WorkspaceProjectClicked(index)),
                    gui_icon_tooltip_button(
                        ICON_NEW_WINDOW,
                        LABEL_OPEN,
                        Message::WorkspaceProjectNewWindowClicked(index),
                        "Open workspace project in a new window",
                        state.settings,
                    ),
                    gui_icon_tooltip_button(
                        ICON_DELETE,
                        LABEL_DELETE,
                        Message::WorkspaceProjectDeleteClicked(index),
                        "Delete workspace project",
                        state.settings,
                    ),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            );
        }
    }

    projects.into()
}
