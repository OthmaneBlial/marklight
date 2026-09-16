use std::path::Path;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::Result;

/// Watch the parent directory so atomic editor saves keep working.
pub struct DocumentWatcher {
    _watcher: RecommendedWatcher,
}

impl DocumentWatcher {
    pub fn new(path: &Path, on_change: impl Fn() + Send + 'static) -> Result<Self> {
        let expected = path.canonicalize().map_err(|source| crate::Error::Io {
            path: path.into(),
            source,
        })?;
        let parent = expected.parent().unwrap_or(Path::new(".")).to_owned();
        let mut watcher =
            notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result
                    && matches!(
                        event.kind,
                        EventKind::Any
                            | EventKind::Create(_)
                            | EventKind::Modify(_)
                            | EventKind::Remove(_)
                    )
                    && event.paths.iter().any(|p| p == &expected)
                {
                    on_change();
                }
            })?;
        watcher.watch(&parent, RecursiveMode::NonRecursive)?;
        Ok(Self { _watcher: watcher })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn watches_atomic_replacement_without_access_event_loops() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("README.md");
        std::fs::write(&path, "old").unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let _watcher = DocumentWatcher::new(&path, move || {
            let _ = tx.send(());
        })
        .unwrap();
        let other = tmp.path().join("replacement.tmp");
        std::fs::write(&other, "new").unwrap();
        std::fs::rename(other, &path).unwrap();
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "new");
    }
}
