pub(crate) fn insert_paste(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    text: &str,
) -> bool {
    let normalized = text.replace("\r\n", "\n");
    let normalized = normalized.replace('\r', "\n");
    let mut handled = false;
    document.with_compound_edit(|document| {
        for value in normalized.chars() {
            match value {
                '\n' => {
                    handled = handle_key_event(
                        document,
                        cursor,
                        runtime,
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                    );
                }
                value => {
                    let event = KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE);
                    handled = handle_key_event(document, cursor, runtime, event);
                }
            }
            if handled {
                break;
            }
        }
    });
    handled
}
