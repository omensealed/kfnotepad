use super::*;

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

fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[path = "tests/editor_commands.rs"]
mod editor_commands;
#[path = "tests/editor_workspace_tabs.rs"]
mod editor_workspace_tabs;
#[path = "tests/menu_input_and_wrap.rs"]
mod menu_input_and_wrap;
#[path = "tests/rendering.rs"]
mod rendering;
#[path = "tests/settings_and_preferences.rs"]
mod settings_and_preferences;
#[path = "tests/sidebar_and_projects.rs"]
mod sidebar_and_projects;
