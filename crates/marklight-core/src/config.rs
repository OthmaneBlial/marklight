use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub theme: Theme,
    pub font_size: u8,
    pub toc: bool,
    pub zen_mode: bool,
    pub recent: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            font_size: 16,
            toc: true,
            zen_mode: false,
            recent: Vec::new(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(12..=28).contains(&self.font_size) {
            return Err(Error::Config("font_size must be between 12 and 28".into()));
        }
        if self.recent.len() > 12 {
            return Err(Error::Config("at most 12 recent files are allowed".into()));
        }
        Ok(())
    }
    pub fn remember(&mut self, path: &Path) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.into());
        self.recent.truncate(12);
    }
}

#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn platform() -> Result<Self> {
        let dirs =
            directories::ProjectDirs::from("dev", "Marklight", "Marklight").ok_or_else(|| {
                Error::Config("platform configuration directory is unavailable".into())
            })?;
        Ok(Self::at(dirs.config_dir().join("config.toml")))
    }
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn load(&self) -> Result<Config> {
        let file = match std::fs::File::open(&self.path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(source) => {
                return Err(Error::Io {
                    path: self.path.clone(),
                    source,
                });
            }
        };
        let mut text = String::new();
        file.take(65537)
            .read_to_string(&mut text)
            .map_err(|source| Error::Io {
                path: self.path.clone(),
                source,
            })?;
        if text.len() > 65536 {
            return Err(Error::Config("configuration exceeds 64 KiB".into()));
        }
        let config: Config = toml::from_str(&text).map_err(|e| Error::Config(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }
    pub fn save(&self, config: &Config) -> Result<()> {
        config.validate()?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| Error::Config("configuration has no parent directory".into()))?;
        let io = |source| Error::Io {
            path: self.path.clone(),
            source,
        };
        std::fs::create_dir_all(parent).map_err(io)?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(io)?;
        let text = toml::to_string_pretty(config).map_err(|e| Error::Config(e.to_string()))?;
        temp.write_all(text.as_bytes()).map_err(io)?;
        temp.as_file().sync_all().map_err(io)?;
        temp.persist(&self.path).map_err(|e| io(e.error))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_roundtrip_defaults_validation_and_bounded_recents() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ConfigStore::at(tmp.path().join("config.toml"));
        let mut config = store.load().unwrap();
        assert_eq!(config.theme, Theme::System);
        for i in 0..20 {
            config.remember(Path::new(&format!("{i}.md")));
        }
        config.remember(Path::new("19.md"));
        assert_eq!(config.recent.len(), 12);
        config.theme = Theme::Dark;
        store.save(&config).unwrap();
        assert_eq!(store.load().unwrap().theme, Theme::Dark);
        config.font_size = 1;
        assert!(store.save(&config).is_err());
        std::fs::write(tmp.path().join("config.toml"), "unknown = true").unwrap();
        assert!(store.load().is_err());
    }
}
