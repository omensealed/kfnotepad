//! External-file checks, syntax caches, edit locks, and reload behavior.

use super::*;

#[path = "external_and_syntax/edit_locks.rs"]
mod edit_locks;
#[path = "external_and_syntax/external_checks.rs"]
mod external_checks;
#[path = "external_and_syntax/reload.rs"]
mod reload;
#[path = "external_and_syntax/syntax_cache.rs"]
mod syntax_cache;
