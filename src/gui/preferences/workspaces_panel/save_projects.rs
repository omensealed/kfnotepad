//! Workspace-project capture, saving, and last-workspace persistence.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn save_current_workspace_project(&mut self) {
        self.save_workspace_project_named("current workspace", "current workspace");
    }

    pub(in crate::gui) fn save_named_workspace_project(&mut self) {
        let name = self.workspace_project_name.trim().to_string();
        if name.is_empty() {
            self.status_message = "workspace save failed: project name required".to_string();
            return;
        }
        self.save_workspace_project_named(&name, &name);
    }

    pub(in crate::gui) fn save_workspace_project_named(
        &mut self,
        project_name: &str,
        status_name: &str,
    ) {
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            self.status_message =
                "workspace save failed: cannot resolve config directory".to_string();
            return;
        };
        let Some(project_path) = gui_workspace_project_path(&projects_dir, project_name) else {
            self.status_message = "workspace save failed: invalid project name".to_string();
            return;
        };
        let Some(project) = self.current_workspace_project(project_name) else {
            self.status_message = "workspace save failed: cannot capture layout".to_string();
            return;
        };

        match save_gui_workspace_project(&project_path, &project) {
            Ok(()) => {
                self.refresh_workspace_projects();
                self.status_message = format!("workspace project saved: {status_name}");
            }
            Err(error) => {
                self.status_message = format!("workspace save failed: {error}");
            }
        }
    }

    pub(in crate::gui) fn current_workspace_project(
        &self,
        project_name: &str,
    ) -> Option<GuiWorkspaceProject> {
        let layout = gui_layout_from_state(
            &self.panes,
            &self.workspace,
            self.browser_visible,
            self.browser_width,
        )?;
        let active_ordinal = self
            .workspace
            .tiles
            .iter()
            .position(|tile| tile.id == self.workspace.active)
            .unwrap_or(0);
        Some(GuiWorkspaceProject {
            name: project_name.to_string(),
            files: self
                .workspace
                .tiles
                .iter()
                .map(|tile| tile.document.path.clone())
                .collect(),
            active_ordinal,
            layout: Some(layout),
        })
    }

    pub(in crate::gui) fn persist_last_workspace_if_enabled(&mut self) {
        if !self.settings.gui_restore_last_workspace {
            return;
        }
        let Some(projects_dir) = self.workspace_projects_dir.clone() else {
            return;
        };
        let Some(project_path) = gui_workspace_project_path(&projects_dir, "current workspace")
        else {
            return;
        };
        let Some(project) = self.current_workspace_project("current workspace") else {
            self.status_message = "workspace autosave failed: cannot capture layout".to_string();
            return;
        };
        if let Err(error) = save_gui_workspace_project(&project_path, &project) {
            self.status_message = format!("workspace autosave failed: {error}");
        }
    }
}
