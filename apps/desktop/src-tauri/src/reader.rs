use cap_std::{ambient_authority, fs::Dir};
use marklight_core::{
    CodeBlock, Config, ConfigStore, Document, DocumentWatcher, Error, Heading, LinkTarget,
    Metadata, Result, classify_link, load_file, resolve_image,
};
use marklight_render::{HtmlOptions, render_html};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct Payload {
    pub path: String,
    pub name: String,
    pub html: String,
    pub headings: Vec<Heading>,
    pub code_blocks: Vec<CodeBlock>,
    pub metadata: Metadata,
    pub recent: Vec<PathBuf>,
    pub warning: Option<String>,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Navigation {
    Anchor {
        id: String,
    },
    External {
        url: String,
    },
    Markdown {
        path: String,
        anchor: Option<String>,
    },
}

pub struct Reader {
    pub config: Config,
    store: ConfigStore,
    document: Option<(PathBuf, Document)>,
    watcher: Option<DocumentWatcher>,
    image_root: Option<Dir>,
    images: HashMap<String, PathBuf>,
    generation: u64,
    config_warning: Option<String>,
}
impl Reader {
    pub fn new(store: ConfigStore) -> Self {
        let (config, config_warning) = match store.load() {
            Ok(c) => (c, None),
            Err(e) => (Config::default(), Some(e.to_string())),
        };
        Self {
            config,
            store,
            document: None,
            watcher: None,
            image_root: None,
            images: HashMap::new(),
            generation: 0,
            config_warning,
        }
    }
    pub fn open(
        &mut self,
        path: &Path,
        dark: bool,
        changed: impl Fn() + Send + 'static,
    ) -> Result<Payload> {
        let (path, document) = load_file(path)?;
        let image_root = open_image_root(&path)?;
        self.watcher = None;
        let watch_warning = match DocumentWatcher::new(&path, changed) {
            Ok(w) => {
                self.watcher = Some(w);
                None
            }
            Err(e) => Some(e.to_string()),
        };
        self.config.remember(&path);
        let config_warning = self.store.save(&self.config).err().map(|e| e.to_string());
        self.document = Some((path, document));
        self.image_root = Some(image_root);
        Ok(self.payload(dark, watch_warning.or(config_warning)))
    }
    pub fn reload(&mut self, dark: bool) -> Result<Payload> {
        let path = self
            .document
            .as_ref()
            .ok_or_else(|| Error::Config("no document is open".into()))?
            .0
            .clone();
        let (path, document) = load_file(&path)?;
        let image_root = open_image_root(&path)?;
        self.document = Some((path, document));
        self.image_root = Some(image_root);
        Ok(self.payload(dark, None))
    }
    fn payload(&mut self, dark: bool, warning: Option<String>) -> Payload {
        let (path, doc) = self.document.as_ref().unwrap();
        self.generation += 1;
        self.images.clear();
        let mut urls = HashMap::new();
        for (i, link) in doc.links.iter().filter(|link| link.image).enumerate() {
            if let Ok(image) = resolve_image(path, &link.destination) {
                let token = format!("{}-{i}", self.generation);
                urls.insert(
                    link.destination.clone(),
                    format!("marklight-image://localhost/{token}"),
                );
                self.images.insert(token, image);
            }
        }
        let resolve = |href: &str| urls.get(href).cloned();
        let html = render_html(
            doc,
            &HtmlOptions {
                image_url: Some(&resolve),
                dark,
            },
        );
        Payload {
            path: path.to_string_lossy().into_owned(),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            html,
            headings: doc.headings.clone(),
            code_blocks: doc.code_blocks.clone(),
            metadata: doc.metadata.clone(),
            recent: self.config.recent.clone(),
            warning: warning.or_else(|| self.config_warning.take()),
        }
    }
    pub fn save_config(&mut self, config: Config) -> Result<()> {
        self.store.save(&config)?;
        self.config = config;
        Ok(())
    }
    pub fn navigate(&self, href: &str) -> Result<Navigation> {
        let (path, doc) = self
            .document
            .as_ref()
            .ok_or_else(|| Error::Config("no document is open".into()))?;
        if !doc
            .links
            .iter()
            .any(|link| !link.image && link.destination == href)
        {
            return Err(Error::UnsafeLink(href.into()));
        }
        Ok(match classify_link(path, href)? {
            LinkTarget::Anchor(id) => Navigation::Anchor { id },
            LinkTarget::External(url) => Navigation::External { url },
            LinkTarget::Markdown { path, anchor } => Navigation::Markdown {
                path: path.to_string_lossy().into_owned(),
                anchor,
            },
        })
    }
    pub fn image(&self, token: &str) -> Result<(Vec<u8>, &'static str)> {
        let path = self.images.get(token).ok_or_else(|| {
            Error::UnsafeLink("image is outside the current document scope".into())
        })?;
        let document = &self.document.as_ref().unwrap().0;
        let relative = path
            .strip_prefix(document.parent().unwrap())
            .map_err(|_| Error::UnsafeLink("image is outside document directory".into()))?;
        // Open beneath the directory handle captured with the document. A path
        // can be replaced after render, so a second canonicalize + File::open
        // would still permit a symlink swap between authorization and read.
        let root = self.image_root.as_ref().ok_or_else(|| {
            Error::UnsafeLink("image is outside the current document scope".into())
        })?;
        let mut file = root.open(relative).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        if !file
            .metadata()
            .map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?
            .is_file()
        {
            return Err(Error::UnsafeLink("image is not a regular file".into()));
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "bmp" => "image/bmp",
            _ => "image/png",
        };
        use std::io::Read;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::TooLarge);
        }
        Ok((bytes, mime))
    }
}

fn open_image_root(document: &Path) -> Result<Dir> {
    let parent = document.parent().unwrap_or_else(|| Path::new("."));
    Dir::open_ambient_dir(parent, ambient_authority()).map_err(|source| Error::Io {
        path: parent.to_owned(),
        source,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_open_keeps_document_reload_and_recents_work() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("README.md");
        std::fs::write(&path, "# One\n\n[Section](#one)").unwrap();
        let mut reader = Reader::new(ConfigStore::at(tmp.path().join("config.toml")));
        assert_eq!(
            reader.open(&path, false, || {}).unwrap().metadata.title,
            "One"
        );
        assert!(
            reader
                .open(&tmp.path().join("missing.md"), false, || {})
                .is_err()
        );
        assert!(matches!(
            reader.navigate("#one").unwrap(),
            Navigation::Anchor { .. }
        ));
        assert!(reader.navigate("file:///etc/passwd").is_err());
        std::fs::write(&path, "# Two").unwrap();
        let payload = reader.reload(false).unwrap();
        assert_eq!(payload.metadata.title, "Two");
        assert_eq!(payload.recent.len(), 1);
    }
    #[test]
    fn image_tokens_expire_and_cannot_read_arbitrary_files() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("README.md");
        std::fs::write(&path, "![image](a.png)").unwrap();
        std::fs::write(tmp.path().join("a.png"), b"raster").unwrap();
        let mut reader = Reader::new(ConfigStore::at(tmp.path().join("config.toml")));
        let payload = reader.open(&path, false, || {}).unwrap();
        assert!(payload.html.contains("marklight-image://localhost/1-0"));
        assert_eq!(reader.image("1-0").unwrap().0, b"raster");
        assert!(reader.image("../../etc/passwd").is_err());
        reader.reload(false).unwrap();
        assert!(reader.image("1-0").is_err());
        std::fs::write(tmp.path().join("a.png"), vec![0; 16 * 1024 * 1024 + 1]).unwrap();
        assert!(matches!(reader.image("2-0"), Err(Error::TooLarge)));
    }
    #[cfg(unix)]
    #[test]
    fn image_requests_reject_leaf_and_parent_symlink_swaps() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("docs");
        let outside = tmp.path().join("private");
        std::fs::create_dir_all(root.join("images")).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(root.join("README.md"), "![image](images/a.png)").unwrap();
        std::fs::write(root.join("images/a.png"), b"public").unwrap();
        std::fs::write(outside.join("a.png"), b"private").unwrap();
        let mut reader = Reader::new(ConfigStore::at(tmp.path().join("config.toml")));
        reader.open(&root.join("README.md"), false, || {}).unwrap();
        assert_eq!(reader.image("1-0").unwrap().0, b"public");

        std::fs::rename(root.join("images/a.png"), root.join("images/safe.png")).unwrap();
        symlink(outside.join("a.png"), root.join("images/a.png")).unwrap();
        assert!(reader.image("1-0").is_err());
        std::fs::remove_file(root.join("images/a.png")).unwrap();
        std::fs::rename(root.join("images/safe.png"), root.join("images/a.png")).unwrap();

        std::fs::rename(root.join("images"), root.join("old-images")).unwrap();
        symlink(&outside, root.join("images")).unwrap();
        assert!(reader.image("1-0").is_err());
        std::fs::remove_file(root.join("images")).unwrap();
        std::fs::rename(root.join("old-images"), root.join("images")).unwrap();
        assert_eq!(reader.image("1-0").unwrap().0, b"public");

        let moved_root = tmp.path().join("moved-docs");
        std::fs::rename(&root, &moved_root).unwrap();
        symlink(&outside, &root).unwrap();
        assert_eq!(reader.image("1-0").unwrap().0, b"public");
    }
}
