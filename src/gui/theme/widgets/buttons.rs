pub(super) fn gui_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    tooltip_text: impl Into<String>,
    position: iced::widget::tooltip::Position,
    settings: EditorSettings,
) -> Element<'a, Message> {
    iced::widget::tooltip(
        content,
        container(text(tooltip_text.into()).size(gui_ui_tooltip_text_size(settings)))
            .padding(8)
            .style(|_theme| container::Style {
                text_color: Some(color(226, 240, 255)),
                background: Some(color(3, 7, 18).into()),
                border: iced::Border {
                    color: color(102, 229, 255),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..container::Style::default()
            }),
        position,
    )
    .gap(6)
    .snap_within_viewport(true)
    .style(|_theme| container::Style {
        text_color: Some(color(226, 240, 255)),
        background: Some(color(3, 7, 18).into()),
        border: iced::Border {
            color: color(102, 229, 255),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

pub(super) fn gui_tooltip_button<'a>(
    label: impl Into<String>,
    message: Message,
    tooltip_text: impl Into<String>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    gui_tooltip(
        button(text(label.into()).size(gui_ui_text_size(settings)))
            .padding(GUI_CHROME_PADDING)
            .style(move |_theme, status| gui_chrome_button_style(palette, status))
            .on_press(message),
        tooltip_text,
        iced::widget::tooltip::Position::Bottom,
        settings,
    )
}

pub(super) fn gui_icon_font() -> Font {
    iced_fonts::NERD_FONT
}

pub(super) fn gui_centered_icon<'a>(
    icon: &'a str,
    settings: EditorSettings,
) -> Element<'a, Message> {
    container(
        text(gui_icon_only_label(icon))
            .font(gui_icon_font())
            .size(gui_ui_icon_text_size(settings))
            .line_height(GUI_ICON_LINE_HEIGHT)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Shrink)
            .height(Length::Shrink),
    )
    .width(Length::Shrink)
    .height(Length::Shrink)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

pub(super) fn gui_labeled_icon_button<'a>(
    icon: &'static str,
    _icon_label: &'a str,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let icon_color = text(gui_icon_only_label(icon))
        .font(gui_icon_font())
        .size(gui_ui_icon_text_size(settings))
        .line_height(GUI_ICON_LINE_HEIGHT)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    container(icon_color)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

pub(super) fn gui_icon_tooltip_button<'a>(
    icon: &'static str,
    icon_label: &'a str,
    message: Message,
    tooltip_text: impl Into<String>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    gui_tooltip(
        button(gui_labeled_icon_button(icon, icon_label, settings))
            .width(Length::Shrink)
            .padding(GUI_CHROME_PADDING)
            .style(move |_theme, status| gui_chrome_button_style(palette, status))
            .on_press(message),
        tooltip_text,
        iced::widget::tooltip::Position::Bottom,
        settings,
    )
}
