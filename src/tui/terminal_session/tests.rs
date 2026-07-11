use std::cell::RefCell;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

use super::*;

fn lock_term_env() -> std::sync::MutexGuard<'static, ()> {
    static TERM_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TERM_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("term env mutex")
}

pub(crate) struct FakeBackend {
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl TerminalBackend for FakeBackend {
    type Writer = Vec<u8>;

    fn enter() -> io::Result<(Self::Writer, Self)> {
        let events = Rc::new(RefCell::new(vec!["enter"]));
        Ok((Vec::new(), Self { events }))
    }

    fn restore(&mut self) {
        self.events.borrow_mut().push("restore");
    }
}

impl TerminalSession<FakeBackend> {
    pub(crate) fn enter_fake() -> io::Result<(Self, Rc<RefCell<Vec<&'static str>>>)> {
        let (stdout, backend) = FakeBackend::enter()?;
        let events = Rc::clone(&backend.events);
        Ok((Self { stdout, backend }, events))
    }
}

#[test]
fn terminal_session_restores_backend_on_drop() {
    let (session, events) = TerminalSession::enter_fake().expect("enter fake terminal");

    drop(session);

    assert_eq!(&*events.borrow(), &["enter", "restore"]);
}

#[test]
fn terminal_session_restores_backend_on_panic() {
    let events = {
        let (session, events) = TerminalSession::enter_fake().expect("enter fake terminal");
        let _result = catch_unwind(AssertUnwindSafe(|| {
            let _session = session;
            panic!("simulate editor panic");
        }));
        assert!(_result.is_err());
        events
    };

    assert_eq!(&*events.borrow(), &["enter", "restore"]);
}

#[test]
fn keyboard_enhancement_flags_disambiguate_modified_keys_only() {
    let flags = editor_keyboard_enhancement_flags();

    assert!(flags.contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
    assert!(!flags.contains(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
    assert!(!flags.contains(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES));
}

#[test]
fn supports_tui_terminal_rejects_dumb_terminals() {
    let _guard = lock_term_env();
    let original = env::var_os("TERM");
    env::set_var("TERM", "dumb");

    assert!(!supports_tui_terminal());

    if let Some(value) = original {
        env::set_var("TERM", value);
    } else {
        env::remove_var("TERM");
    }
}

#[test]
fn supports_tui_terminal_rejects_unknown_and_accepts_likely_supported() {
    let _guard = lock_term_env();
    let original = env::var_os("TERM");

    env::set_var("TERM", "unknown");
    assert!(!supports_tui_terminal());

    env::set_var("TERM", "xterm-256color");
    assert!(supports_tui_terminal());

    if let Some(value) = original {
        env::set_var("TERM", value);
    } else {
        env::remove_var("TERM");
    }
}

#[test]
fn terminal_enter_reports_readable_error_on_unsupported_terminal() {
    let _guard = lock_term_env();
    let original = env::var_os("TERM");
    env::set_var("TERM", "unknownterm");

    let error = match CrosstermBackend::enter() {
        Ok(_) => panic!("terminal should reject unsupported mode"),
        Err(error) => error,
    };
    assert!(matches!(
        error.kind(),
        io::ErrorKind::InvalidInput | io::ErrorKind::Other
    ));
    assert!(error
        .to_string()
        .contains("terminal does not support full-screen TUI mode"));

    if let Some(value) = original {
        env::set_var("TERM", value);
    } else {
        env::remove_var("TERM");
    }
}
