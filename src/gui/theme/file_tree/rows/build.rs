//! Recursive file-tree row model construction.

use super::*;

pub(in crate::gui::app::state) fn gui_file_tree_rows(
    root: &Path,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
) -> Vec<GuiFileTreeRowModel> {
    let mut rows = Vec::new();
    push_gui_file_tree_rows(root, 0, expanded_paths, selected_path, &mut rows);
    rows
}

fn push_gui_file_tree_rows(
    path: &Path,
    depth: usize,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
    rows: &mut Vec<GuiFileTreeRowModel>,
) {
    let expanded = expanded_paths.contains(path);
    rows.push(GuiFileTreeRowModel {
        path: path.to_path_buf(),
        label: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
        kind: FileSidebarEntryKind::Directory,
        depth,
        expanded,
        selected: selected_path == Some(path),
        error: false,
    });

    if !expanded || depth >= GUI_FILE_TREE_MAX_DEPTH {
        return;
    }

    let Ok(entries) = list_file_sidebar_entries(path) else {
        rows.push(GuiFileTreeRowModel {
            path: path.to_path_buf(),
            label: "cannot read directory".to_string(),
            kind: FileSidebarEntryKind::File,
            depth: depth + 1,
            expanded: false,
            selected: false,
            error: true,
        });
        return;
    };

    for entry in entries {
        if entry.kind == FileSidebarEntryKind::Parent {
            continue;
        }
        match entry.kind {
            FileSidebarEntryKind::Directory => {
                push_gui_file_tree_rows(&entry.path, depth + 1, expanded_paths, selected_path, rows)
            }
            FileSidebarEntryKind::File => rows.push(GuiFileTreeRowModel {
                selected: selected_path == Some(entry.path.as_path()),
                path: entry.path,
                label: entry.label,
                kind: FileSidebarEntryKind::File,
                depth: depth + 1,
                expanded: false,
                error: false,
            }),
            FileSidebarEntryKind::Parent => {}
        }
    }
}
