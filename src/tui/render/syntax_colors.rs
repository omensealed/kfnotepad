//! Syntax-role color mapping and terminal palette adaptation.

use super::*;

mod hue;
mod palettes;
mod roles;

use hue::terminal_rgb_hue_degrees;
use palettes::terminal_syntax_role_rgb;
use roles::{terminal_syntax_color_role, TerminalSyntaxColorRole};

pub(crate) fn syntax_color_to_terminal(color: SyntaxColor, theme_id: EditorThemeId) -> Color {
    let (r, g, b) = terminal_syntax_role_rgb(
        theme_id,
        terminal_syntax_color_role(color.r, color.g, color.b),
    );
    Color::Rgb { r, g, b }
}
