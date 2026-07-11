include!("syntax_colors/roles.rs");
include!("syntax_colors/hue.rs");
include!("syntax_colors/palettes.rs");

pub(crate) fn syntect_color_to_terminal(
    color: syntect::highlighting::Color,
    theme_id: EditorThemeId,
) -> Color {
    let (r, g, b) = terminal_syntax_role_rgb(
        theme_id,
        terminal_syntax_color_role(color.r, color.g, color.b),
    );
    Color::Rgb { r, g, b }
}
