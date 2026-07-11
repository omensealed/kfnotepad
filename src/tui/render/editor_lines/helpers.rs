fn write_editor_body_padding(writer: &mut impl Write, remaining: &mut usize) -> io::Result<()> {
    for _ in 0..EDITOR_BODY_PADDING {
        if *remaining == 0 {
            break;
        }
        queue!(writer, Print(" "))?;
        *remaining = remaining.saturating_sub(1);
    }
    Ok(())
}
