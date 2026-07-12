//! RAII terminal session that restores its backend on drop and panic unwind.

use super::{CrosstermBackend, TerminalBackend};

pub(crate) struct TerminalSession<B: TerminalBackend = CrosstermBackend> {
    pub(crate) stdout: B::Writer,
    pub(super) backend: B,
}

impl TerminalSession<CrosstermBackend> {
    pub(crate) fn enter() -> std::io::Result<Self> {
        let (stdout, backend) = CrosstermBackend::enter()?;
        Ok(Self { stdout, backend })
    }
}

impl<B: TerminalBackend> Drop for TerminalSession<B> {
    fn drop(&mut self) {
        self.backend.restore();
    }
}

impl<B: TerminalBackend> TerminalSession<B> {
    pub(crate) fn uses_alternate_screen(&self) -> bool {
        self.backend.uses_alternate_screen()
    }
}
