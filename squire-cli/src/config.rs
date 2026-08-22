//! Remembers the settings, so that they are given once rather than every run.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// What is remembered between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub game_dir: Option<String>,
    pub dosbox: Option<String>,
    pub conf: Option<String>,
}

impl Config {
    /// Where the config file lives, following the XDG base directory rules.
    pub fn path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("goldbox-squire").join("config.toml"))
    }

    /// Reads the config file. A missing or unreadable file gives the defaults,
    /// because a first run has no file and that is not an error.
    pub fn load() -> Config {
        let Some(p) = Config::path() else {
            return Config::default();
        };
        let Ok(text) = std::fs::read_to_string(p) else {
            return Config::default();
        };
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(p) = Config::path() else {
            return Ok(());
        };
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(p, text)
    }

    /// Applies anything the command line said, and reports whether that changed
    /// what is stored.
    pub fn merge(&mut self, args: &crate::args::Args) -> bool {
        let before = self.clone();
        if args.game_dir.is_some() {
            self.game_dir = args.game_dir.clone();
        }
        if args.dosbox.is_some() {
            self.dosbox = args.dosbox.clone();
        }
        if args.conf.is_some() {
            self.conf = args.conf.clone();
        }
        *self != before
    }
}
