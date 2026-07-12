//! Sidebar navigation, mutation prompts, scrolling, and file activation.

use super::*;

mod helpers;
mod navigation;

include!("sidebar/prompts_and_mutation.rs");
include!("sidebar/activation.rs");

pub(crate) use helpers::*;
pub(crate) use navigation::*;
