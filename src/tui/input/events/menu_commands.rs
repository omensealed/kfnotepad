//! Menu navigation and document/workspace command dispatch.

use super::*;

mod document_dispatch;
mod help_document;
mod navigation;
mod new_file;
mod workspace_dispatch;

pub(crate) use document_dispatch::*;
pub(crate) use help_document::*;
pub(crate) use navigation::*;
pub(crate) use new_file::*;
pub(crate) use workspace_dispatch::*;
