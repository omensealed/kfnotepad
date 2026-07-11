pub(crate) fn load_tui_workspace_project(path: &Path) -> Result<GuiWorkspaceProject, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    parse_gui_workspace_project(&text)
        .ok_or_else(|| format!("invalid workspace project {}", path.display()))
}

pub(crate) fn workspace_from_project_documents(
    project: &GuiWorkspaceProject,
    current_dir: PathBuf,
) -> Result<RestoredTuiWorkspace, String> {
    let mut tabs = Vec::new();
    let mut active = 0usize;
    let mut active_loaded = false;
    let mut skipped_files = Vec::new();

    for (ordinal, path) in project.files.iter().enumerate() {
        let document = match open_text_file(path) {
            Ok(document) => document,
            Err(error) => {
                skipped_files.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        if ordinal == project.active_ordinal {
            active = tabs.len();
            active_loaded = true;
        }
        tabs.push(EditorTab {
            document: EditorTabDocument::Owned(Box::new(document)),
            state: EditorTabState::default(),
        });
    }

    let created_blank = tabs.is_empty();
    if tabs.is_empty() {
        tabs.push(EditorTab {
            document: EditorTabDocument::Owned(Box::new(TextDocument {
                path: current_dir.join("untitled.txt"),
                buffer: kfnotepad::TextBuffer::from_text(""),
            })),
            state: EditorTabState::default(),
        });
        active_loaded = true;
    }
    if !active_loaded {
        active = tabs.len() - 1;
    }
    active = active.min(tabs.len() - 1);
    Ok(RestoredTuiWorkspace {
        project_name: project.name.clone(),
        workspace: EditorWorkspace { tabs, active },
        skipped_files,
        created_blank,
    })
}
