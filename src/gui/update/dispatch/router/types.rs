enum GuiDispatchResult {
    Handled(Task<Message>),
    Unhandled(Message),
}

fn handled_none() -> GuiDispatchResult {
    GuiDispatchResult::Handled(Task::none())
}
