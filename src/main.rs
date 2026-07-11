#![forbid(unsafe_code)]

use kfnotepad::tui::app;

pub fn main() -> std::process::ExitCode {
    app::run()
}
