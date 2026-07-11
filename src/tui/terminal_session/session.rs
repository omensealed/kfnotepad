pub(crate) struct TerminalSession<B: TerminalBackend = CrosstermBackend> {
    pub(crate) stdout: B::Writer,
    backend: B,
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
