//! Containment-checked workspace project deletion through OS trash.

use super::*;

pub fn delete_gui_workspace_project(
    projects_dir: &Path,
    project_path: &Path,
) -> Result<GuiWorkspaceProjectDeleteResult, EditorConfigError> {
    if project_path.extension().and_then(|ext| ext.to_str()) != Some("v1") {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path must end in .v1".to_string(),
        });
    }
    let Some(project_parent) = project_path.parent() else {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path has no parent directory".to_string(),
        });
    };

    let canonical_projects_dir =
        projects_dir
            .canonicalize()
            .map_err(|source| EditorConfigError::Read {
                path: projects_dir.to_path_buf(),
                source,
            })?;
    let canonical_project_parent =
        project_parent
            .canonicalize()
            .map_err(|source| EditorConfigError::Read {
                path: project_parent.to_path_buf(),
                source,
            })?;
    if canonical_project_parent != canonical_projects_dir {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path is outside the project directory".to_string(),
        });
    }

    match project_path.canonicalize() {
        Ok(canonical_project_path) => {
            if canonical_project_path.parent() != Some(canonical_projects_dir.as_path()) {
                return Err(EditorConfigError::Invalid {
                    path: project_path.to_path_buf(),
                    message: "workspace project target is outside the project directory"
                        .to_string(),
                });
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(GuiWorkspaceProjectDeleteResult::Missing);
        }
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: project_path.to_path_buf(),
                source,
            });
        }
    }

    match move_path_to_trash(project_path) {
        Ok(()) => Ok(GuiWorkspaceProjectDeleteResult::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(GuiWorkspaceProjectDeleteResult::Missing)
        }
        Err(source) => Err(EditorConfigError::Remove {
            path: project_path.to_path_buf(),
            source,
        }),
    }
}
