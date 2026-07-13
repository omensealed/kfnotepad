use super::*;

pub(in crate::gui::app::state) fn gui_editor_replacement_inputs_from_text(
    text: &str,
) -> Vec<GuiEditorReplacementInput> {
    text.chars()
        .filter(|value| !value.is_control())
        .map(GuiEditorReplacementInput::InsertChar)
        .collect()
}
