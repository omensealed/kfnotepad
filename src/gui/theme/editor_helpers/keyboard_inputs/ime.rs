#[cfg(test)]
use super::*;

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_editor_replacement_inputs_from_ime_event(
    event: &input_method::Event,
) -> Vec<GuiEditorReplacementInput> {
    match event {
        input_method::Event::Commit(text) => gui_editor_replacement_inputs_from_text(text),
        input_method::Event::Opened
        | input_method::Event::Preedit(_, _)
        | input_method::Event::Closed => Vec::new(),
    }
}
