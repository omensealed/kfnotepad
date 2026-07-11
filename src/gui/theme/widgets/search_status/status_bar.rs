pub(super) fn gui_status_bar<'a>(
    status_message: &'a str,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    text(status_message)
        .size(gui_ui_small_text_size(settings))
        .color(palette.text)
        .into()
}
