//! Small path validation helpers shared by deterministic resolvers.

use std::path::Path;

pub(super) fn non_empty(path: Option<&Path>) -> Option<&Path> {
    path.filter(|path| !path.as_os_str().is_empty())
}
