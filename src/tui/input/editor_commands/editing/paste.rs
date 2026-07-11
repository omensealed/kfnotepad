pub(crate) fn insert_paste(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    text: &str,
) -> bool {
    let normalized = text.replace("\r\n", "\n");
    for value in normalized.replace('\r', "\n").chars() {
        match value {
            '\n' => {
                let handled = handle_key_event(
                    document,
                    cursor,
                    runtime,
                    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                );
                if handled {
                    return true;
                }
            }
            value => {
                let event = KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE);
                if handle_key_event(document, cursor, runtime, event) {
                    return true;
                }
            }
        }
    }
    false
}
