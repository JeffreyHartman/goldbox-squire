//! Remembers the settings, so that they are given once rather than every run.
//!
//! The config is a map of installs plus the last choice, so two installs never
//! overwrite each other's settings. The save slot is deliberately never
//! stored: a slot describes one sitting, and a remembered slot would pin the
//! user to a save they stopped playing.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Who laid the install out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallKind {
    Gog,
    Steam,
    /// The user named the pieces themselves rather than letting Squire find
    /// them.
    Manual,
}

impl std::fmt::Display for InstallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InstallKind::Gog => "GOG",
            InstallKind::Steam => "Steam",
            InstallKind::Manual => "manual",
        })
    }
}

/// One copy of one game on disk, as a publisher laid it out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Install {
    /// The game's registry id.
    pub game: String,
    pub kind: InstallKind,
    pub root: String,
    /// The save folder, relative to `root`. Empty means the root itself.
    pub saves: String,
    /// The emulator configuration files, in launch order. Later files
    /// override earlier ones, so the order is part of the install.
    pub confs: Vec<String>,
    /// An emulator binary that overrides `dosbox` on PATH.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emulator: Option<String>,
    /// Whether the first-run note naming the conf files was printed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub introduced: bool,
}

impl Install {
    /// The folder the game writes saves into.
    pub fn save_dir(&self) -> PathBuf {
        if self.saves.is_empty() {
            PathBuf::from(&self.root)
        } else {
            PathBuf::from(&self.root).join(&self.saves)
        }
    }
}

/// What is remembered between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The key of the install picked last time, the wizard's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_install: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub installs: BTreeMap<String, Install>,
    /// Extra folders install discovery searches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_roots: Vec<String>,
}

/// Both file versions at once. The v1 keys are read so that an existing
/// user's file migrates on load; they are never written back.
#[derive(Debug, Deserialize)]
struct Raw {
    // v1: one flat game folder.
    game_dir: Option<String>,
    dosbox: Option<String>,
    conf: Option<String>,
    // v2: installs.
    last_install: Option<String>,
    #[serde(default)]
    installs: BTreeMap<String, Install>,
    #[serde(default)]
    extra_roots: Vec<String>,
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
        Config::from_toml(&text).unwrap_or_default()
    }

    /// Parses either file version. A v1 file becomes one manual install, so an
    /// existing user upgrades without typing anything.
    pub fn from_toml(text: &str) -> Result<Config, String> {
        let raw: Raw = toml::from_str(text).map_err(|e| e.to_string())?;

        let mut config = Config {
            last_install: raw.last_install,
            installs: raw.installs,
            extra_roots: raw.extra_roots,
        };

        if config.installs.is_empty() {
            if let Some(game_dir) = raw.game_dir {
                // The old file described Pool of Radiance only, and its
                // game_dir pointed straight at the save folder.
                let key = "manual:pool-of-radiance".to_string();
                config.installs.insert(
                    key.clone(),
                    Install {
                        game: "pool-of-radiance".into(),
                        kind: InstallKind::Manual,
                        root: game_dir,
                        saves: String::new(),
                        confs: raw.conf.into_iter().collect(),
                        emulator: raw.dosbox,
                        introduced: false,
                    },
                );
                config.last_install = Some(key);
            }
        }

        Ok(config)
    }

    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(p) = Config::path() else {
            return Ok(());
        };
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = self
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(p, text)
    }

    /// The install picked last time, when it still exists.
    pub fn last(&self) -> Option<(&String, &Install)> {
        let key = self.last_install.as_ref()?;
        self.installs.get_key_value(key)
    }

    /// Applies a hand-named setup from the command line, and reports whether
    /// that changed what is stored.
    ///
    /// `--game-dir` and `--conf` do not bypass the config; they feed it. The
    /// pair becomes a manual install, listed and remembered like a discovered
    /// one.
    pub fn remember_manual(&mut self, args: &crate::args::Args) -> bool {
        if args.game_dir.is_none() && args.conf.is_none() && args.dosbox.is_none() {
            return false;
        }
        let before = self.clone();
        let game = "pool-of-radiance".to_string();
        let key = format!("manual:{game}");
        let install = self.installs.entry(key.clone()).or_insert(Install {
            game,
            kind: InstallKind::Manual,
            root: String::new(),
            saves: String::new(),
            confs: Vec::new(),
            emulator: None,
            introduced: false,
        });
        if let Some(dir) = &args.game_dir {
            install.root = dir.clone();
        }
        if let Some(conf) = &args.conf {
            install.confs = vec![conf.clone()];
        }
        if let Some(dosbox) = &args.dosbox {
            install.emulator = Some(dosbox.clone());
        }
        self.last_install = Some(key);
        *self != before
    }
}
