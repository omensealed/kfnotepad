//! Sidebar navigation, mutation prompts, scrolling, and file activation.

use super::*;

mod helpers;
mod navigation;
mod prompts_and_mutation;

include!("sidebar/activation.rs");

pub(crate) use helpers::*;
pub(crate) use navigation::*;
pub(crate) use prompts_and_mutation::*;
