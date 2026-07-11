fn startup_help_panel(state: &KfnotepadGui) -> Option<Element<'_, Message>> {
    if !state.show_startup_help_panel {
        return None;
    }
    let palette = gui_theme_palette(state.settings.theme_id);
    let heading = text("Welcome to kfnotepad")
        .size(gui_ui_heading_text_size(state.settings))
        .color(palette.primary);
    let body = widget::column![
        text("No file was opened. Use these actions to get started quickly:")
            .size(gui_ui_text_size(state.settings)),
        text("• Ctrl+N creates a new untitled tile."),
        text("• Ctrl+O opens a local file."),
        text("• Ctrl-S saves the active tile."),
        text("• Ctrl-B toggles the Files sidebar (or open from View menu)."),
        text("• Ctrl-Shift-Arrow moves the active tile."),
        text("• F3/Shift-F3 search forward/backward in the active file."),
        text("• Ctrl-M and Ctrl-Shift-M minimize/maximize tiles."),
        text("• External file conflicts show a lock; use Unlock to continue editing."),
    ]
    .spacing(4)
    .width(Length::Fill);
    let actions = widget::column![
        row![
            gui_tooltip_button(
                "Open full help tile",
                Message::MenuCommand(GuiMenuCommand::OpenHelp),
                "Open the built-in help document",
                state.settings,
            ),
            gui_tooltip_button(
                LABEL_DISMISS_HELP,
                Message::DismissStartupHelp,
                "Hide this startup help panel",
                state.settings,
            ),
        ]
        .spacing(8),
        text("Press F1 at any time to open built-in help.").size(gui_ui_small_text_size(state.settings)),
    ]
    .spacing(8);

    Some(
        container(widget::column![heading, body, actions].spacing(10))
            .width(Length::Fill)
            .padding(10)
            .style(move |_theme| container::Style {
                text_color: Some(palette.text),
                background: Some(palette.background.into()),
                border: iced::Border {
                    color: palette.primary,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..container::Style::default()
            })
            .into(),
    )
}
