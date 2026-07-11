fn queue_set_foreground_color(
    writer: &mut impl Write,
    no_color: bool,
    color: Color,
) -> io::Result<()> {
    if no_color {
        return Ok(());
    }
    queue!(writer, SetForegroundColor(color))
}

fn queue_set_background_color(
    writer: &mut impl Write,
    no_color: bool,
    color: Color,
) -> io::Result<()> {
    if no_color {
        return Ok(());
    }
    queue!(writer, SetBackgroundColor(color))
}
