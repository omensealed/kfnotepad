//! Prompt validation and status updates.

use super::*;

pub(crate) fn refresh_sidebar_prompt_status(runtime: &mut EditorRuntime) {
    runtime.status = match runtime.sidebar_prompt.as_ref() {
        Some(SidebarPrompt::CreateFile) => format!("New file name: {}", runtime.sidebar_query),
        Some(SidebarPrompt::CreateDirectory) => {
            format!("New directory name: {}", runtime.sidebar_query)
        }
        Some(SidebarPrompt::DeleteConfirm { recursive, .. }) => {
            if *recursive {
                format!(
                    "Delete directory and all contents? type yes: {}",
                    runtime.sidebar_query
                )
            } else {
                format!("Delete file? type yes: {}", runtime.sidebar_query)
            }
        }
        None => runtime.status.clone(),
    };
}
