//! Crossterm setup and best-effort terminal restoration backend.

use std::io::{self, Error, ErrorKind, Write};

use crossterm::cursor::Show;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use super::{editor_keyboard_enhancement_flags, supports_tui_terminal};

pub(crate) trait TerminalBackend {
    type Writer: Write;

    fn enter() -> io::Result<(Self::Writer, Self)>
    where
        Self: Sized;
    fn restore(&mut self);
    fn uses_alternate_screen(&self) -> bool {
        false
    }
}

pub(crate) struct CrosstermBackend {
    keyboard_enhancement_active: bool,
    mouse_capture_active: bool,
    alternate_screen_active: bool,
    bracketed_paste_active: bool,
}

impl TerminalBackend for CrosstermBackend {
    type Writer = io::Stdout;

    fn enter() -> io::Result<(Self::Writer, Self)> {
        if !supports_tui_terminal() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "terminal does not support full-screen TUI mode",
            ));
        }

        enable_raw_mode()?;
        let mut stdout = io::stdout();

        let mut keyboard_enhancement_active = false;
        if supports_keyboard_enhancement().unwrap_or(false)
            && execute!(
                stdout,
                PushKeyboardEnhancementFlags(editor_keyboard_enhancement_flags())
            )
            .is_ok()
        {
            keyboard_enhancement_active = true;
        }

        let mut mouse_capture_active = false;
        if execute!(stdout, EnableMouseCapture).is_ok() {
            mouse_capture_active = true;
        }

        let mut alternate_screen_active = false;
        if execute!(stdout, EnterAlternateScreen).is_ok() {
            alternate_screen_active = true;
        }
        let mut bracketed_paste_active = false;
        if execute!(stdout, EnableBracketedPaste).is_ok() {
            bracketed_paste_active = true;
        }
        let _ = execute!(stdout, Show);

        Ok((
            stdout,
            Self {
                keyboard_enhancement_active,
                mouse_capture_active,
                alternate_screen_active,
                bracketed_paste_active,
            },
        ))
    }

    fn restore(&mut self) {
        let mut stdout = io::stdout();
        if self.mouse_capture_active {
            let _ = execute!(stdout, DisableMouseCapture);
        }
        if self.alternate_screen_active {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }
        if self.keyboard_enhancement_active {
            let _ = execute!(stdout, PopKeyboardEnhancementFlags);
        }
        if self.bracketed_paste_active {
            let _ = execute!(stdout, DisableBracketedPaste);
        }
        let _ = execute!(stdout, Show);
        let _ = disable_raw_mode();
    }

    fn uses_alternate_screen(&self) -> bool {
        self.alternate_screen_active
    }
}
