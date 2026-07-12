//! Text printing, truncation, status-line composition, and prompt composition.

use super::*;

mod line;
mod printing;
mod prompt;
mod truncate_end;
mod truncate_start;

pub(crate) use line::{compose_status_line, StatusLineRender};
pub(crate) use printing::print_truncated;
pub(crate) use prompt::compose_prompt_status_line;
pub(crate) use truncate_end::fit_text_end;
use truncate_start::fit_text_start;
