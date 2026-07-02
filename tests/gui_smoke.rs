use std::process::Command;

#[test]
fn gui_help_smoke_test() {
    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad-gui"))
        .arg("--help")
        .output()
        .expect("run kfnotepad-gui --help");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("kfnotepad-gui [FILE ...]"));
    assert!(stdout.contains("tiled GUI document panes"));
    assert!(stdout.contains("Ctrl-S save"));
    assert!(stdout.contains("Ctrl-Shift-M maximize"));
    assert!(stdout.contains("Ctrl-F4 close tile"));
    assert!(stdout.contains("Ctrl-R reader mode"));
    assert!(stdout.contains("Ctrl-Shift-T syntax theme"));
    assert!(stdout.contains("case-insensitive by default"));
    assert!(stdout.contains("Reader mode auto-scrolls"));
}

#[test]
fn gui_version_smoke_test() {
    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad-gui"))
        .arg("--version")
        .output()
        .expect("run kfnotepad-gui --version");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(stdout, format!("kfnotepad-gui {}\n", kfnotepad::VERSION));
}

#[test]
fn gui_describe_smoke_test() {
    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad-gui"))
        .arg("--describe")
        .output()
        .expect("run kfnotepad-gui --describe");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("kfnotepad-gui tiled Iced editor is available."));
    assert!(stdout.contains("Run `kfnotepad-gui FILE [FILE...]`"));
    assert!(stdout.contains("Safe file behavior: UTF-8 regular files only"));
    assert!(stdout.contains("Layout: resizable tiled panes, compact icon chrome"));
    assert!(stdout.contains("Left panel: Files, Workspaces, and Preferences modes."));
    assert!(stdout.contains("Persistence: XDG config preferences"));
    assert!(stdout.contains("./scripts/gui-visual-smoke.sh"));
    assert!(stdout.contains("Current review gaps: manual desktop dialog coverage"));
}

#[test]
fn gui_unknown_option_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_kfnotepad-gui"))
        .arg("--unknown")
        .output()
        .expect("run kfnotepad-gui --unknown");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("unknown option: --unknown"));
}
