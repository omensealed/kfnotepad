//! Command dispatch for native and replacement editor backends.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn apply(&mut self, command: GuiEditorCommand) {
        self.apply_replacement_command(command);
    }
}
