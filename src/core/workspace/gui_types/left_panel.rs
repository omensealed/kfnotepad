//! GUI left-panel mode and visibility state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiLeftPanelMode {
    Files,
    Workspaces,
    Preferences,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiLeftPanelState {
    pub visible: bool,
    pub mode: GuiLeftPanelMode,
}

impl Default for GuiLeftPanelState {
    fn default() -> Self {
        Self {
            visible: true,
            mode: GuiLeftPanelMode::Files,
        }
    }
}

impl GuiLeftPanelState {
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn show_files(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Files;
    }

    pub fn show_workspaces(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Workspaces;
    }

    pub fn show_preferences(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Preferences;
    }

    pub fn toggle_mode(&mut self) {
        self.visible = true;
        self.mode = match self.mode {
            GuiLeftPanelMode::Files => GuiLeftPanelMode::Workspaces,
            GuiLeftPanelMode::Workspaces => GuiLeftPanelMode::Preferences,
            GuiLeftPanelMode::Preferences => GuiLeftPanelMode::Files,
        };
    }

    pub fn title(&self) -> &'static str {
        match self.mode {
            GuiLeftPanelMode::Files => "Files",
            GuiLeftPanelMode::Workspaces => "Workspaces",
            GuiLeftPanelMode::Preferences => "Preferences",
        }
    }
}
