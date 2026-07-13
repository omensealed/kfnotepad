//! Incremental bindings from replacement inputs to Iced editor actions.

use super::*;

impl KfnotepadGui {
    pub(super) fn gui_editor_replacement_input_has_editor_delta_binding(
        input: &GuiEditorReplacementInput,
    ) -> bool {
        matches!(
            input,
            GuiEditorReplacementInput::InsertChar(_)
                | GuiEditorReplacementInput::InsertNewline
                | GuiEditorReplacementInput::DeleteBackward
                | GuiEditorReplacementInput::DeleteForward
        )
    }

    pub(super) fn gui_editor_replacement_input_apply_delta_to_editor(
        editor: &mut GuiEditorAdapter,
        input: &GuiEditorReplacementInput,
    ) {
        let Some(action) = (match input {
            GuiEditorReplacementInput::InsertChar(value) => {
                Some(text_editor::Action::Edit(text_editor::Edit::Insert(*value)))
            }
            GuiEditorReplacementInput::InsertNewline => {
                Some(text_editor::Action::Edit(text_editor::Edit::Enter))
            }
            GuiEditorReplacementInput::DeleteBackward => {
                Some(text_editor::Action::Edit(text_editor::Edit::Backspace))
            }
            GuiEditorReplacementInput::DeleteForward => {
                Some(text_editor::Action::Edit(text_editor::Edit::Delete))
            }
            _ => None,
        }) else {
            return;
        };

        editor.apply(GuiEditorCommand::IcedAction(action));
    }
}
