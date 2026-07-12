//! Header and menu-bar/dropdown rendering.

use super::*;

mod dropdown;
mod formatting;
mod header;
mod menu_bar;

pub(crate) use dropdown::write_menu_dropdown;
use formatting::{format_menu_item, menu_bar_text};
pub(crate) use formatting::{menu_dropdown_column, menu_item_display_width};
pub(crate) use header::{show_menu_bar, write_header};
use menu_bar::write_menu_bar;
