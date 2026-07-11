fn theme_reader_shortcut_message(event: &Event) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    }) = event
    else {
        return None;
    };

    if modifiers.control()
        && modifiers.shift()
        && shortcut_character_matches(key, *physical_key, 't')
    {
        return Some(Message::CycleSyntaxTheme);
    }
    if modifiers.control()
        && !modifiers.shift()
        && shortcut_character_matches(key, *physical_key, 't')
    {
        return Some(Message::CycleTheme);
    }
    if modifiers.control()
        && !modifiers.shift()
        && shortcut_character_matches(key, *physical_key, 'r')
    {
        return Some(Message::MenuCommand(GuiMenuCommand::ToggleReaderMode));
    }
    None
}
