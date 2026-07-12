//! Workspace-project listing, deletion, and new-window opening.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn refresh_workspace_projects(&mut self) {
        self.pending_project_delete = None;
        let Some(projects_dir) = self.workspace_projects_dir.as_deref() else {
            self.workspace_projects.clear();
            self.status_message =
                "workspace projects unavailable: cannot resolve config directory".to_string();
            return;
        };
        match list_gui_workspace_projects(projects_dir) {
            Ok(projects) => {
                let count = projects.len();
                self.workspace_projects = projects;
                self.status_message = format!("workspace projects: {count}");
            }
            Err(error) => {
                self.workspace_projects.clear();
                self.status_message = format!("workspace projects failed: {error}");
            }
        }
    }

    pub(in crate::gui) fn delete_workspace_project(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        if self.pending_project_delete != Some(index) {
            self.pending_project_delete = Some(index);
            self.pending_project_open = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = if self.is_current_workspace_project(&entry)
                && self.settings.gui_restore_last_workspace
            {
                "restore target selected; delete again to remove last-workspace restore project"
                    .to_string()
            } else {
                format!(
                    "delete workspace project {}? click delete again",
                    entry.project.name
                )
            };
            return;
        }

        self.pending_project_delete = None;
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            self.status_message =
                "workspace delete failed: cannot resolve config directory".to_string();
            return;
        };

        match delete_gui_workspace_project(&projects_dir, &entry.path) {
            Ok(GuiWorkspaceProjectDeleteResult::Deleted) => {
                self.refresh_workspace_projects();
                self.status_message =
                    format!("workspace project moved to trash: {}", entry.project.name);
            }
            Ok(GuiWorkspaceProjectDeleteResult::Missing) => {
                self.refresh_workspace_projects();
                self.status_message =
                    format!("workspace project already missing: {}", entry.project.name);
            }
            Err(error) => {
                self.status_message = format!("workspace delete failed: {error}");
            }
        }
    }

    pub(in crate::gui) fn is_current_workspace_project(
        &self,
        entry: &GuiWorkspaceProjectEntry,
    ) -> bool {
        self.workspace_projects_dir
            .as_deref()
            .and_then(|projects_dir| gui_workspace_project_path(projects_dir, "current workspace"))
            .is_some_and(|current_path| current_path == entry.path)
    }

    pub(in crate::gui) fn open_workspace_project_in_new_window(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        match self.spawn_workspace_project_window(&entry.path) {
            Ok(()) => {
                self.status_message = format!(
                    "opened workspace project {} in new window",
                    entry.project.name
                );
            }
            Err(error) => {
                self.status_message = format!("workspace project new window failed: {error}");
            }
        }
    }

    #[cfg(test)]
    pub(in crate::gui) fn spawn_workspace_project_window(&mut self, path: &Path) -> io::Result<()> {
        self.spawned_workspace_project_paths
            .push(path.to_path_buf());
        Ok(())
    }

    #[cfg(not(test))]
    pub(in crate::gui) fn spawn_workspace_project_window(&mut self, path: &Path) -> io::Result<()> {
        let executable = env::current_exe()?;
        workspace_project_launch_command(&executable, path)
            .spawn()
            .map(|_| ())
    }
}
