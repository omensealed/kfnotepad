//! Command-line parsing and short/full help text.

use super::*;

#[path = "cli_help/args.rs"]
mod args;
#[path = "cli_help/short_help.rs"]
mod short_help;
#[path = "cli_help/tui_help.rs"]
mod tui_help;

pub use args::parse_args;
pub use short_help::help_text;
pub use tui_help::tui_help_document_text;
