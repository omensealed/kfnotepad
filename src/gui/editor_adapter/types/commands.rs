//! Editor commands and syntax/text invalidation classification.

use super::*;

pub(crate) enum GuiEditorCommand {
    IcedAction(text_editor::Action),
    Delete,
    MoveTo(DocumentCursor),
    Paste(String),
    ScrollViewportLines(i32),
    SelectAll,
    SelectRightChars(usize),
}

pub(crate) fn gui_editor_command_invalidates_syntax(command: &GuiEditorCommand) -> bool {
    match command {
        GuiEditorCommand::IcedAction(action) => action.is_edit(),
        GuiEditorCommand::Delete | GuiEditorCommand::Paste(_) => true,
        GuiEditorCommand::MoveTo(_)
        | GuiEditorCommand::ScrollViewportLines(_)
        | GuiEditorCommand::SelectAll
        | GuiEditorCommand::SelectRightChars(_) => false,
    }
}

pub(crate) fn gui_editor_command_may_extend_syntax_cache(command: &GuiEditorCommand) -> bool {
    matches!(
        command,
        GuiEditorCommand::MoveTo(_)
            | GuiEditorCommand::ScrollViewportLines(_)
            | GuiEditorCommand::IcedAction(text_editor::Action::Scroll { .. })
    )
}

pub(crate) fn gui_editor_command_mutates_text(command: &GuiEditorCommand) -> bool {
    match command {
        GuiEditorCommand::IcedAction(action) => action.is_edit(),
        GuiEditorCommand::Delete | GuiEditorCommand::Paste(_) => true,
        GuiEditorCommand::MoveTo(_)
        | GuiEditorCommand::ScrollViewportLines(_)
        | GuiEditorCommand::SelectAll
        | GuiEditorCommand::SelectRightChars(_) => false,
    }
}

pub(crate) fn gui_replacement_inputs_invalidate_syntax(
    inputs: &[GuiEditorReplacementInput],
) -> bool {
    inputs.iter().any(|input| {
        matches!(
            input,
            GuiEditorReplacementInput::InsertChar(_)
                | GuiEditorReplacementInput::InsertNewline
                | GuiEditorReplacementInput::DeleteBackward
                | GuiEditorReplacementInput::DeleteForward
                | GuiEditorReplacementInput::DeletePreviousWord
                | GuiEditorReplacementInput::DeleteNextWord
                | GuiEditorReplacementInput::DeleteToLineEnd
        )
    })
}

pub(crate) fn gui_replacement_inputs_mutates_text(inputs: &[GuiEditorReplacementInput]) -> bool {
    inputs.iter().any(|input| {
        matches!(
            input,
            GuiEditorReplacementInput::InsertChar(_)
                | GuiEditorReplacementInput::InsertNewline
                | GuiEditorReplacementInput::DeleteBackward
                | GuiEditorReplacementInput::DeleteForward
                | GuiEditorReplacementInput::DeletePreviousWord
                | GuiEditorReplacementInput::DeleteNextWord
                | GuiEditorReplacementInput::DeleteToLineEnd
        )
    })
}
