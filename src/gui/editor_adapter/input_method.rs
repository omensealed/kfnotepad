//! IME state, requests, event capture area, and advanced Iced widget bridge.

use super::*;

#[path = "input_method/area.rs"]
mod area;
#[path = "input_method/request.rs"]
mod request;
#[path = "input_method/types.rs"]
mod types;
#[path = "input_method/widget.rs"]
mod widget;

pub(crate) use area::*;
pub(crate) use request::*;
pub(crate) use types::*;
