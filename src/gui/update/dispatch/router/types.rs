//! Shared result type for ordered message-domain routing.

use super::*;

pub(super) enum GuiDispatchResult {
    Handled(Task<Message>),
    Unhandled(Message),
}

pub(super) fn handled_none() -> GuiDispatchResult {
    GuiDispatchResult::Handled(Task::none())
}
