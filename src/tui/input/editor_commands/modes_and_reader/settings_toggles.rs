//! Persisted editor display and search setting toggles.

use super::*;

pub(crate) fn toggle_line_numbers(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.show_line_numbers = !runtime.settings.show_line_numbers;
    runtime.status = if runtime.settings.show_line_numbers {
        String::from("Line numbers on")
    } else {
        String::from("Line numbers off")
    };
    persist_runtime_settings(runtime);
}

pub(crate) fn cycle_theme(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.theme_id = runtime.settings.theme_id.next();
    runtime.status = format!("Theme: {}", runtime.settings.theme_id.label());
    persist_runtime_settings(runtime);
}

pub(crate) fn cycle_syntax_theme(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.syntax_theme_id = runtime.settings.syntax_theme_id.next();
    runtime.status = format!("Syntax theme: {}", runtime.settings.syntax_theme_id.label());
    persist_runtime_settings(runtime);
}

pub(crate) fn toggle_search_case(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.search_case_sensitive = !runtime.settings.search_case_sensitive;
    persist_runtime_settings(runtime);
    runtime.status = if runtime.settings.search_case_sensitive {
        String::from("Search exact case")
    } else {
        String::from("Search ignore case")
    };
}

pub(crate) fn toggle_wrap(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.settings.wrap_lines = !runtime.settings.wrap_lines;
    runtime.status = if runtime.settings.wrap_lines {
        String::from("Wrap on")
    } else {
        String::from("Wrap off")
    };
    persist_runtime_settings(runtime);
}
