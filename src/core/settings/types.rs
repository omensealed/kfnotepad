//! Editor settings, font/theme identifiers, and configuration errors.

use super::*;

#[path = "types/editor_config_error.rs"]
mod editor_config_error;
#[path = "types/editor_settings.rs"]
mod editor_settings;
#[path = "types/editor_theme_id.rs"]
mod editor_theme_id;
#[path = "types/gui_font_family.rs"]
mod gui_font_family;

pub use editor_config_error::EditorConfigError;
pub use editor_settings::*;
pub use editor_theme_id::EditorThemeId;
pub use gui_font_family::GuiFontFamily;
