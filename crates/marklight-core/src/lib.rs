//! Shared document engine. No UI or Tauri dependencies.

mod config;
mod document;
mod paths;
mod watch;

pub use config::{Config, ConfigStore, Theme};
pub use document::{CodeBlock, Document, Heading, Link, Metadata};
pub use paths::{
    LinkTarget, classify_link, load_file, read_source, resolve_document, resolve_image,
};
pub use watch::DocumentWatcher;

use std::path::PathBuf;

pub const MAX_DOCUMENT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to open {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("document is not UTF-8; save it as UTF-8 and try again")]
    Encoding,
    #[error("document exceeds the 32 MiB safety limit")]
    TooLarge,
    #[error("no README.md, README.markdown, or index.md found in {0:?}")]
    NoReadme(PathBuf),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("unsafe or unsupported link: {0}")]
    UnsafeLink(String),
    #[error("unable to watch document: {0}")]
    Watch(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
