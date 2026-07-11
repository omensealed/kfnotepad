use super::*;

use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;
use crate::tui::sidebar::*;
use crate::tui::theme::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crossterm::style::Color;
use kfnotepad::*;
use std::fs;
use std::path::PathBuf;

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kfnotepad-main-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).expect("create temp test directory");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TempArea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_no_temp_files(directory: &Path) {
    let entries = fs::read_dir(directory)
        .expect("read directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect directory entries");
    assert!(
        entries
            .iter()
            .all(|entry| !entry.file_name().to_string_lossy().contains(".kfnotepad-")),
        "temporary file left in {}",
        directory.display()
    );
}

include!("tests/settings_and_preferences.rs");
include!("tests/editor_workspace_tabs.rs");
include!("tests/sidebar_and_projects.rs");
include!("tests/rendering.rs");
include!("tests/menu_input_and_wrap.rs");
include!("tests/editor_commands.rs");
