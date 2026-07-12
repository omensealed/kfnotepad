//! Native-dialog availability checks and path-prompt fallback selection.

use super::*;

impl KfnotepadGui {
    pub(crate) fn gui_file_dialog_unavailable_reason() -> Option<&'static str> {
        if env::var_os("KFNOTEPAD_DISABLE_NATIVE_FILE_DIALOG").is_some_and(|value| {
            matches!(
                value.to_string_lossy().as_ref(),
                "1" | "true" | "on" | "yes" | "TRUE" | "ON" | "YES"
            )
        }) {
            return Some("disabled by KFNOTEPAD_DISABLE_NATIVE_FILE_DIALOG");
        }

        #[cfg(target_os = "linux")]
        {
            let has_display = env::var_os("DISPLAY").is_some();
            let has_wayland = env::var_os("WAYLAND_DISPLAY").is_some();
            let session_type = env::var_os("XDG_SESSION_TYPE")
                .and_then(|value| value.into_string().ok())
                .map(|value| value.to_lowercase());
            let has_xdg_session = session_type
                .as_deref()
                .is_some_and(|value| value == "x11" || value == "wayland");
            if !(has_display || has_wayland || has_xdg_session) {
                return Some("no desktop session detected");
            }
        }

        None
    }

    pub(super) fn request_file_dialog_fallback(
        &mut self,
        prompt: GuiPathPrompt,
        reason: &str,
    ) -> Task<Message> {
        self.show_path_prompt(prompt);
        self.status_message = format!(
            "{} dialog unavailable ({reason}); using path prompt",
            match prompt {
                GuiPathPrompt::Open => "open",
                GuiPathPrompt::SaveAs => "save as",
                _ => "file",
            }
        );
        Task::none()
    }
}
