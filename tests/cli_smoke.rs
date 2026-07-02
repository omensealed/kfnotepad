use std::process::Command;

struct TempArea {
    root: std::path::PathBuf,
}

impl TempArea {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kfnotepad-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create temp test directory");
        Self { root }
    }

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.root.join(name)
    }
}

impl Drop for TempArea {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn help_smoke_test() {
    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad"))
        .arg("--help")
        .output()
        .expect("run kfnotepad --help");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("kfnotepad [FILE]"));
    assert!(stdout.contains("kfnotepad --note TITLE"));
    assert!(stdout.contains("kfnotepad --notes"));
    assert!(stdout.contains("Ctrl-Z undoes"));
    assert!(stdout.contains("Ctrl-Y redoes"));
    assert!(stdout.contains("Ctrl-F searches"));
    assert!(stdout.contains("F3 repeats"));
    assert!(stdout.contains("Shift-F3 repeats"));
    assert!(stdout.contains("Ctrl-G goes to"));
    assert!(stdout.contains("Ctrl-Home and Ctrl-End"));
    assert!(stdout.contains("Ctrl-K deletes"));
    assert!(stdout.contains("Ctrl-Left and Ctrl-Right"));
    assert!(stdout.contains("Ctrl-Backspace and Ctrl-Delete"));
    assert!(stdout.contains("PageUp and PageDown"));
    assert!(stdout.contains("F10 opens the keyboard menu"));
    assert!(stdout.contains("Mouse clicks move the cursor"));
    assert!(stdout.contains("Ctrl-B toggles the file sidebar"));
    assert!(stdout.contains("Ctrl-W toggles word wrap"));
    assert!(stdout.contains("Ctrl-S saves"));
    assert!(stdout.contains("Ctrl-Q quits"));
}

#[test]
fn notes_list_smoke_test_uses_xdg_data_home() {
    let temp = TempArea::new("notes-list");
    let xdg_data_home = temp.path("xdg-data");
    let notes_dir = xdg_data_home.join("kfnotepad").join("notes");
    std::fs::create_dir_all(&notes_dir).expect("create notes dir");
    std::fs::write(notes_dir.join("zeta.md"), "z\n").expect("write note");
    std::fs::write(notes_dir.join("alpha.md"), "a\n").expect("write note");

    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad"))
        .arg("--notes")
        .env("XDG_DATA_HOME", &xdg_data_home)
        .env_remove("HOME")
        .output()
        .expect("run kfnotepad --notes");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(stdout, "alpha.md\nzeta.md\n");
}

#[test]
fn note_smoke_test_creates_managed_note_under_xdg_data_home() {
    let temp = TempArea::new("note-open");
    let xdg_data_home = temp.path("xdg-data");
    let notes_dir = xdg_data_home.join("kfnotepad").join("notes");

    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad"))
        .args(["--note", "Daily Note"])
        .env("XDG_DATA_HOME", &xdg_data_home)
        .env_remove("HOME")
        .output()
        .expect("run kfnotepad --note");

    assert!(output.status.success());
    assert_eq!(
        std::fs::read_to_string(notes_dir.join("daily-note.md")).expect("read created note"),
        ""
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("Opened managed note daily-note.md:"));
    assert!(stdout.contains("0 bytes, 0 lines, trailing_newline=false"));
}
