fn gui_files_panel<'a>(
    state: &'a KfnotepadGui,
    panel_tabs: Element<'a, Message>,
) -> Element<'a, Message> {
    match &state.browser {
        Some(browser) => {
            let current_dir = &browser.sidebar.current_dir;
            let tree_view = gui_file_tree_view(&state.browser_tree_rows, state.settings);
            widget::column![
                panel_tabs,
                row![
                    text(LABEL_FILES).size(gui_ui_heading_text_size(state.settings)),
                    text(format!("{:.0}px", state.browser_width))
                        .size(gui_ui_small_text_size(state.settings)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                gui_tooltip(
                    text(gui_sidebar_path_label(current_dir))
                        .size(gui_ui_small_text_size(state.settings)),
                    current_dir.display().to_string(),
                    iced::widget::tooltip::Position::Bottom,
                    state.settings,
                ),
                slider(
                    GUI_BROWSER_WIDTH_MIN..=GUI_BROWSER_WIDTH_MAX,
                    state.browser_width,
                    Message::BrowserWidthChanged,
                )
                .step(10.0_f32),
                row![
                    gui_icon_tooltip_button(
                        ICON_PARENT_DIR,
                        "Up",
                        Message::BrowserParentRequested,
                        "Go to parent directory",
                        state.settings,
                    ),
                    gui_icon_tooltip_button(
                        ICON_REFRESH,
                        "Refresh",
                        Message::BrowserRefreshRequested,
                        "Refresh file browser",
                        state.settings,
                    ),
                    gui_icon_tooltip_button(
                        ICON_CREATE_FILE,
                        LABEL_CREATE_FILE,
                        Message::BrowserCreateFileRequested,
                        format!("{LABEL_CREATE_FILE} in selected folder"),
                        state.settings,
                    ),
                    gui_icon_tooltip_button(
                        ICON_CREATE_DIRECTORY,
                        LABEL_CREATE_DIRECTORY,
                        Message::BrowserCreateDirectoryRequested,
                        "Create folder in selected folder",
                        state.settings,
                    ),
                    gui_icon_tooltip_button(
                        ICON_DELETE,
                        LABEL_DELETE,
                        Message::BrowserDeleteSelectedRequested,
                        "Delete selected file or folder",
                        state.settings,
                    ),
                ]
                .spacing(GUI_PANEL_CONTROL_SPACING)
                .align_y(Alignment::Center),
                container(tree_view).padding(iced::Padding {
                    top: GUI_PANEL_TREE_TOP_PADDING,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
            ]
            .spacing(GUI_PANEL_SECTION_SPACING)
            .into()
        }
        None => widget::column![
            panel_tabs,
            text(LABEL_FILES).size(gui_ui_heading_text_size(state.settings)),
            text("file browser unavailable").size(gui_ui_text_size(state.settings)),
        ]
        .into(),
    }
}
