// Theme-aware style callbacks grouped by widget family.
use super::*;

#[path = "styles/buttons.rs"]
mod button_styles;
#[path = "styles/editor.rs"]
mod editor_styles;
#[path = "styles/forms.rs"]
mod form_styles;
#[path = "styles/menu.rs"]
mod menu_styles;
#[path = "styles/scrollbar.rs"]
mod scrollbar_styles;

pub(in crate::gui::app::state) use button_styles::*;
pub(in crate::gui::app::state) use editor_styles::*;
pub(in crate::gui::app::state) use form_styles::*;
pub(in crate::gui::app::state) use menu_styles::*;
pub(in crate::gui::app::state) use scrollbar_styles::*;
