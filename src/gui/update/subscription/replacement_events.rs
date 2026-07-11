fn replacement_editor_event_message(
    event: Event,
    status: iced::event::Status,
) -> Option<Message> {
    match event {
        Event::Keyboard(event)
            if GUI_USE_READ_ONLY_EDITOR_RENDERER
                && matches!(status, iced::event::Status::Ignored) =>
        {
            let inputs = gui_editor_replacement_inputs_from_keyboard_event(&event);
            (!inputs.is_empty()).then_some(Message::ReplacementEditorInputs(inputs))
        }
        Event::InputMethod(event)
            if GUI_USE_READ_ONLY_EDITOR_RENDERER
                && matches!(status, iced::event::Status::Ignored) =>
        {
            Some(Message::ReplacementEditorIme(event))
        }
        _ => None,
    }
}
