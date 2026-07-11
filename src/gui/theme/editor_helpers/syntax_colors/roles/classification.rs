fn gui_syntax_color_role(red: u8, green: u8, blue: u8) -> GuiSyntaxColorRole {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let chroma = max.saturating_sub(min);
    let luminance = 0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue);

    if chroma < 24 {
        return if luminance < 150.0 {
            GuiSyntaxColorRole::Comment
        } else {
            GuiSyntaxColorRole::Text
        };
    }

    let hue = gui_rgb_hue_degrees(red, green, blue);
    if !(25.0..345.0).contains(&hue) {
        GuiSyntaxColorRole::Rose
    } else if hue < 55.0 {
        GuiSyntaxColorRole::Orange
    } else if hue < 78.0 {
        GuiSyntaxColorRole::Yellow
    } else if hue < 160.0 {
        GuiSyntaxColorRole::Green
    } else if hue < 200.0 {
        GuiSyntaxColorRole::Cyan
    } else if hue < 255.0 {
        GuiSyntaxColorRole::Blue
    } else if hue < 315.0 {
        GuiSyntaxColorRole::Purple
    } else {
        GuiSyntaxColorRole::Rose
    }
}

fn gui_rgb_hue_degrees(red: u8, green: u8, blue: u8) -> f32 {
    let red = f32::from(red) / 255.0;
    let green = f32::from(green) / 255.0;
    let blue = f32::from(blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;

    if delta == 0.0 {
        return 0.0;
    }

    let hue = if max == red {
        60.0 * (((green - blue) / delta) % 6.0)
    } else if max == green {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    if hue < 0.0 { hue + 360.0 } else { hue }
}
