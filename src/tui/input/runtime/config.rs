//! Runtime configuration paths, workspace restore requests, and settings persistence.

use super::*;

pub(crate) fn current_editor_config_path() -> Option<PathBuf> {
    kfnotepad::current_editor_config_path()
}

pub(crate) fn current_workspace_projects_dir() -> Option<PathBuf> {
    kfnotepad::current_gui_workspace_projects_dir().map(|path| path.join(TUI_WORKSPACE_DIR_NAME))
}

pub(crate) fn current_tui_restore_project_request() -> Option<(PathBuf, EditorSettings)> {
    let config_path = current_editor_config_path()?;
    let settings = load_editor_settings(&config_path).ok()?;
    if !settings.gui_restore_last_workspace {
        return None;
    }
    let projects_dir = current_workspace_projects_dir()?;
    let project_path = gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME)?;
    Some((project_path, settings))
}

pub(crate) fn persist_runtime_settings(runtime: &mut EditorRuntime) {
    let Some(config_path) = runtime.config_path.as_deref() else {
        return;
    };

    if let Err(error) = save_editor_settings(config_path, runtime.settings) {
        runtime.status = format!("{}; config not saved: {error}", runtime.status);
    }
}
