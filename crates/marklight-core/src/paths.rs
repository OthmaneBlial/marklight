use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use percent_encoding::percent_decode_str;

use crate::{Document, Error, MAX_DOCUMENT_BYTES, Result};

pub fn read_source(reader: impl Read) -> Result<String> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_DOCUMENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|source| Error::Io {
            path: PathBuf::from("<input>"),
            source,
        })?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(Error::TooLarge);
    }
    String::from_utf8(bytes).map_err(|_| Error::Encoding)
}

pub fn resolve_document(path: &Path) -> Result<PathBuf> {
    let chosen = if path.is_dir() {
        ["README.md", "README.markdown", "index.md"]
            .into_iter()
            .map(|name| path.join(name))
            .find(|p| p.is_file())
            .ok_or_else(|| Error::NoReadme(path.to_owned()))?
    } else {
        path.to_owned()
    };
    let absolute = chosen.canonicalize().map_err(|source| Error::Io {
        path: chosen.clone(),
        source,
    })?;
    if !absolute.is_file() {
        return Err(Error::Io {
            path: chosen,
            source: std::io::Error::other("expected a regular file"),
        });
    }
    Ok(absolute)
}

pub fn load_file(path: &Path) -> Result<(PathBuf, Document)> {
    let path = resolve_document(path)?;
    let file = File::open(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let source = read_source(file)?;
    Ok((path, Document::parse(&source)))
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Anchor(String),
    External(String),
    Markdown {
        path: PathBuf,
        anchor: Option<String>,
    },
}

pub fn classify_link(document: &Path, href: &str) -> Result<LinkTarget> {
    let unsafe_link = || Error::UnsafeLink(href.into());
    if href.chars().any(char::is_control) {
        return Err(unsafe_link());
    }
    if let Some(anchor) = href.strip_prefix('#') {
        return Ok(LinkTarget::Anchor(decode(anchor)?));
    }
    if let Ok(url) = url::Url::parse(href) {
        return match url.scheme() {
            "http" | "https" if url.host_str().is_some() => Ok(LinkTarget::External(url.into())),
            "mailto" => Ok(LinkTarget::External(url.into())),
            _ => Err(unsafe_link()),
        };
    }
    let (relative, anchor) = href
        .split_once('#')
        .map_or((href, None), |(p, a)| (p, Some(a)));
    let relative = decode(relative)?;
    if relative.contains([':', '\\', '?']) || relative.starts_with('/') || relative.is_empty() {
        return Err(unsafe_link());
    }
    let path = document.parent().unwrap_or(Path::new(".")).join(&relative);
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !extension.eq_ignore_ascii_case("md") && !extension.eq_ignore_ascii_case("markdown") {
        return Err(unsafe_link());
    }
    Ok(LinkTarget::Markdown {
        path: resolve_document(&path)?,
        anchor: anchor.map(decode).transpose()?,
    })
}

/// Raster files in the document directory tree only. Canonicalize before granting
/// access, so traversal and symlinks cannot read elsewhere on the filesystem.
pub fn resolve_image(document: &Path, href: &str) -> Result<PathBuf> {
    let relative = decode(href)?;
    if relative.contains([':', '\\', '?', '#']) || relative.starts_with('/') {
        return Err(Error::UnsafeLink(href.into()));
    }
    let root = document
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|source| Error::Io {
            path: document.into(),
            source,
        })?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|source| Error::Io {
            path: root.clone(),
            source,
        })?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !path.starts_with(&root)
        || !path.is_file()
        || !["png", "jpg", "jpeg", "gif", "webp", "avif", "bmp"].contains(&ext.as_str())
    {
        return Err(Error::UnsafeLink(href.into()));
    }
    Ok(path)
}

fn decode(value: &str) -> Result<String> {
    let decoded = percent_decode_str(value)
        .decode_utf8()
        .map_err(|_| Error::UnsafeLink(value.into()))?
        .into_owned();
    if decoded.chars().any(char::is_control) {
        return Err(Error::UnsafeLink(value.into()));
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directory_priority_and_friendly_errors() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(
            resolve_document(tmp.path()),
            Err(Error::NoReadme(_))
        ));
        std::fs::write(tmp.path().join("index.md"), "# Index").unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Readme").unwrap();
        assert_eq!(load_file(tmp.path()).unwrap().1.metadata.title, "Readme");
        assert!(matches!(
            load_file(&tmp.path().join("missing.md")),
            Err(Error::Io { .. })
        ));
        assert!(matches!(read_source(&b"\xff"[..]), Err(Error::Encoding)));
        assert!(matches!(
            read_source(std::io::repeat(b'a')),
            Err(Error::TooLarge)
        ));
    }
    #[test]
    fn separates_anchors_external_and_local_links() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("README.md");
        std::fs::write(&path, "# Intro").unwrap();
        assert_eq!(
            classify_link(&path, "#hello%20world").unwrap(),
            LinkTarget::Anchor("hello world".into())
        );
        assert!(matches!(
            classify_link(&path, "https://example.com").unwrap(),
            LinkTarget::External(_)
        ));
        assert!(matches!(
            classify_link(&path, "README.md#intro").unwrap(),
            LinkTarget::Markdown {
                anchor: Some(_),
                ..
            }
        ));
        for href in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "//remote/path",
            "%2fetc/passwd.md",
            "C:\\secret.md",
            "data:text/html,x",
            "README.md%00",
        ] {
            assert!(classify_link(&path, href).is_err(), "{href}");
        }
    }
    #[test]
    fn images_are_scoped_to_raster_files_in_document_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("docs");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("image.png"), b"image").unwrap();
        std::fs::write(tmp.path().join("private.png"), b"private").unwrap();
        std::fs::write(root.join("image.svg"), b"<svg/>").unwrap();
        assert!(resolve_image(&root.join("README.md"), "image.png").is_ok());
        for href in [
            "../private.png",
            "%2e%2e/private.png",
            "image.svg",
            "https://x.com/i.png",
            "/etc/passwd",
        ] {
            assert!(resolve_image(&root.join("README.md"), href).is_err());
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(tmp.path().join("private.png"), root.join("symlink.png"))
                .unwrap();
            assert!(resolve_image(&root.join("README.md"), "symlink.png").is_err());
        }
    }
}
