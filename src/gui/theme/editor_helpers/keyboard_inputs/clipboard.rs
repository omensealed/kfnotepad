pub(super) fn gui_editor_clipboard_shortcut_command(
    event: &keyboard::Event,
) -> Option<GuiMenuCommand> {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    } = event
    else {
        return None;
    };
    if !modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }

    let character = key.to_latin(*physical_key).or_else(|| match key.as_ref() {
        Key::Character(value) => value.chars().next().map(|value| value.to_ascii_lowercase()),
        _ => None,
    })?;

    match character.to_ascii_lowercase() {
        'c' => Some(GuiMenuCommand::Copy),
        'x' => Some(GuiMenuCommand::Cut),
        'v' => Some(GuiMenuCommand::Paste),
        'z' if modifiers.shift() => Some(GuiMenuCommand::Redo),
        'z' => Some(GuiMenuCommand::Undo),
        'y' => Some(GuiMenuCommand::Redo),
        _ => None,
    }
}
