//! GUI path adapters and external-file snapshot checks.

use super::*;

pub(in crate::gui::app::state) fn empty_document(current_dir: PathBuf) -> TextDocument {
    TextDocument {
        path: current_dir.join("untitled.txt"),
        buffer: TextBuffer::from_text(""),
    }
}

#[cfg(not(test))]
pub(in crate::gui::app::state) fn current_editor_config_path() -> Option<PathBuf> {
    kfnotepad::current_editor_config_path()
}

#[cfg(not(test))]
pub(in crate::gui::app::state) fn current_gui_layout_path() -> Option<PathBuf> {
    kfnotepad::current_gui_layout_path()
}

#[cfg(not(test))]
pub(in crate::gui::app::state) fn current_gui_workspace_projects_dir() -> Option<PathBuf> {
    kfnotepad::current_gui_workspace_projects_dir()
}

#[cfg(not(test))]
pub(in crate::gui::app::state) fn current_gui_workspace_project_launch_path() -> Option<PathBuf> {
    env::var_os(WORKSPACE_PROJECT_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(in crate::gui::app::state) fn gui_file_snapshot(
    path: &Path,
) -> io::Result<Option<GuiFileSnapshot>> {
    snapshot_text_file(path)
}

pub(in crate::gui::app::state) fn check_external_file_changes(
    candidates: Vec<GuiExternalFileCheckCandidate>,
) -> Vec<GuiExternalFileCheckResult> {
    let mut results = Vec::new();
    for candidate in candidates {
        let metadata_snapshot = match snapshot_text_file_metadata(&candidate.path) {
            Ok(Some(snapshot)) => snapshot,
            _ => continue,
        };
        if metadata_snapshot.bytes > MAX_TEXT_FILE_BYTES {
            results.push(GuiExternalFileCheckResult::Oversized {
                tile_id: candidate.tile_id,
                path: candidate.path,
            });
            continue;
        }
        if !external_file_snapshot_requires_deep_check(
            &metadata_snapshot,
            candidate.previous_snapshot.as_ref(),
            candidate.force_deep_check,
        ) {
            continue;
        }

        let current_snapshot = match gui_file_snapshot(&candidate.path) {
            Ok(Some(snapshot)) => snapshot,
            Err(error) if error.kind() == io::ErrorKind::FileTooLarge => {
                results.push(GuiExternalFileCheckResult::Oversized {
                    tile_id: candidate.tile_id,
                    path: candidate.path,
                });
                continue;
            }
            _ => continue,
        };
        let Some(previous_snapshot) = candidate.previous_snapshot else {
            results.push(GuiExternalFileCheckResult::SnapshotInitialized {
                tile_id: candidate.tile_id,
                snapshot: current_snapshot,
            });
            continue;
        };

        if current_snapshot == previous_snapshot {
            continue;
        }

        if candidate.dirty {
            results.push(GuiExternalFileCheckResult::DirtyChanged {
                tile_id: candidate.tile_id,
                path: candidate.path,
                snapshot: current_snapshot,
            });
            continue;
        }

        match open_text_file(&candidate.path) {
            Ok(document) => results.push(GuiExternalFileCheckResult::Reloaded {
                tile_id: candidate.tile_id,
                path: candidate.path,
                snapshot: current_snapshot,
                document: Box::new(document),
            }),
            Err(error) => {
                results.push(GuiExternalFileCheckResult::LoadFailed {
                    tile_id: candidate.tile_id,
                    path: candidate.path,
                    message: error.to_string(),
                });
            }
        }
    }

    results
}

pub(in crate::gui::app::state) fn external_file_snapshot_requires_deep_check(
    metadata: &FileMetadataSnapshot,
    previous_snapshot: Option<&GuiFileSnapshot>,
    force_deep_check: bool,
) -> bool {
    force_deep_check
        || previous_snapshot.is_none_or(|previous| !metadata.matches_file_snapshot(previous))
}

pub(in crate::gui::app::state) async fn check_external_file_changes_async(
    candidates: Vec<GuiExternalFileCheckCandidate>,
) -> Vec<GuiExternalFileCheckResult> {
    check_external_file_changes(candidates)
}

pub(in crate::gui::app::state) fn load_workspace_project_launch(
    path: &Path,
) -> Result<GuiWorkspaceProject, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_gui_workspace_project(&text).ok_or_else(|| "invalid workspace project".to_string())
}
