use super::*;
use iced::widget::text_editor;
use kfnotepad::DEFAULT_GUI_FONT_SIZE;
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn numbered_lines(count: usize) -> String {
    (1..=count)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn gui_test_syntax_cache_for_document(
    highlighter: &SyntaxHighlighter,
    document: &TextDocument,
    visible_rows: usize,
) -> GuiSyntaxCache {
    let (highlighted, state) =
        highlighter.highlight_lines_incremental(document, 0, visible_rows, None);
    GuiSyntaxCache {
        path: document.path.clone(),
        line_count: document.buffer.line_count().max(1),
        highlighted_until: highlighted.len(),
        lines: highlighted
            .into_iter()
            .map(|line| {
                line.map(|segments| {
                    gui_syntax_segments_from_syntect(segments, EditorThemeId::Nocturne)
                })
            })
            .collect(),
        state,
    }
}

#[path = "tests/actions_preferences_icons.rs"]
mod actions_preferences_icons;
#[path = "tests/editor_renderer.rs"]
mod editor_renderer;
#[path = "tests/launch_and_file_io.rs"]
mod launch_and_file_io;
#[path = "tests/managed_external_browser.rs"]
mod managed_external_browser;
#[path = "tests/panes_search_menu_layout.rs"]
mod panes_search_menu_layout;
#[path = "tests/workspaces.rs"]
mod workspaces;

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        let root =
            env::temp_dir().join(format!("kfnotepad-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&root).expect("create temp dir");
        let root = root.canonicalize().expect("canonicalize temp dir");
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

fn pane_for_path(state: &KfnotepadGui, path: &PathBuf) -> pane_grid::Pane {
    state
        .panes
        .iter()
        .find_map(|(pane, pane_state)| {
            state
                .workspace
                .tile(pane_state.tile_id)
                .and_then(|tile| (tile.document.path == *path).then_some(*pane))
        })
        .expect("pane for path")
}

fn pane_x(state: &KfnotepadGui, pane: pane_grid::Pane) -> f32 {
    state
        .panes
        .layout()
        .pane_regions(
            GUI_PANE_GRID_SPACING,
            GUI_PANE_GRID_MIN_SIZE,
            GUI_PANE_GRID_REFERENCE_SIZE,
        )
        .get(&pane)
        .expect("pane region")
        .x
}

fn node_path(state: &KfnotepadGui, node: &pane_grid::Node) -> Option<PathBuf> {
    let pane_grid::Node::Pane(pane) = node else {
        return None;
    };
    let tile_id = state.panes.get(*pane)?.tile_id;
    Some(state.workspace.tile(tile_id)?.document.path.clone())
}

fn layout_leaf_ordinals(node: &GuiLayoutNode) -> Vec<usize> {
    match node {
        GuiLayoutNode::Leaf { ordinal } => vec![*ordinal],
        GuiLayoutNode::Split { first, second, .. } => {
            let mut ordinals = layout_leaf_ordinals(first);
            ordinals.extend(layout_leaf_ordinals(second));
            ordinals
        }
    }
}
