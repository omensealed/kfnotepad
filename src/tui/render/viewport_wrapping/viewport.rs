//! Active, passive, and horizontal viewport clamping.

use super::*;

mod active;
mod horizontal;
mod passive;

pub(crate) use active::*;
pub(crate) use horizontal::*;
pub(crate) use passive::clamp_passive_viewport;
use passive::max_viewport_start;
