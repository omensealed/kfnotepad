// GUI widget style callbacks, controls, and composed chrome helpers.

#[path = "widgets/buttons.rs"]
mod widget_buttons;
#[path = "widgets/chrome.rs"]
mod widget_chrome;
#[path = "widgets/editor_surface.rs"]
mod widget_editor_surface;
#[path = "widgets/menu_bar.rs"]
mod widget_menu_bar;
#[path = "widgets/search_status.rs"]
mod widget_search_status;
#[path = "widgets/styles.rs"]
mod widget_styles;

pub(in crate::gui::app::state) use widget_buttons::*;
pub(in crate::gui::app::state) use widget_chrome::*;
pub(in crate::gui::app::state) use widget_editor_surface::*;
pub(in crate::gui::app::state) use widget_menu_bar::*;
pub(in crate::gui::app::state) use widget_search_status::*;
pub(in crate::gui::app::state) use widget_styles::*;
