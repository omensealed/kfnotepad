use super::settings::path_to_hex;
use super::*;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kfnotepad-lib-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create temp area");
        let root = root.canonicalize().expect("canonicalize temp area");
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
    let entries = fs::read_dir(directory).expect("read temp dir");
    for entry in entries {
        let entry = entry.expect("read temp entry");
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.contains(".kfnotepad-"),
            "unexpected temporary config file left behind: {file_name}"
        );
    }
}

mod cli_summary;
mod search;
mod settings_layout_projects;
mod sidebar_gui_models;
mod text_buffer_editing;
mod undo_resource_limits;
mod workspace_commands;
