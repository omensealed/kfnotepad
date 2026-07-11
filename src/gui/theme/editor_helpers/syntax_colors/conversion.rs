pub(super) fn gui_syntax_segments_from_syntect(
    segments: Vec<(SyntectStyle, String)>,
    theme_id: EditorThemeId,
) -> Vec<GuiEditorSyntaxSegment> {
    segments
        .into_iter()
        .filter_map(|(style, text)| {
            (!text.is_empty()).then_some(GuiEditorSyntaxSegment {
                text,
                color: gui_color_from_syntect(style.foreground, theme_id),
            })
        })
        .collect()
}

pub(super) fn gui_color_from_syntect(
    color: syntect::highlighting::Color,
    theme_id: EditorThemeId,
) -> Color {
    let (r, g, b) = gui_syntax_rgb_for_theme(color.r, color.g, color.b, theme_id);
    Color::from_rgba8(r, g, b, f32::from(color.a) / 255.0)
}

pub(super) fn gui_syntax_rgb_for_theme(
    red: u8,
    green: u8,
    blue: u8,
    theme_id: EditorThemeId,
) -> (u8, u8, u8) {
    let role = gui_syntax_color_role(red, green, blue);
    let rgb = gui_syntax_role_rgb(theme_id, role);
    gui_ensure_syntax_contrast_rgb(rgb, gui_theme_palette(theme_id).background)
}
