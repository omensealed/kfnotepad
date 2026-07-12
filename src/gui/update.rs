//! GUI update and event subscription logic.
//!
//! This module owns transition behavior for messages, async work scheduling,
//! and short-lived UI side effects.

use super::*;

#[path = "update/dispatch.rs"]
mod dispatch;
#[path = "update/subscription.rs"]
mod subscription;

pub(super) use dispatch::update;
pub(super) use subscription::subscription;
