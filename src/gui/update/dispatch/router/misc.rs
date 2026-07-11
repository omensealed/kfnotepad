fn dispatch_miscellaneous(state: &mut KfnotepadGui, message: Message) -> GuiDispatchResult {
    match message {
        Message::QuitRequested(window_id) => GuiDispatchResult::Handled(state.request_app_close(window_id)),
        Message::QuitLatestWindow(Some(window_id)) => {
            GuiDispatchResult::Handled(state.request_app_close(window_id))
        }
        Message::QuitLatestWindow(None) => {
            handle_quit_latest_window_missing(state);
            handled_none()
        }
        Message::WindowCloseRequested(window_id) => {
            GuiDispatchResult::Handled(state.request_app_close(window_id))
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
