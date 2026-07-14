use std::fs;
use std::path::PathBuf;

use kfnotepad::{open_text_file, snapshot_text_file, OpenError, MAX_TEXT_FILE_BYTES};

struct TempArea {
    root: PathBuf,
}

impl TempArea {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("kfnotepad-{name}-{}", std::process::id()));
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
fn opens_utf8_file_into_clean_buffer() {
    let temp = TempArea::new("opens-utf8");
    let path = temp.path("note.txt");
    fs::write(&path, "first\nsecond\n").expect("write fixture");

    let document = open_text_file(&path).expect("open text file");

    assert_eq!(document.path, path);
    assert_eq!(
        document.buffer.lines(),
        &["first".to_string(), "second".to_string()]
    );
    assert!(document.buffer.has_trailing_newline());
    assert!(!document.buffer.is_dirty());
}

#[test]
fn opens_empty_file_as_one_editable_line() {
    let temp = TempArea::new("opens-empty");
    let path = temp.path("empty.txt");
    fs::write(&path, "").expect("write empty fixture");

    let document = open_text_file(&path).expect("open empty text file");

    assert_eq!(document.buffer.lines(), &["".to_string()]);
    assert!(!document.buffer.has_trailing_newline());
    assert!(!document.buffer.is_dirty());
}

#[test]
fn rejects_missing_file() {
    let temp = TempArea::new("missing-file");
    let path = temp.path("missing.txt");

    let error = open_text_file(&path).expect_err("missing file should fail");

    assert!(matches!(error, OpenError::Access { .. }));
}

#[test]
fn rejects_directory() {
    let temp = TempArea::new("directory");

    let error = open_text_file(&temp.root).expect_err("directory should fail");

    assert!(matches!(error, OpenError::Directory { .. }));
}

#[cfg(unix)]
#[test]
fn rejects_symlink() {
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("open-symlink");
    let target = temp.path("target.txt");
    let link = temp.path("link.txt");
    fs::write(&target, "linked text\n").expect("write target");
    symlink(&target, &link).expect("create symlink");

    let error = open_text_file(&link).expect_err("symlink open should fail");

    assert!(matches!(error, OpenError::Symlink { .. }));
}

#[cfg(unix)]
#[test]
fn rejects_fifo_without_reading_from_it() {
    use std::process::Command;

    let temp = TempArea::new("open-fifo");
    let path = temp.path("fifo");
    let status = Command::new("mkfifo")
        .arg(&path)
        .status()
        .expect("run mkfifo");
    assert!(status.success(), "mkfifo failed with {status}");

    let error = open_text_file(&path).expect_err("fifo open should fail");

    assert!(matches!(error, OpenError::NotRegular { .. }));
}

#[test]
fn rejects_non_utf8_file() {
    let temp = TempArea::new("non-utf8");
    let path = temp.path("binary.dat");
    fs::write(&path, [0xff, 0xfe, 0xfd]).expect("write fixture");

    let error = open_text_file(&path).expect_err("non-UTF-8 should fail");

    assert!(matches!(error, OpenError::ReadUtf8 { .. }));
}

#[cfg(unix)]
#[test]
fn reports_permission_denied_as_access_error() {
    use std::io;
    use std::os::unix::fs::PermissionsExt;

    let temp = TempArea::new("permission-denied");
    let path = temp.path("private.txt");
    fs::write(&path, "private\n").expect("write fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("remove permissions");

    let result = open_text_file(&path);

    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore permissions");

    match result {
        Err(OpenError::Access { source, .. }) => {
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        Ok(_) => {
            // Some elevated test environments can still read mode 000 files.
        }
        Err(error) => panic!("expected access error for permission denial, got {error}"),
    }
}

#[test]
fn rejects_file_larger_than_open_limit_before_reading() {
    let temp = TempArea::new("too-large");
    let path = temp.path("large.txt");
    let file = fs::File::create(&path).expect("create fixture");
    file.set_len(MAX_TEXT_FILE_BYTES + 1)
        .expect("set sparse fixture length");

    let error = open_text_file(&path).expect_err("large file should fail");

    assert!(matches!(
        error,
        OpenError::TooLarge {
            bytes,
            limit,
            ..
        } if bytes == MAX_TEXT_FILE_BYTES + 1 && limit == MAX_TEXT_FILE_BYTES
    ));
}

#[test]
fn opens_file_exactly_at_open_limit() {
    let temp = TempArea::new("exact-limit");
    let path = temp.path("exact.txt");
    let file = fs::File::create(&path).expect("create exact-limit fixture");
    file.set_len(MAX_TEXT_FILE_BYTES)
        .expect("size exact-limit fixture");
    drop(file);

    let document = open_text_file(&path).expect("exact-limit file should open");

    assert_eq!(document.buffer.to_text().len() as u64, MAX_TEXT_FILE_BYTES);
    assert_eq!(
        document
            .buffer
            .file_snapshot()
            .map(|snapshot| snapshot.bytes),
        Some(MAX_TEXT_FILE_BYTES)
    );
}

#[test]
fn strong_snapshot_rejects_file_above_open_limit() {
    let temp = TempArea::new("snapshot-too-large");
    let path = temp.path("large.txt");
    let file = fs::File::create(&path).expect("create oversized fixture");
    file.set_len(MAX_TEXT_FILE_BYTES + 1)
        .expect("size oversized fixture");
    drop(file);

    let error = snapshot_text_file(&path).expect_err("oversized snapshot should fail");

    assert_eq!(error.kind(), std::io::ErrorKind::FileTooLarge);
    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn strong_snapshot_matches_snapshot_created_by_normal_open() {
    let temp = TempArea::new("snapshot-normal");
    let path = temp.path("note.txt");
    fs::write(&path, "normal snapshot\n").expect("write fixture");

    let document = open_text_file(&path).expect("open fixture");
    let snapshot = snapshot_text_file(&path)
        .expect("snapshot fixture")
        .expect("regular file snapshot");

    assert_eq!(document.buffer.file_snapshot(), Some(&snapshot));
    assert_eq!(snapshot.bytes, "normal snapshot\n".len() as u64);
}
