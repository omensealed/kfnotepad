impl<'a> EditorWorkspace<'a> {
    pub fn from_document(document: &'a mut TextDocument) -> Self {
        Self {
            tabs: vec![EditorTab {
                document: EditorTabDocument::Borrowed(document),
                state: EditorTabState::default(),
            }],
            active: 0,
        }
    }

    pub fn active_tab(&self) -> &EditorTab<'a> {
        &self.tabs[self.active]
    }

    pub fn active_tab_mut(&mut self) -> &mut EditorTab<'a> {
        &mut self.tabs[self.active]
    }

    pub fn push_owned_tab(&mut self, document: TextDocument) {
        self.tabs.push(EditorTab {
            document: EditorTabDocument::Owned(document),
            state: EditorTabState::default(),
        });
        self.active = self.tabs.len() - 1;
    }

    pub fn select_previous_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.active = if self.active == 0 {
            self.tabs.len() - 1
        } else {
            self.active - 1
        };
        true
    }

    pub fn select_next_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.active = (self.active + 1) % self.tabs.len();
        true
    }

    pub fn close_active_tab(&mut self, confirm_dirty: bool) -> CloseActiveTabResult {
        if self.tabs.len() <= 1 {
            return CloseActiveTabResult::OnlyTab;
        }

        if self.active_tab().document.as_ref().buffer.is_dirty() && !confirm_dirty {
            return CloseActiveTabResult::Dirty;
        }

        let closed_path = self.active_tab().document.as_ref().path.clone();
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
        CloseActiveTabResult::Closed { path: closed_path }
    }

    pub fn tab_strip_items(&self) -> Vec<TabStripItem> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| TabStripItem {
                label: document_display_name(&tab.document.as_ref().path).to_string(),
                active: index == self.active,
                dirty: tab.document.as_ref().buffer.is_dirty(),
            })
            .collect()
    }
}
