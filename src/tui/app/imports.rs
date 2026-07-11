use std::env;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kfnotepad::{
    list_managed_notes, open_or_create_managed_note, open_text_file, parse_args, summarize_path,
    summarize_text, Command, EditorWorkspace, TextDocument, VERSION,
};

use crate::tui::input::{
    current_tui_restore_project_request, load_tui_workspace_project,
    workspace_from_project_documents,
};
