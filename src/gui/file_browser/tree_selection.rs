impl KfnotepadGui {
pub(super) fn handle_browser_tree_event(&mut self, event: DirectoryTreeEvent) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        match &event {
            DirectoryTreeEvent::Selected(path, is_dir, _) => {
                if *is_dir {
                    self.select_browser_path(path);
                    self.status_message = format!("selected folder {}", path.display());
                } else {
                    self.select_browser_path(path);
                    self.status_message = format!("selected file {}", path.display());
                }
            }
            DirectoryTreeEvent::DragCompleted { .. } => {
                self.status_message = "file browser drag is view-only".to_string();
            }
            DirectoryTreeEvent::Toggled(_)
            | DirectoryTreeEvent::Drag(_)
            | DirectoryTreeEvent::Loaded(_) => {}
        }

        let Some(tree) = self.browser_tree.as_mut() else {
            self.status_message = "file tree unavailable".to_string();
            return Task::none();
        };
        tree.update(event).map(Message::BrowserTreeEvent)
    }

    pub(super) fn toggle_local_browser_tree_path(&mut self, path: PathBuf) {
        if self.browser_expanded_paths.contains(&path) {
            self.browser_expanded_paths.remove(&path);
        } else {
            self.browser_expanded_paths.insert(path);
        }
    }

    pub(super) fn select_local_browser_tree_path(
        &mut self,
        path: PathBuf,
        is_dir: bool,
    ) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        self.status_message = if is_dir {
            format!("selected folder {}", path.display())
        } else {
            format!("selected file {}", path.display())
        };
        Task::none()
    }

    pub(super) fn activate_local_browser_tree_path(
        &mut self,
        path: PathBuf,
        is_dir: bool,
    ) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        if is_dir {
            self.set_browser_root(path)
        } else {
            #[cfg(test)]
            {
                let _opened = self.open_path_in_new_pane(path);
                Task::none()
            }
            #[cfg(not(test))]
            {
                self.open_path_in_new_pane_async(path)
            }
        }
    }

pub(super) fn select_browser_path(&mut self, path: &Path) {
        self.browser_selected_path = Some(path.to_path_buf());
        let Some(browser) = self.browser.as_mut() else {
            return;
        };
        if let Some(index) = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.path == path)
        {
            browser.sidebar.selected = index;
            browser.sidebar.keep_selection_visible(1);
        }
    self.pending_browser_delete = None;
}
}
