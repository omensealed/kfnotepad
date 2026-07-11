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
        let root = root
            .canonicalize()
            .expect("canonicalize temp test directory");
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

fn sidebar_fixture(count: usize) -> FileSidebarState {
    FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..count)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 0,
    }
}

#[path = "tests/editor_navigation_editing.rs"]
mod editor_navigation_editing;
#[path = "tests/editor_reader.rs"]
mod editor_reader;
#[path = "tests/editor_save_conflicts.rs"]
mod editor_save_conflicts;
#[path = "tests/editor_search_undo.rs"]
mod editor_search_undo;
#[path = "tests/editor_themes_preferences.rs"]
mod editor_themes_preferences;
#[path = "tests/editor_workspace_tabs.rs"]
mod editor_workspace_tabs;
#[path = "tests/input_paste_modes.rs"]
mod input_paste_modes;
#[path = "tests/input_viewport_wrap.rs"]
mod input_viewport_wrap;
#[path = "tests/menu_commands.rs"]
mod menu_commands;
#[path = "tests/render_chrome_layout.rs"]
mod render_chrome_layout;
#[path = "tests/render_interactions.rs"]
mod render_interactions;
#[path = "tests/render_overlays.rs"]
mod render_overlays;
#[path = "tests/render_unicode_wrap.rs"]
mod render_unicode_wrap;
#[path = "tests/settings_and_preferences.rs"]
mod settings_and_preferences;
#[path = "tests/sidebar_file_operations.rs"]
mod sidebar_file_operations;
#[path = "tests/workspace_manager.rs"]
mod workspace_manager;
#[path = "tests/workspace_persistence.rs"]
mod workspace_persistence;
#[path = "tests/workspace_restore.rs"]
mod workspace_restore;
