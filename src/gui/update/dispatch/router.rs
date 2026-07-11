include!("router/types.rs");
include!("router/browser_files.rs");
include!("router/workspace_preferences.rs");
include!("router/panes.rs");
include!("router/search_editor.rs");
include!("router/replacement.rs");
include!("router/misc.rs");

pub(super) fn update(state: &mut KfnotepadGui, message: Message) -> Task<Message> {
    let message = match dispatch_browser_and_files(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_workspace_and_preferences(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_panes(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_search_and_editor(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_replacement_editor(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    match dispatch_miscellaneous(state, message) {
        GuiDispatchResult::Handled(task) => task,
        GuiDispatchResult::Unhandled(_) => Task::none(),
    }
}
