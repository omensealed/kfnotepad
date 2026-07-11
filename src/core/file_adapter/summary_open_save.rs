pub fn summarize_text(text: &str) -> FileSummary {
    FileSummary {
        bytes: text.len() as u64,
        lines: text.lines().count(),
        trailing_newline: text.ends_with('\n'),
    }
}

pub fn summarize_path(path: &Path) -> Result<FileSummary, String> {
    let text = read_text_file(path).map_err(|error| error.to_string())?;
    Ok(summarize_text(&text))
}

pub fn open_text_file(path: &Path) -> Result<TextDocument, OpenError> {
    let (text, snapshot) = read_text_file_with_snapshot(path)?;
    let mut buffer = TextBuffer::from_text(&text);
    buffer.set_file_snapshot(Some(snapshot));
    Ok(TextDocument {
        path: path.to_path_buf(),
        buffer,
    })
}

pub fn save_text_document(document: &mut TextDocument) -> Result<(), SaveError> {
    save_text_buffer_for_document(&document.path, &mut document.buffer)?;
    document.buffer.mark_clean();
    Ok(())
}

pub fn save_text_buffer(path: &Path, buffer: &TextBuffer) -> Result<(), SaveError> {
    save_text_buffer_inner(path, buffer, None).map(|_| ())
}
