//! Explicit file and workspace-restore launch document loading.

use super::super::*;
use super::types::GuiLaunchDocuments;

pub(super) fn load_gui_launch_documents(
    launch: GuiLaunch,
    current_dir: &std::path::Path,
    settings: EditorSettings,
    workspace_project_path: Option<PathBuf>,
    workspace_projects_dir: Option<&std::path::Path>,
    status_messages: &mut Vec<String>,
) -> GuiLaunchDocuments {
    let mut documents = Vec::new();
    let mut launch_paths = launch.requested_paths;
    let mut project_layout = None;
    let mut project_active_ordinal = None;
    let project_restore_path = workspace_project_path
        .map(|path| (path, false))
        .or_else(|| {
            if !launch_paths.is_empty() || !settings.gui_restore_last_workspace {
                return None;
            }
            let projects_dir = workspace_projects_dir?;
            gui_workspace_project_path(projects_dir, "current workspace").map(|path| (path, true))
        });
    let show_startup_help_panel = launch_paths.is_empty() && project_restore_path.is_none();

    if let Some((path, is_auto_restore)) = project_restore_path {
        match load_workspace_project_launch_documents(&path, current_dir.to_path_buf()) {
            Ok(restored) => {
                if is_auto_restore {
                    status_messages.push(format!(
                        "restored last workspace project {}",
                        restored.project.name
                    ));
                } else {
                    status_messages.push(format!(
                        "opened workspace project {}",
                        restored.project.name
                    ));
                }
                if let Some(message) = restored.skipped_status_message() {
                    status_messages.push(message);
                }
                documents = restored.documents;
                launch_paths.clear();
                project_layout = if restored.skipped_files.is_empty() {
                    restored.project.layout.clone()
                } else {
                    None
                };
                project_active_ordinal = restored.active_loaded_ordinal;
            }
            Err(error) => {
                let action = if is_auto_restore {
                    "workspace auto-restore failed"
                } else {
                    "workspace project open failed"
                };
                status_messages.push(format!("{action}: {}: {error}", path.display()));
                launch_paths.clear();
            }
        }
    }

    for path in launch_paths {
        match open_text_file(&path) {
            Ok(document) => {
                status_messages.push(format!("opened {}", document.path.display()));
                documents.push(document);
            }
            Err(error) => {
                status_messages.push(format!("cannot open {}: {error}", path.display()));
            }
        }
    }

    if documents.is_empty() {
        documents.push(empty_document(current_dir.to_path_buf()));
        if status_messages.is_empty() {
            status_messages.push("started empty GUI document tile".to_string());
        }
    }

    GuiLaunchDocuments {
        documents,
        project_layout,
        project_active_ordinal,
        show_startup_help_panel,
    }
}
