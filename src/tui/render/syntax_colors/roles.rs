#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalSyntaxColorRole {
    Text,
    Comment,
    Rose,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}

fn terminal_syntax_color_role(red: u8, green: u8, blue: u8) -> TerminalSyntaxColorRole {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let chroma = max.saturating_sub(min);
    let luminance = 0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue);

    if chroma < 24 {
        return if luminance < 150.0 {
            TerminalSyntaxColorRole::Comment
        } else {
            TerminalSyntaxColorRole::Text
        };
    }

    let hue = terminal_rgb_hue_degrees(red, green, blue);
    if !(25.0..345.0).contains(&hue) {
        TerminalSyntaxColorRole::Rose
    } else if hue < 55.0 {
        TerminalSyntaxColorRole::Orange
    } else if hue < 78.0 {
        TerminalSyntaxColorRole::Yellow
    } else if hue < 160.0 {
        TerminalSyntaxColorRole::Green
    } else if hue < 200.0 {
        TerminalSyntaxColorRole::Cyan
    } else if hue < 255.0 {
        TerminalSyntaxColorRole::Blue
    } else if hue < 315.0 {
        TerminalSyntaxColorRole::Purple
    } else {
        TerminalSyntaxColorRole::Rose
    }
}
