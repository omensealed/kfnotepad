pub(super) fn gui_mix_syntax_rgb(
    red: u8,
    green: u8,
    blue: u8,
    target: (f32, f32, f32),
    mix: f32,
) -> (u8, u8, u8) {
    (
        (f32::from(red) * (1.0 - mix) + target.0 * mix).round() as u8,
        (f32::from(green) * (1.0 - mix) + target.1 * mix).round() as u8,
        (f32::from(blue) * (1.0 - mix) + target.2 * mix).round() as u8,
    )
}

pub(super) fn gui_ensure_syntax_contrast_rgb(
    mut rgb: (u8, u8, u8),
    background: Color,
) -> (u8, u8, u8) {
    let background_rgb = gui_color_to_rgb(background);
    if gui_contrast_ratio(rgb, background_rgb) >= 4.5 {
        return rgb;
    }

    let background_luminance = gui_relative_luminance(background_rgb);
    let target = if background_luminance > 0.5 {
        (28.0, 24.0, 36.0)
    } else {
        (238.0, 248.0, 255.0)
    };
    for _ in 0..8 {
        rgb = gui_mix_syntax_rgb(rgb.0, rgb.1, rgb.2, target, 0.25);
        if gui_contrast_ratio(rgb, background_rgb) >= 4.5 {
            break;
        }
    }
    rgb
}

pub(super) fn gui_color_to_rgb(color: Color) -> (u8, u8, u8) {
    (
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

pub(super) fn gui_contrast_ratio(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f32 {
    let foreground = gui_relative_luminance(foreground);
    let background = gui_relative_luminance(background);
    let lighter = foreground.max(background);
    let darker = foreground.min(background);
    (lighter + 0.05) / (darker + 0.05)
}

pub(super) fn gui_relative_luminance((red, green, blue): (u8, u8, u8)) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)
}
