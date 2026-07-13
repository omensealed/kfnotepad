//! Workspace project loading, partial restore status, and launch command construction.

use super::*;

#[derive(Debug)]
pub(in crate::gui::app::state) struct RestoredGuiWorkspaceProject {
    pub(in crate::gui::app::state) project: GuiWorkspaceProject,
    pub(in crate::gui::app::state) documents: Vec<TextDocument>,
    pub(in crate::gui::app::state) active_loaded_ordinal: Option<usize>,
    pub(in crate::gui::app::state) skipped_files: Vec<String>,
    pub(in crate::gui::app::state) created_blank: bool,
}

impl RestoredGuiWorkspaceProject {
    pub(in crate::gui::app::state) fn skipped_status_message(&self) -> Option<String> {
        if self.skipped_files.is_empty() {
            return None;
        }
        let first = self
            .skipped_files
            .first()
            .map(String::as_str)
            .unwrap_or("unknown path");
        let loaded = if self.created_blank {
            "opened blank tile".to_string()
        } else {
            format!("loaded {} file(s)", self.documents.len())
        };
        Some(format!(
            "skipped {} missing/unavailable workspace file(s), {loaded}; first: {first}",
            self.skipped_files.len()
        ))
    }
}

pub(in crate::gui::app::state) fn load_workspace_project_launch_documents(
    path: &Path,
    current_dir: PathBuf,
) -> Result<RestoredGuiWorkspaceProject, String> {
    let project = load_workspace_project_launch(path)?;
    Ok(restore_gui_workspace_project_documents(
        project,
        current_dir,
    ))
}

pub(in crate::gui::app::state) fn restore_gui_workspace_project_documents(
    project: GuiWorkspaceProject,
    current_dir: PathBuf,
) -> RestoredGuiWorkspaceProject {
    let mut documents = Vec::new();
    let mut active_loaded_ordinal = None;
    let mut skipped_files = Vec::new();
    for (ordinal, file_path) in project.files.iter().enumerate() {
        let document = match open_text_file(file_path) {
            Ok(document) => document,
            Err(error) => {
                skipped_files.push(format!("{}: {error}", file_path.display()));
                continue;
            }
        };
        if ordinal == project.active_ordinal {
            active_loaded_ordinal = Some(documents.len());
        }
        documents.push(document);
    }
    let created_blank = documents.is_empty();
    if created_blank {
        documents.push(empty_document(current_dir));
        active_loaded_ordinal = Some(0);
    }
    if active_loaded_ordinal.is_none() && !documents.is_empty() {
        active_loaded_ordinal = Some(documents.len() - 1);
    }
    RestoredGuiWorkspaceProject {
        project,
        documents,
        active_loaded_ordinal,
        skipped_files,
        created_blank,
    }
}

pub(in crate::gui::app::state) fn workspace_project_launch_command(
    executable: &Path,
    project_path: &Path,
) -> Command {
    let mut command = Command::new(executable);
    command.env(WORKSPACE_PROJECT_ENV, project_path);
    command
}

pub(in crate::gui::app::state) fn current_managed_notes_dir(
) -> Result<PathBuf, kfnotepad::ManagedNotesError> {
    kfnotepad::current_managed_notes_dir()
}
