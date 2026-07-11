fn file_window_shortcut_message(event: &Event, window_id: window::Id) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'o') =>
        {
            Some(Message::OpenPromptRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("o" | "O")) =>
        {
            Some(Message::OpenPromptRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && modifiers.shift()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 's') =>
        {
            Some(Message::SaveAsPromptRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control()
                && modifiers.shift()
                && matches!(key.as_ref(), Key::Character("s" | "S")) =>
        {
            Some(Message::SaveAsPromptRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'n') =>
        {
            Some(Message::NewTileRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("n" | "N")) =>
        {
            Some(Message::NewTileRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 's') =>
        {
            Some(Message::SaveRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("s" | "S")) =>
        {
            Some(Message::SaveRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'b') =>
        {
            Some(Message::ToggleBrowser)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("b" | "B")) =>
        {
            Some(Message::ToggleBrowser)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::F4)) =>
        {
            Some(Message::CloseActivePane)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'q') =>
        {
            Some(Message::QuitRequested(window_id))
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("q" | "Q")) =>
        {
            Some(Message::QuitRequested(window_id))
        }
        _ => None,
    }
}
