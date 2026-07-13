use super::*;

#[path = "syntax_colors/contrast.rs"]
mod syntax_contrast;
#[path = "syntax_colors/conversion.rs"]
mod syntax_conversion;
#[path = "syntax_colors/roles.rs"]
mod syntax_roles;

pub(in crate::gui::app::state) use syntax_contrast::*;
pub(in crate::gui::app::state) use syntax_conversion::*;
use syntax_roles::*;
