pub(crate) fn write_help_line(
    writer: &mut impl Write,
    screen_row: u16,
    frame: RenderFrame,
) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    let help = compose_help_line(frame.terminal_width);
    queue!(
        writer,
        frame.move_to(0, screen_row),
        Clear(ClearType::CurrentLine),
    )?;
    queue_set_foreground_color(writer, &frame, frame.theme.help_fg)?;
    queue_set_background_color(writer, &frame, frame.theme.help_bg)?;
    print_truncated(writer, &help, &mut remaining)?;
    queue!(writer, ResetColor)
}

pub(crate) fn compose_help_line(width: usize) -> String {
    let controls = [
        "F2 Command",
        "F10 Menu/Help",
        "Ctrl-S Save",
        "Ctrl-B Files",
        "Ctrl-Q Quit",
    ];
    let width = width.max(1);
    let mut line = String::new();
    for control in controls {
        let candidate = if line.is_empty() {
            format!(" {control}")
        } else {
            format!("{line} | {control}")
        };
        if text_display_width(&candidate) > width {
            break;
        }
        line = candidate;
    }
    if text_display_width(&line) < width {
        line.push(' ');
    }
    line
}
