//! Long-lived, debounced filesystem notifications for open GUI documents.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

pub(super) struct GuiExternalFileWatcher {
    debouncer: Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>,
    receiver: Receiver<DebounceEventResult>,
    watched_directories: HashSet<PathBuf>,
}

#[derive(Default)]
pub(super) struct GuiExternalWatcherDrain {
    pub(super) changed_paths: HashSet<PathBuf>,
    pub(super) error: Option<String>,
}

impl GuiExternalFileWatcher {
    pub(super) fn new() -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        let debouncer =
            new_debouncer(Duration::from_millis(250), sender).map_err(|error| error.to_string())?;
        Ok(Self {
            debouncer,
            receiver,
            watched_directories: HashSet::new(),
        })
    }

    pub(super) fn sync_paths(&mut self, paths: &[PathBuf]) -> Result<(), String> {
        let desired = watched_parent_directories(paths);
        let removed = self
            .watched_directories
            .difference(&desired)
            .cloned()
            .collect::<Vec<_>>();
        let added = desired
            .difference(&self.watched_directories)
            .cloned()
            .collect::<Vec<_>>();

        for directory in removed {
            self.debouncer
                .watcher()
                .unwatch(&directory)
                .map_err(|error| error.to_string())?;
            self.watched_directories.remove(&directory);
        }
        for directory in added {
            self.debouncer
                .watcher()
                .watch(&directory, RecursiveMode::NonRecursive)
                .map_err(|error| error.to_string())?;
            self.watched_directories.insert(directory);
        }
        Ok(())
    }

    pub(super) fn drain(&self) -> GuiExternalWatcherDrain {
        let mut drained = GuiExternalWatcherDrain::default();
        for result in self.receiver.try_iter() {
            match result {
                Ok(events) => {
                    drained
                        .changed_paths
                        .extend(events.into_iter().map(|event| event.path));
                }
                Err(error) => drained.error = Some(error.to_string()),
            }
        }
        drained
    }
}

fn watched_parent_directories(paths: &[PathBuf]) -> HashSet<PathBuf> {
    paths
        .iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect()
}

pub(super) fn watcher_event_matches_path(event_path: &Path, document_path: &Path) -> bool {
    event_path == document_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    struct WatcherTempArea {
        root: PathBuf,
    }

    impl WatcherTempArea {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("kfnotepad-watcher-{}-{unique}", std::process::id()));
            fs::create_dir_all(&root).expect("create watcher temp directory");
            Self { root }
        }
    }

    impl Drop for WatcherTempArea {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn watcher_parent_directories_are_deduplicated() {
        let paths = vec![
            PathBuf::from("/tmp/notes/one.txt"),
            PathBuf::from("/tmp/notes/two.txt"),
            PathBuf::from("/tmp/other/three.txt"),
        ];
        let directories = watched_parent_directories(&paths);
        assert_eq!(directories.len(), 2);
        assert!(directories.contains(Path::new("/tmp/notes")));
        assert!(directories.contains(Path::new("/tmp/other")));
    }

    #[test]
    fn watcher_events_only_match_the_open_document_path() {
        let document = Path::new("/tmp/notes/open.txt");
        assert!(watcher_event_matches_path(document, document));
        assert!(!watcher_event_matches_path(
            Path::new("/tmp/notes/other.txt"),
            document
        ));
    }

    #[test]
    fn watcher_service_reports_changed_file_without_recreation() {
        let temp = WatcherTempArea::new();
        let document = temp.root.join("watched.txt");
        fs::write(&document, "before\n").expect("seed watched file");
        let mut watcher = GuiExternalFileWatcher::new().expect("create watcher");
        watcher
            .sync_paths(std::slice::from_ref(&document))
            .expect("watch document parent");

        fs::write(&document, "after\n").expect("change watched file");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let drained = watcher.drain();
            assert!(
                drained.error.is_none(),
                "watcher error: {:?}",
                drained.error
            );
            if drained.changed_paths.contains(&document) {
                break;
            }
            assert!(Instant::now() < deadline, "watcher event timed out");
            thread::sleep(Duration::from_millis(50));
        }

        assert_eq!(watcher.watched_directories.len(), 1);
    }
}
