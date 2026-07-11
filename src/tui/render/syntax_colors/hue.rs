fn terminal_rgb_hue_degrees(red: u8, green: u8, blue: u8) -> f32 {
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

    if hue < 0.0 {
        hue + 360.0
    } else {
        hue
    }
}
