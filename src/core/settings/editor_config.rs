//! Editor configuration loading, parsing, and atomic persistence.

use super::*;

#[path = "editor_config/load.rs"]
mod load;
#[path = "editor_config/parse.rs"]
mod parse;
#[path = "editor_config/save.rs"]
mod save;

pub use load::load_editor_settings;
pub use parse::parse_editor_settings_config;
pub use save::save_editor_settings;
