pub(crate) fn tab_strip_height_for_width(tab_strip: &[TabStripItem], terminal_width: usize) -> u16 {
    if tab_strip.len() <= 1 {
        return 1;
    }
    1 + tab_strip_rows_for_width(tab_strip, terminal_width) as u16
}

fn tab_strip_rows_for_width(tab_strip: &[TabStripItem], terminal_width: usize) -> usize {
    let width = terminal_width.max(1);
    let mut rows = 1usize;
    let mut remaining = width;
    for (index, item) in tab_strip.iter().enumerate() {
        let label_width = text_display_width(&compose_tab_label(index, item)).max(1);
        if label_width > remaining && remaining < width {
            rows += 1;
            remaining = width;
        }
        remaining = remaining.saturating_sub(label_width.min(remaining));
    }
    rows
}

pub(crate) fn write_tab_strip(
    writer: &mut impl Write,
    tab_strip: &[TabStripItem],
    frame: RenderFrame,
) -> io::Result<()> {
    let mut row = 1u16;
    let mut remaining = frame.terminal_width;
    clear_tab_strip_rows(writer, frame)?;
    for (index, item) in tab_strip.iter().enumerate() {
        if row >= frame.body_top {
            break;
        }
        let label = compose_tab_label(index, item);
        let label_width = text_display_width(&label).max(1);
        if label_width > remaining && remaining < frame.terminal_width {
            row += 1;
            remaining = frame.terminal_width;
        }
        if row >= frame.body_top {
            break;
        }
        queue!(
            writer,
            frame.move_to(frame.terminal_width.saturating_sub(remaining) as u16, row),
        )?;
        if item.active {
            queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
            queue!(writer, SetAttribute(Attribute::Bold))?;
        } else {
            queue_set_foreground_color(writer, &frame, frame.theme.help_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.help_bg)?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
        }
        print_truncated(writer, &label, &mut remaining)?;
    }
    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)
}

fn clear_tab_strip_rows(writer: &mut impl Write, frame: RenderFrame) -> io::Result<()> {
    for row in 1..frame.body_top {
        queue!(writer, frame.move_to(0, row), Clear(ClearType::CurrentLine),)?;
        queue_set_foreground_color(writer, &frame, frame.theme.help_fg)?;
        queue_set_background_color(writer, &frame, frame.theme.help_bg)?;
    }
    Ok(())
}
