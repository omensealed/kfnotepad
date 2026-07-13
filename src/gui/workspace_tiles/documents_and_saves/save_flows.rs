//! Synchronous and asynchronous save orchestration.

use super::*;

#[path = "save_flows/async_completions.rs"]
mod async_completions;
#[path = "save_flows/async_requests.rs"]
mod async_requests;
#[path = "save_flows/save_as_sync.rs"]
mod save_as_sync;
#[path = "save_flows/sync_save.rs"]
mod sync_save;
