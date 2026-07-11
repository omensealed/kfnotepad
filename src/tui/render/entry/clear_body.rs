fn clear_editor_body_row(
    writer: &mut impl Write,
    screen_row: u16,
    frame: RenderFrame,
) -> io::Result<()> {
    queue!(
        writer,
        frame.move_to(0, screen_row),
        Clear(ClearType::CurrentLine)
    )
}
