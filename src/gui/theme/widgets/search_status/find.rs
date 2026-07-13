use super::*;

pub(super) fn gui_find_controls<'a>(
    state: &'a KfnotepadGui,
    field_width: f32,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(state.settings.theme_id);
    let input = text_input("Find", &state.search_query)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchNext)
        .size(gui_ui_text_size(state.settings))
        .style(move |_theme, status| gui_text_input_style(palette, status))
        .width(Length::Fixed(field_width));
    let mut input_stack = iced::widget::column![input]
        .spacing(2)
        .width(Length::Fixed(field_width));
    if state.search_history_open
        && state.search_query.is_empty()
        && !state.search_history.is_empty()
    {
        let mut history = iced::widget::column![].spacing(1).width(Length::Fill);
        for query in state.search_history.iter().take(GUI_FIND_HISTORY_LIMIT) {
            let query_for_message = query.clone();
            history = history.push(
                button(
                    text(query)
                        .size(gui_ui_small_text_size(state.settings))
                        .width(Length::Fill),
                )
                .width(Length::Fill)
                .padding([2, 5])
                .style(move |_theme, status| gui_menu_item_button_style(palette, status))
                .on_press(Message::SearchHistorySelected(query_for_message)),
            );
        }
        input_stack = input_stack.push(
            container(history)
                .width(Length::Fill)
                .padding(3)
                .style(move |_theme| gui_find_history_style(palette)),
        );
    }

    row![
        input_stack,
        gui_icon_tooltip_button(
            ICON_CASE_SENSITIVE,
            LABEL_CASE_SENSITIVE,
            Message::SearchCaseSensitiveChanged(!state.settings.search_case_sensitive),
            if state.settings.search_case_sensitive {
                "Case-sensitive search on"
            } else {
                "Case-sensitive search off"
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FIND_PREVIOUS,
            LABEL_FIND_PREVIOUS,
            Message::SearchPrevious,
            LABEL_FIND_PREVIOUS,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_FIND_NEXT,
            LABEL_FIND_NEXT,
            Message::SearchNext,
            LABEL_FIND_NEXT,
            state.settings
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn gui_find_history_style(palette: iced::theme::Palette) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: Some(palette.background.into()),
        border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..container::Style::default()
    }
}
