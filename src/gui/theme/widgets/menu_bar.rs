use super::*;

pub(in crate::gui::app::state) fn gui_menu_command_item<'a>(
    item: GuiMenuItem,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> iced_aw::menu::Item<'a, Message, Theme, iced::Renderer> {
    iced_aw::menu::Item::new(
        button(text(item.label).size(gui_ui_text_size(settings)))
            .width(Length::Fill)
            .padding(GUI_MENU_ITEM_PADDING)
            .style(move |_theme, status| gui_menu_item_button_style(palette, status))
            .on_press(Message::MenuCommand(item.command)),
    )
    .close_on_click(true)
}

pub(in crate::gui::app::state) fn gui_menu_dropdown<'a>(
    group: GuiMenuGroup,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> Menu<'a, Message, Theme, iced::Renderer> {
    let items = gui_menu_items(group)
        .into_iter()
        .map(|item| gui_menu_command_item(item, palette, settings))
        .collect();

    Menu::new(items)
        .width(Length::Fixed(GUI_MENU_DROPDOWN_WIDTH))
        .spacing(3)
        .padding(7)
        .close_on_item_click(true)
        .close_on_background_click(true)
}

pub(in crate::gui::app::state) fn gui_menu_root_item<'a>(
    group: GuiMenuGroup,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> iced_aw::menu::Item<'a, Message, Theme, iced::Renderer> {
    // iced_aw's menu tree expects roots to be Item::with_menu and commands to be
    // regular widgets inside Item::new. Keep the current shallow command groups
    // flat until nested hover submenus have a clear UX win.
    iced_aw::menu::Item::with_menu(
        container(
            text(gui_menu_group_chrome_label(group))
                .size(gui_ui_text_size(settings))
                .color(palette.text),
        )
        .height(Length::Fixed(GUI_MENU_ROOT_HEIGHT))
        .padding([
            GUI_MENU_ROOT_VERTICAL_PADDING,
            GUI_MENU_ROOT_HORIZONTAL_PADDING,
        ])
        .style(move |_theme| gui_menu_root_style(palette)),
        gui_menu_dropdown(group, palette, settings),
    )
}

pub(in crate::gui::app::state) fn gui_menu_bar<'a>(
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let roots = gui_menu_groups()
        .into_iter()
        .map(|group| gui_menu_root_item(group, palette, settings))
        .collect();

    MenuBar::new(roots)
        .spacing(GUI_MENU_BAR_SPACING)
        .padding(0)
        .draw_path(menu::DrawPath::Backdrop)
        .close_on_item_click_global(true)
        .close_on_background_click_global(true)
        .style(move |_theme, _status| gui_menu_panel_style(palette))
        .into()
}
