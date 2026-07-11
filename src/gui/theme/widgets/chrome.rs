pub(super) fn gui_header_action_row<'a>(state: &'a KfnotepadGui) -> Element<'a, Message> {
    row![
        gui_icon_tooltip_button(
            ICON_NEW_TILE,
            LABEL_NEW_TILE,
            Message::NewTileRequested,
            "Create a new tile (Ctrl-N)",
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FILES,
            LABEL_FILES,
            Message::ToggleBrowser,
            if state.left_panel.visible {
                format!("Hide {} panel (Ctrl-B)", state.left_panel.title())
            } else {
                format!("Show {} panel (Ctrl-B)", state.left_panel.title())
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_THEME,
            LABEL_THEME,
            Message::CycleTheme,
            format!("Cycle theme: {}", state.settings.theme_id.label()),
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_SYNTAX_THEME,
            LABEL_SYNTAX_THEME,
            Message::CycleSyntaxTheme,
            format!(
                "Cycle syntax theme: {} (Ctrl-Shift-T)",
                state.settings.syntax_theme_id.label()
            ),
            state.settings,
        ),
        gui_icon_tooltip_button(
            if state.settings.gui_reader_mode_enabled {
                ICON_READER_MODE_PAUSE
            } else {
                ICON_READER_MODE_PLAY
            },
            LABEL_READER_MODE,
            Message::MenuCommand(GuiMenuCommand::ToggleReaderMode),
            if state.settings.gui_reader_mode_enabled {
                format!(
                    "Stop reader mode (Ctrl-R), {} lines/min",
                    state.settings.gui_reader_lines_per_minute
                )
            } else {
                format!(
                    "Start reader mode (Ctrl-R), {} lines/min",
                    state.settings.gui_reader_lines_per_minute
                )
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_SAVE,
            LABEL_SAVE,
            Message::SaveRequested,
            "Save active tile (Ctrl-S)",
            state.settings,
        ),
    ]
    .spacing(GUI_HEADER_ACTION_SPACING)
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn gui_minimized_tray<'a>(state: &'a KfnotepadGui) -> Option<Element<'a, Message>> {
    let items = state.minimized_tray_items();
    if items.is_empty() {
        return None;
    }

    let palette = gui_theme_palette(state.settings.theme_id);
    let mut tray = row![text("Minimized").size(gui_ui_small_text_size(state.settings))]
        .spacing(6)
        .align_y(Alignment::Center);

    for item in items {
        let tooltip = format!("Restore {}", item.tooltip);
        tray = tray.push(gui_tooltip(
            button(text(item.title).size(gui_ui_small_text_size(state.settings)))
                .padding(GUI_CHROME_PADDING)
                .on_press(Message::RestoreMinimizedTile(item.tile_id)),
            tooltip,
            iced::widget::tooltip::Position::Bottom,
            state.settings,
        ));
    }

    Some(
        container(tray)
            .width(Length::Fill)
            .padding([2, 4])
            .style(move |_theme| container::Style {
                text_color: Some(palette.text),
                background: Some(palette.background.into()),
                border: iced::Border {
                    color: Color {
                        a: 0.55,
                        ..palette.primary
                    },
                    width: 1.0,
                    radius: GUI_TILE_RADIUS.into(),
                },
                ..container::Style::default()
            })
            .into(),
    )
}
