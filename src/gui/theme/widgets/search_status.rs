// Search, navigation, and status controls for the editor shell.
#[path = "search_status/find.rs"]
mod search_find;
#[path = "search_status/layout.rs"]
mod search_layout;
#[path = "search_status/navigation.rs"]
mod search_navigation;
#[path = "search_status/status_bar.rs"]
mod status_bar;

use search_find::gui_find_controls;
pub(in crate::gui::app::state) use search_layout::gui_search_controls;
use search_navigation::gui_navigation_controls;
pub(in crate::gui::app::state) use status_bar::gui_status_bar;
