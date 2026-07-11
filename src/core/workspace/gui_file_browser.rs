#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiFileBrowser {
    pub sidebar: FileSidebarState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiFileBrowserActivation {
    Navigated { current_dir: PathBuf },
    OpenTile { path: PathBuf },
}

#[derive(Debug)]
pub enum GuiFileBrowserError {
    EmptySelection,
    Navigate { source: FileSidebarError },
}

impl fmt::Display for GuiFileBrowserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => write!(formatter, "no file-browser entry selected"),
            Self::Navigate { source } => write!(formatter, "{source}"),
        }
    }
}

impl GuiFileBrowser {
    pub fn load(current_dir: PathBuf) -> Result<Self, FileSidebarError> {
        Ok(Self {
            sidebar: FileSidebarState::load(current_dir)?,
        })
    }

    pub fn refresh(&mut self) -> Result<(), FileSidebarError> {
        let selected_path = self
            .sidebar
            .selected_entry()
            .map(|entry| entry.path.clone());
        let previous_selected = self.sidebar.selected;
        let previous_scroll = self.sidebar.scroll;
        let mut refreshed = FileSidebarState::load(self.sidebar.current_dir.clone())?;
        refreshed.selected = selected_path
            .and_then(|path| {
                refreshed
                    .entries
                    .iter()
                    .position(|entry| entry.path == path)
            })
            .unwrap_or_else(|| previous_selected.min(refreshed.entries.len().saturating_sub(1)));
        refreshed.scroll = previous_scroll.min(refreshed.selected);
        refreshed.keep_selection_visible(1);
        self.sidebar = refreshed;
        Ok(())
    }

    pub fn selected_entry(&self) -> Option<&FileSidebarEntry> {
        self.sidebar.selected_entry()
    }

    pub fn select_previous_wrapping(&mut self, visible_rows: usize) {
        self.sidebar.select_previous_wrapping(visible_rows);
    }

    pub fn select_next_wrapping(&mut self, visible_rows: usize) {
        self.sidebar.select_next_wrapping(visible_rows);
    }

    pub fn activate_selected(&mut self) -> Result<GuiFileBrowserActivation, GuiFileBrowserError> {
        let Some(entry) = self.sidebar.selected_entry().cloned() else {
            return Err(GuiFileBrowserError::EmptySelection);
        };
        self.activate_entry(entry)
    }

    pub fn activate_mouse_row(
        &mut self,
        row: u16,
    ) -> Result<Option<GuiFileBrowserActivation>, GuiFileBrowserError> {
        let Some(entry) = self.sidebar.selected_entry_for_mouse_row(row) else {
            return Ok(None);
        };
        self.activate_entry(entry).map(Some)
    }

    fn activate_entry(
        &mut self,
        entry: FileSidebarEntry,
    ) -> Result<GuiFileBrowserActivation, GuiFileBrowserError> {
        match entry.kind {
            FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
                self.sidebar = FileSidebarState::load(entry.path)
                    .map_err(|source| GuiFileBrowserError::Navigate { source })?;
                Ok(GuiFileBrowserActivation::Navigated {
                    current_dir: self.sidebar.current_dir.clone(),
                })
            }
            FileSidebarEntryKind::File => {
                Ok(GuiFileBrowserActivation::OpenTile { path: entry.path })
            }
        }
    }
}
