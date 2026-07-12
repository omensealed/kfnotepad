//! Private-permission atomic GUI layout persistence.

use super::*;

pub fn save_gui_layout(path: &Path, layout: &GuiLayout) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text = serialize_gui_layout(layout);
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}
