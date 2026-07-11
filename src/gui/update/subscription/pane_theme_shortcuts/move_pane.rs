fn move_pane_shortcut_message(event: &Event) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };
    if !(modifiers.control() && modifiers.shift()) {
        return None;
    }

    match key.as_ref() {
        Key::Named(Named::ArrowLeft) => Some(Message::MoveActivePane(pane_grid::Direction::Left)),
        Key::Named(Named::ArrowRight) => Some(Message::MoveActivePane(pane_grid::Direction::Right)),
        Key::Named(Named::ArrowUp) => Some(Message::MoveActivePane(pane_grid::Direction::Up)),
        Key::Named(Named::ArrowDown) => Some(Message::MoveActivePane(pane_grid::Direction::Down)),
        _ => None,
    }
}
