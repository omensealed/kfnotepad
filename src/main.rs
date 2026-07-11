#![forbid(unsafe_code)]

mod tui;

use tui::app;

pub fn main() -> std::process::ExitCode {
    app::run()
}
