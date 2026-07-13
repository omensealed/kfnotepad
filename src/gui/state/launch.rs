//! GUI command-line parsing and Iced application startup.

use super::*;

pub fn run() -> iced::Result {
    let launch = match GuiLaunch::from_args(env::args().skip(1).collect()) {
        LaunchAction::Run(launch) => launch,
        LaunchAction::Printed => return Ok(()),
        LaunchAction::Error(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    iced::application(
        move || KfnotepadGui::new_with_task(launch.clone()),
        update,
        view,
    )
    .font(iced_fonts::NERD_FONT_BYTES)
    .title(title)
    .theme(theme)
    .subscription(subscription)
    .window_size(Size::new(1100.0, 720.0))
    .exit_on_close_request(false)
    .centered()
    .run()
}

#[derive(Clone)]
pub(super) struct GuiLaunch {
    pub(super) requested_paths: Vec<PathBuf>,
}

enum LaunchAction {
    Run(GuiLaunch),
    Printed,
    Error(String),
}

impl GuiLaunch {
    fn from_args(args: Vec<String>) -> LaunchAction {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            print_gui_help();
            return LaunchAction::Printed;
        }
        if args.iter().any(|arg| arg == "--version" || arg == "-V") {
            println!("kfnotepad-gui {VERSION}");
            return LaunchAction::Printed;
        }
        if args.iter().any(|arg| arg == "--describe") {
            print_gui_describe();
            return LaunchAction::Printed;
        }
        if let Some(option) = args.iter().find(|arg| arg.starts_with('-')) {
            return LaunchAction::Error(format!("unknown option: {option}"));
        }

        let requested_paths = args.into_iter().map(PathBuf::from).collect();
        LaunchAction::Run(Self { requested_paths })
    }
}

fn print_gui_help() {
    println!("Usage:");
    println!("  kfnotepad-gui [FILE ...]");
    println!("  kfnotepad-gui --describe");
    println!("  kfnotepad-gui --version");
    println!();
    println!("Opens local UTF-8 files as tiled GUI document panes.");
    println!("File open/save validation matches the terminal editor: regular UTF-8 files only.");
    println!(
        "Current controls: Ctrl-N {}, Ctrl-O {}, Ctrl-S {}, Ctrl-Shift-S {}, Ctrl-B {},",
        LABEL_NEW_TILE.to_ascii_lowercase(),
        LABEL_OPEN.to_ascii_lowercase(),
        LABEL_SAVE.to_ascii_lowercase(),
        LABEL_SAVE_AS.to_ascii_lowercase(),
        LABEL_FILES.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-F search, F3/Shift-F3 next/previous, Ctrl-G {}, Ctrl-T app theme,",
        LABEL_GO_TO_LINE.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-Shift-T syntax theme, Ctrl-R reader mode, Ctrl-M {}, Ctrl-Shift-M {},",
        LABEL_MINIMIZE.to_ascii_lowercase(),
        LABEL_MAXIMIZE.to_ascii_lowercase(),
    );
    println!(
        "Ctrl-F4 {}, Ctrl-Q quit, Ctrl-Shift-arrow move tile.",
        LABEL_CLOSE_TILE.to_ascii_lowercase()
    );
    println!("Search is case-insensitive by default; toggle exact-case search in the Find row.");
    println!(
        "Reader mode auto-scrolls the active document at the speed configured in Preferences."
    );
    println!(
        "Preferences also configure app theme, syntax theme, wrapping, line numbers, and fonts."
    );
    println!("Path prompts resolve relative paths from the current file-browser directory.");
    println!(
        "Current browser control: Ctrl-B {}.",
        LABEL_FILES.to_ascii_lowercase(),
    );
}

fn print_gui_describe() {
    println!("kfnotepad-gui tiled Iced editor is available.");
    println!("Run `kfnotepad-gui FILE [FILE...]` to open editable GUI document panes.");
    println!(
        "Safe file behavior: UTF-8 regular files only, symlink/non-regular rejection, atomic saves, save-time external-change conflict checks."
    );
    println!("Layout: resizable tiled panes, compact icon chrome, collapsible left panel.");
    println!("Left panel: Files, Workspaces, and Preferences modes.");
    println!("Persistence: XDG config preferences, workspace projects, and geometry-only layout.");
    println!("Smoke: ./scripts/gui-visual-smoke.sh captures a nonblank local screenshot.");
    println!(
        "Current review gaps: manual desktop dialog coverage, live accessibility review, rich visual regression."
    );
}
