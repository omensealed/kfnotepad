impl BufferSnapshot {
    fn from_buffer(buffer: &TextBuffer) -> Self {
        Self {
            lines: buffer.lines.clone(),
            trailing_newline: buffer.trailing_newline,
            byte_size: buffer_bytes(&buffer.lines, buffer.trailing_newline),
        }
    }
}

fn buffer_bytes(lines: &[String], trailing_newline: bool) -> usize {
    let line_bytes: usize = lines.iter().map(String::len).sum();
    let newline_bytes = lines.len().saturating_sub(1);
    let trailing_newline_byte = usize::from(trailing_newline);
    line_bytes + newline_bytes + trailing_newline_byte
}

pub(crate) fn trim_undo_history(
    history: &mut Vec<BufferSnapshot>,
    max_entries: usize,
    max_bytes: usize,
) {
    while history.len() > max_entries {
        let _ = history.remove(0);
    }

    let mut used_bytes: usize = history.iter().map(|snapshot| snapshot.byte_size).sum();
    while used_bytes > max_bytes && !history.is_empty() {
        let removed = history.remove(0);
        used_bytes = used_bytes.saturating_sub(removed.byte_size);
    }
}
