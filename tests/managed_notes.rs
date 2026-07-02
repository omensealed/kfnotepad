use std::path::{Path, PathBuf};

use kfnotepad::{
    delete_managed_note, list_managed_notes, managed_note_path, managed_notes_dir, note_slug,
    open_or_create_managed_note, ManagedNoteDeleteResult, ManagedNoteEntry, ManagedNotesError,
};

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kfnotepad-managed-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create temp test directory");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TempArea {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn managed_notes_dir_prefers_xdg_data_home() {
    let temp = TempArea::new("xdg");
    let xdg = temp.path("xdg-data");
    let home = temp.path("home");

    let notes_dir = managed_notes_dir(Some(xdg.as_path()), Some(home.as_path()))
        .expect("resolve managed notes directory");

    assert_eq!(notes_dir, xdg.join("kfnotepad").join("notes"));
}

#[test]
fn managed_notes_dir_falls_back_to_home_local_share() {
    let temp = TempArea::new("home");
    let home = temp.path("home");

    let notes_dir =
        managed_notes_dir(None, Some(home.as_path())).expect("resolve managed notes directory");

    assert_eq!(
        notes_dir,
        home.join(".local")
            .join("share")
            .join("kfnotepad")
            .join("notes")
    );
}

#[test]
fn managed_notes_dir_requires_a_base_directory() {
    let error = managed_notes_dir(None, None).expect_err("missing base directory should fail");

    assert!(matches!(error, ManagedNotesError::MissingDataHome));
}

#[test]
fn note_slug_normalizes_titles_to_local_markdown_filenames() {
    assert_eq!(note_slug("Project Plan").expect("slug"), "project-plan.md");
    assert_eq!(
        note_slug("  Rust_API notes  ").expect("slug"),
        "rust-api-notes.md"
    );
    assert_eq!(
        note_slug("release-0.1.0").expect("slug"),
        "release-0-1-0.md"
    );
}

#[test]
fn note_slug_rejects_path_like_or_hidden_names() {
    for title in [
        "",
        "   ",
        ".hidden",
        ".",
        "..",
        "../secret",
        "folder/note",
        "folder\\note",
        "note\nname",
        "!!!",
    ] {
        let error = note_slug(title).expect_err("unsafe title should fail");
        assert!(matches!(error, ManagedNotesError::InvalidNoteName));
    }
}

#[test]
fn managed_note_path_stays_under_notes_directory() {
    let notes_dir = Path::new("/tmp/kfnotepad-notes");

    let path = managed_note_path(notes_dir, "Daily Note").expect("managed note path");

    assert_eq!(path, notes_dir.join("daily-note.md"));
}

#[test]
fn opens_new_managed_note_as_empty_clean_document() {
    let temp = TempArea::new("open-new");
    let notes_dir = temp.path("notes");

    let document =
        open_or_create_managed_note(&notes_dir, "Daily Note").expect("open managed note");

    assert_eq!(document.path, notes_dir.join("daily-note.md"));
    assert_eq!(document.buffer.lines(), &["".to_string()]);
    assert!(!document.buffer.is_dirty());
    assert!(notes_dir.is_dir());
    assert_eq!(
        std::fs::read_to_string(notes_dir.join("daily-note.md")).expect("read note"),
        ""
    );
}

#[test]
fn opens_existing_managed_note_without_overwriting_contents() {
    let temp = TempArea::new("open-existing");
    let notes_dir = temp.path("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(notes_dir.join("daily-note.md"), "existing\n").expect("write existing note");

    let document =
        open_or_create_managed_note(&notes_dir, "Daily Note").expect("open managed note");

    assert_eq!(document.buffer.lines(), &["existing".to_string()]);
    assert!(document.buffer.has_trailing_newline());
    assert_eq!(
        std::fs::read_to_string(notes_dir.join("daily-note.md")).expect("read note"),
        "existing\n"
    );
}

#[test]
fn rejects_invalid_managed_note_title_before_creating_directory() {
    let temp = TempArea::new("reject-invalid");
    let notes_dir = temp.path("notes");

    let error = open_or_create_managed_note(&notes_dir, "../secret")
        .expect_err("invalid title should fail");

    assert!(matches!(error, ManagedNotesError::InvalidNoteName));
    assert!(!notes_dir.exists());
}

#[cfg(unix)]
#[test]
fn creates_new_managed_note_with_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempArea::new("permissions");
    let notes_dir = temp.path("notes");

    open_or_create_managed_note(&notes_dir, "Private").expect("open managed note");

    let mode = std::fs::metadata(notes_dir.join("private.md"))
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn lists_managed_notes_in_filename_order() {
    let temp = TempArea::new("list-order");
    let notes_dir = temp.path("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(notes_dir.join("zeta.md"), "z\n").expect("write note");
    std::fs::write(notes_dir.join("alpha.md"), "a\n").expect("write note");
    std::fs::write(notes_dir.join("middle.md"), "m\n").expect("write note");

    let notes = list_managed_notes(&notes_dir).expect("list notes");

    assert_eq!(
        notes,
        vec![
            ManagedNoteEntry {
                file_name: "alpha.md".to_string(),
                path: notes_dir.join("alpha.md")
            },
            ManagedNoteEntry {
                file_name: "middle.md".to_string(),
                path: notes_dir.join("middle.md")
            },
            ManagedNoteEntry {
                file_name: "zeta.md".to_string(),
                path: notes_dir.join("zeta.md")
            }
        ]
    );
}

#[test]
fn list_managed_notes_ignores_non_note_entries() {
    let temp = TempArea::new("list-filter");
    let notes_dir = temp.path("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(notes_dir.join("alpha.md"), "a\n").expect("write note");
    std::fs::write(notes_dir.join("draft.txt"), "draft\n").expect("write non-note");
    std::fs::write(notes_dir.join(".hidden.md"), "hidden\n").expect("write hidden");
    std::fs::create_dir(notes_dir.join("folder.md")).expect("create directory");

    let notes = list_managed_notes(&notes_dir).expect("list notes");

    assert_eq!(
        notes,
        vec![ManagedNoteEntry {
            file_name: "alpha.md".to_string(),
            path: notes_dir.join("alpha.md")
        }]
    );
}

#[test]
fn list_managed_notes_returns_empty_for_missing_directory() {
    let temp = TempArea::new("list-missing");
    let notes_dir = temp.path("notes");

    let notes = list_managed_notes(&notes_dir).expect("list missing notes dir");

    assert!(notes.is_empty());
    assert!(!notes_dir.exists());
}

#[test]
fn deletes_managed_note_under_notes_directory() {
    let temp = TempArea::new("delete");
    let notes_dir = temp.path("notes");
    let note_path = notes_dir.join("daily.md");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(&note_path, "daily\n").expect("write note");

    let result = delete_managed_note(&notes_dir, &note_path).expect("delete note");

    assert_eq!(result, ManagedNoteDeleteResult::Deleted);
    assert!(!note_path.exists());
}

#[test]
fn delete_managed_note_treats_missing_note_as_removed() {
    let temp = TempArea::new("delete-missing");
    let notes_dir = temp.path("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");

    let result =
        delete_managed_note(&notes_dir, &notes_dir.join("missing.md")).expect("delete missing");

    assert_eq!(result, ManagedNoteDeleteResult::Missing);
}

#[test]
fn delete_managed_note_rejects_paths_outside_notes_directory() {
    let temp = TempArea::new("delete-outside");
    let notes_dir = temp.path("notes");
    let outside_path = temp.path("outside.md");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(&outside_path, "outside\n").expect("write outside");

    let error = delete_managed_note(&notes_dir, &outside_path).expect_err("outside delete fails");

    assert!(matches!(
        error,
        ManagedNotesError::InvalidNotePath { message, .. }
            if message.contains("outside the notes directory")
    ));
    assert!(outside_path.exists());
}

#[test]
fn delete_managed_note_rejects_non_markdown_target() {
    let temp = TempArea::new("delete-non-note");
    let notes_dir = temp.path("notes");
    let text_path = notes_dir.join("draft.txt");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(&text_path, "draft\n").expect("write draft");

    let error = delete_managed_note(&notes_dir, &text_path).expect_err("non-note delete fails");

    assert!(matches!(
        error,
        ManagedNotesError::InvalidNotePath { message, .. }
            if message.contains("visible .md file")
    ));
    assert!(text_path.exists());
}

#[cfg(unix)]
#[test]
fn list_managed_notes_ignores_symlinks() {
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("list-symlink");
    let notes_dir = temp.path("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(notes_dir.join("alpha.md"), "a\n").expect("write note");
    std::fs::write(temp.path("outside.md"), "outside\n").expect("write outside file");
    symlink(temp.path("outside.md"), notes_dir.join("outside.md")).expect("create symlink");

    let notes = list_managed_notes(&notes_dir).expect("list notes");

    assert_eq!(
        notes,
        vec![ManagedNoteEntry {
            file_name: "alpha.md".to_string(),
            path: notes_dir.join("alpha.md")
        }]
    );
}

#[cfg(unix)]
#[test]
fn delete_managed_note_rejects_symlinked_notes() {
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("delete-symlink");
    let notes_dir = temp.path("notes");
    let outside_path = temp.path("outside.md");
    let symlink_path = notes_dir.join("outside.md");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(&outside_path, "outside\n").expect("write outside");
    symlink(&outside_path, &symlink_path).expect("create symlink");

    let error = delete_managed_note(&notes_dir, &symlink_path).expect_err("symlink delete fails");

    assert!(matches!(
        error,
        ManagedNotesError::InvalidNotePath { message, .. }
            if message.contains("symlinked")
    ));
    assert!(outside_path.exists());
    assert!(symlink_path.exists());
}
