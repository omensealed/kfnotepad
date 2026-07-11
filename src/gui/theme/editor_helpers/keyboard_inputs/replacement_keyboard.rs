pub(super) fn gui_editor_replacement_inputs_from_keyboard_event(
    event: &keyboard::Event,
) -> Vec<GuiEditorReplacementInput> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        text,
        ..
    } = event
    else {
        return Vec::new();
    };

    let modified_command = modifiers.control() || modifiers.alt() || modifiers.logo();
    if modifiers.control() && !modifiers.alt() && !modifiers.logo() {
        match key.as_ref() {
            Key::Character("k" | "K") => return vec![GuiEditorReplacementInput::DeleteToLineEnd],
            Key::Named(Named::ArrowLeft) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::WordLeft,
                )];
            }
            Key::Named(Named::ArrowRight) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::WordRight,
                )];
            }
            Key::Named(Named::Backspace) => {
                return vec![GuiEditorReplacementInput::DeletePreviousWord];
            }
            Key::Named(Named::Delete) => return vec![GuiEditorReplacementInput::DeleteNextWord],
            _ => {}
        }
    }
    if modifiers.control()
        && !modifiers.alt()
        && !modifiers.logo()
        && matches!(key.as_ref(), Key::Character("a" | "A"))
    {
        return vec![GuiEditorReplacementInput::SelectAll];
    }
    if !modified_command {
        match key.as_ref() {
            Key::Named(Named::Enter) => return vec![GuiEditorReplacementInput::InsertNewline],
            Key::Named(Named::Backspace) => {
                return vec![GuiEditorReplacementInput::DeleteBackward];
            }
            Key::Named(Named::Delete) => return vec![GuiEditorReplacementInput::DeleteForward],
            Key::Named(Named::Escape) => return vec![GuiEditorReplacementInput::ClearSelection],
            Key::Named(Named::Home) => return vec![GuiEditorReplacementInput::MoveLineStart],
            Key::Named(Named::End) => return vec![GuiEditorReplacementInput::MoveLineEnd],
            Key::Named(Named::ArrowLeft) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Left)];
            }
            Key::Named(Named::ArrowRight) => {
                return vec![GuiEditorReplacementInput::Move(
                    kfnotepad::CursorMove::Right,
                )];
            }
            Key::Named(Named::ArrowUp) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Up)];
            }
            Key::Named(Named::ArrowDown) => {
                return vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down)];
            }
            Key::Named(Named::PageUp) => {
                return vec![GuiEditorReplacementInput::ScrollViewportLines(
                    -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
                )];
            }
            Key::Named(Named::PageDown) => {
                return vec![GuiEditorReplacementInput::ScrollViewportLines(
                    GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
                )];
            }
            _ => {}
        }
    }

    if modified_command {
        return Vec::new();
    }

    gui_editor_replacement_inputs_from_text(text.as_deref().unwrap_or_default())
}
