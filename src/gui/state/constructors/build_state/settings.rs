//! Editor settings load with status-preserving fallback.

use super::super::*;

pub(super) fn load_gui_settings(
    config_path: Option<&std::path::Path>,
    status_messages: &mut Vec<String>,
) -> EditorSettings {
    config_path
        .map(load_editor_settings)
        .transpose()
        .map(|settings| settings.unwrap_or_default())
        .unwrap_or_else(|error| {
            status_messages.push(format!("settings unavailable: {error}"));
            EditorSettings::default()
        })
}
