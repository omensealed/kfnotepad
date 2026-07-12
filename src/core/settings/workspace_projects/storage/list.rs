//! Valid-project filtering and deterministic workspace project listing.

use super::*;

pub fn list_gui_workspace_projects(
    projects_dir: &Path,
) -> Result<Vec<GuiWorkspaceProjectEntry>, EditorConfigError> {
    let entries = match fs::read_dir(projects_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: projects_dir.to_path_buf(),
                source,
            });
        }
    };

    let mut projects = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("v1")
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Some(project) = parse_gui_workspace_project(&text) else {
            continue;
        };
        projects.push(GuiWorkspaceProjectEntry {
            path: entry.path(),
            project,
        });
    }

    projects.sort_by(|left, right| {
        left.project
            .name
            .cmp(&right.project.name)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(projects)
}
