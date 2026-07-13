//! Browser root changes.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn set_browser_root(
        &mut self,
        directory: PathBuf,
    ) -> Task<Message> {
        self.status_message = format!("loading browser: {}", directory.display());
        self.request_browser_load(directory, true)
    }
}
