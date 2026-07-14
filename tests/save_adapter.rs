use std::fs;
use std::path::{Path, PathBuf};

use kfnotepad::{
    open_text_file, save_text_buffer, save_text_document, save_text_snapshot, snapshot_text_file,
    SaveError, TextBuffer, MAX_TEXT_FILE_BYTES,
};

struct TempArea {
    root: PathBuf,
}

#[test]
fn text_snapshot_save_returns_final_snapshot_without_document_clone() {
    let temp = TempArea::new("save-text-snapshot");
    let path = temp.path("note.txt");
    fs::write(&path, "before\n").expect("seed file");
    let document = open_text_file(&path).expect("open seed file");

    let snapshot = save_text_snapshot(&path, "after\n", document.buffer.file_snapshot())
        .expect("save text snapshot");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "after\n"
    );
    assert_eq!(snapshot.bytes, 6);
    assert_eq!(
        snapshot_text_file(&path).expect("snapshot file"),
        Some(snapshot)
    );
}

impl TempArea {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kfnotepad-save-{name}-{}", std::process::id()));
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

#[test]
fn saves_existing_file_by_replacing_contents() {
    let temp = TempArea::new("replace");
    let path = temp.path("note.txt");
    fs::write(&path, "old\n").expect("write original");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");

    save_text_document(&mut document).expect("save document");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "!old\n"
    );
    assert!(!document.buffer.is_dirty());
    assert_no_temp_files(&temp.root);
}

#[test]
fn creates_new_file_with_buffer_contents() {
    let temp = TempArea::new("create");
    let path = temp.path("new.txt");
    let buffer = TextBuffer::from_text("new\ntext\n");

    save_text_buffer(&path, &buffer).expect("save new file");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "new\ntext\n"
    );
    assert_no_temp_files(&temp.root);
}

#[test]
fn rejects_directory_save_target_without_temp_file() {
    let temp = TempArea::new("directory-target");
    let target_dir = temp.path("target");
    fs::create_dir(&target_dir).expect("create target directory");
    let buffer = TextBuffer::from_text("changed\n");

    let error = save_text_buffer(&target_dir, &buffer).expect_err("directory save should fail");

    assert!(matches!(error, SaveError::Directory { .. }));
    assert!(target_dir.is_dir());
    assert_no_temp_files(&temp.root);
}

#[test]
fn missing_parent_directory_fails_before_target_creation() {
    let temp = TempArea::new("missing-parent");
    let missing_parent = temp.path("missing");
    let target = missing_parent.join("note.txt");
    let buffer = TextBuffer::from_text("new\n");

    let error = save_text_buffer(&target, &buffer).expect_err("missing parent should fail");

    assert!(matches!(error, SaveError::CreateTemp { .. }));
    assert!(!target.exists());
    assert!(!missing_parent.exists());
    assert_no_temp_files(&temp.root);
}

#[cfg(unix)]
#[test]
fn rejects_symlink_save_target() {
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("symlink");
    let target = temp.path("target.txt");
    let link = temp.path("link.txt");
    fs::write(&target, "target\n").expect("write target");
    symlink(&target, &link).expect("create symlink");
    let buffer = TextBuffer::from_text("changed\n");

    let error = save_text_buffer(&link, &buffer).expect_err("symlink save should fail");

    assert!(matches!(error, SaveError::Symlink { .. }));
    assert_eq!(
        fs::read_to_string(&target).expect("read target"),
        "target\n"
    );
    assert_no_temp_files(&temp.root);
}

#[cfg(unix)]
#[test]
fn rejects_fifo_save_target_without_temp_file() {
    use std::process::Command;

    let temp = TempArea::new("fifo-target");
    let path = temp.path("fifo");
    let status = Command::new("mkfifo")
        .arg(&path)
        .status()
        .expect("run mkfifo");
    assert!(status.success(), "mkfifo failed with {status}");
    let buffer = TextBuffer::from_text("changed\n");

    let error = save_text_buffer(&path, &buffer).expect_err("fifo save should fail");

    assert!(matches!(error, SaveError::NotRegular { .. }));
    assert_no_temp_files(&temp.root);
}

#[test]
fn save_document_rejects_external_modification_since_open() {
    let temp = TempArea::new("external-modification");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    fs::write(&path, "external\n").expect("external edit");

    let error = save_text_document(&mut document).expect_err("conflicting save should fail");

    assert!(matches!(error, SaveError::ExternalModification { .. }));
    assert_eq!(
        fs::read_to_string(&path).expect("read externally edited file"),
        "external\n"
    );
    assert!(document.buffer.is_dirty());
    assert_no_temp_files(&temp.root);
}

#[test]
fn save_document_rejects_missing_file_since_open() {
    let temp = TempArea::new("external-delete");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    fs::remove_file(&path).expect("delete opened file");

    let error = save_text_document(&mut document).expect_err("deleted target should conflict");

    assert!(matches!(error, SaveError::ExternalModification { .. }));
    assert!(!path.exists());
    assert!(document.buffer.is_dirty());
    assert_no_temp_files(&temp.root);
}

#[test]
fn save_document_rejects_oversized_external_target_without_temp_file() {
    let temp = TempArea::new("external-oversized");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    let file = fs::File::create(&path).expect("replace target");
    file.set_len(MAX_TEXT_FILE_BYTES + 1)
        .expect("grow target beyond limit");
    drop(file);

    let error = save_text_document(&mut document).expect_err("oversized target should conflict");

    assert!(matches!(
        error,
        SaveError::ExternalTargetTooLarge {
            bytes,
            limit,
            ..
        } if bytes == MAX_TEXT_FILE_BYTES + 1 && limit == MAX_TEXT_FILE_BYTES
    ));
    assert_eq!(
        fs::metadata(&path).expect("inspect target").len(),
        MAX_TEXT_FILE_BYTES + 1
    );
    assert!(document.buffer.is_dirty());
    assert_no_temp_files(&temp.root);
}

#[test]
fn rejects_oversized_save_before_creating_temp_file() {
    let temp = TempArea::new("too-large");
    let path = temp.path("large.txt");
    let large_text = "x".repeat(MAX_TEXT_FILE_BYTES as usize + 1);
    let buffer = TextBuffer::from_text(&large_text);

    let error = save_text_buffer(&path, &buffer).expect_err("large save should fail");

    assert!(matches!(
        error,
        SaveError::TooLarge {
            bytes,
            limit,
            ..
        } if bytes == MAX_TEXT_FILE_BYTES + 1 && limit == MAX_TEXT_FILE_BYTES
    ));
    assert!(!path.exists());
    assert_no_temp_files(&temp.root);
}

#[cfg(unix)]
#[test]
fn preserves_existing_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempArea::new("permissions");
    let path = temp.path("note.txt");
    fs::write(&path, "old\n").expect("write original");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).expect("set permissions");
    let buffer = TextBuffer::from_text("new\n");

    save_text_buffer(&path, &buffer).expect("save file");

    let mode = fs::metadata(&path).expect("metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

#[cfg(unix)]
#[test]
fn creates_new_file_with_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempArea::new("new-permissions");
    let path = temp.path("note.txt");
    let buffer = TextBuffer::from_text("new\n");

    save_text_buffer(&path, &buffer).expect("save file");

    let mode = fs::metadata(&path).expect("metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn save_normalizes_crlf_input_to_lf() {
    let temp = TempArea::new("crlf-normalization");
    let path = temp.path("note.txt");
    fs::write(&path, "first\r\nsecond\r\n").expect("write crlf fixture");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");

    save_text_document(&mut document).expect("save normalized document");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "!first\nsecond\n"
    );
}

#[test]
fn save_preserves_lf_trailing_newline_state() {
    let temp = TempArea::new("lf-trailing");
    let path = temp.path("note.txt");
    fs::write(&path, "first\nsecond\n").expect("write lf fixture");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(1, 0, '!').expect("edit buffer");

    save_text_document(&mut document).expect("save document");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "first\n!second\n"
    );
}

#[test]
fn save_preserves_multiple_trailing_newlines() {
    let temp = TempArea::new("multiple-trailing");
    let path = temp.path("note.txt");
    fs::write(&path, "first\n\n").expect("write fixture");
    let mut document = open_text_file(&path).expect("open text file");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");

    save_text_document(&mut document).expect("save document");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "!first\n\n"
    );
}

#[test]
fn pressing_enter_at_eof_with_existing_newline_adds_one_blank_line() {
    let temp = TempArea::new("enter-eof");
    let path = temp.path("note.txt");
    fs::write(&path, "first\n").expect("write fixture");
    let mut document = open_text_file(&path).expect("open text file");

    document
        .buffer
        .insert_newline(0, "first".chars().count())
        .expect("insert newline at eof");
    save_text_document(&mut document).expect("save document");

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "first\n\n"
    );
}

fn assert_no_temp_files(root: &Path) {
    for entry in fs::read_dir(root).expect("read temp directory") {
        let entry = entry.expect("read temp directory entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.contains(".kfnotepad-") && !name.ends_with(".tmp"),
            "unexpected temp file left behind: {name}"
        );
    }
}
