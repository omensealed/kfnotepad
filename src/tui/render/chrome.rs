//! Render frame/view models, tab strip, syntax cache, header, and menu chrome.

use super::*;

mod frame_and_view;
mod header_menu;
mod syntax_cache;
mod tab_strip;

pub(crate) use frame_and_view::*;
pub(crate) use header_menu::*;
pub(crate) use syntax_cache::*;
pub(crate) use tab_strip::*;
