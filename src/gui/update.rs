//! GUI update and event subscription logic.
//!
//! This module owns transition behavior for messages, async work scheduling,
//! and short-lived UI side effects.

use super::*;

include!("update/dispatch.rs");
include!("update/subscription.rs");
