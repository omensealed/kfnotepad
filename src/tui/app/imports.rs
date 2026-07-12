use std::env;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;

use kfnotepad::{parse_args, Command, EditorWorkspace, TextDocument, VERSION};
