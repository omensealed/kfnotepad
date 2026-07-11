fn gui_path_prompt_panel(state: &KfnotepadGui) -> Option<Element<'_, Message>> {
    let palette = gui_theme_palette(state.settings.theme_id);
    state.path_prompt.map(|prompt| {
        container(
            row![
                text(gui_path_prompt_label(prompt)).size(gui_ui_text_size(state.settings)),
                text_input("Path", &state.path_prompt_value)
                    .on_input(Message::PathPromptChanged)
                    .on_submit(Message::SubmitPathPrompt)
                    .size(gui_ui_text_size(state.settings))
                    .style(move |_theme, status| gui_text_input_style(palette, status))
                    .width(Length::Fill),
                gui_tooltip_button(
                    LABEL_GO,
                    Message::SubmitPathPrompt,
                    "Apply path prompt",
                    state.settings,
                ),
                gui_tooltip_button(
                    LABEL_CANCEL,
                    Message::CancelPathPrompt,
                    "Cancel path prompt",
                    state.settings,
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(8)
        .into()
    })
}

fn gui_notes_panel(state: &KfnotepadGui) -> Option<Element<'_, Message>> {
    state.notes_panel.as_ref().map(|notes| {
        let mut items = row![text("Notes").size(gui_ui_text_size(state.settings))]
            .spacing(8)
            .align_y(Alignment::Center);
        if notes.is_empty() {
            items = items.push(text("No managed notes").size(gui_ui_text_size(state.settings)));
        } else {
            for (index, note) in notes.iter().enumerate() {
                let delete_tooltip =
                    if state.pending_managed_note_delete.as_deref() == Some(note.path.as_path()) {
                        format!("Confirm delete {}", note.file_name)
                    } else {
                        format!("Delete {}", note.file_name)
                    };
                items = items.push(
                    row![
                        button(text(&note.file_name).size(gui_ui_text_size(state.settings)))
                            .on_press(Message::ManagedNoteClicked(index)),
                        gui_icon_tooltip_button(
                            ICON_DELETE,
                            LABEL_DELETE,
                            Message::ManagedNoteDeleteClicked(index),
                            delete_tooltip,
                            state.settings,
                        ),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center),
                );
            }
        }
        items = items.push(gui_tooltip_button(
            LABEL_CANCEL,
            Message::CancelPathPrompt,
            "Close notes panel",
            state.settings,
        ));
        container(items).width(Length::Fill).padding(8).into()
    })
}
