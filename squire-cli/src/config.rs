//! What is remembered between runs.
//!
//! The model is ADR 0004's: an install is a game directory and nothing else,
//! and each game remembers which directory it uses (`chosen`). Launch
//! configuration lives in the per-game settings conf, not here. The file is
//! TOML at the XDG config path; v1 and v2 files migrate on load and are never
//! written back in their old shape.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Who laid the install out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallKind {
    Gog,
    Steam,
    /// Discovery recognized the shape, but no launch script named a publisher.
    Found,
    /// The user named the directory themselves.
    Manual,
}

impl std::fmt::Display for InstallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InstallKind::Gog => "GOG",
            InstallKind::Steam => "Steam",
            InstallKind::Found => "found",
            InstallKind::Manual => "manual",
        })
    }
}

/// One copy of one game on disk: a game directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Install {
    /// The game's registry id.
    pub game: String,
    pub kind: InstallKind,
    pub root: String,
    /// The save folder, relative to `root`. Empty means the root itself.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub saves: String,
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

    /// The install's game folder, resolved so two paths to it compare equal.
    fn folder_identity(&self) -> PathBuf {
        let dir = self.save_dir();
        std::fs::canonicalize(&dir).unwrap_or(dir)
    }
}

/// The size the user left the HUD's window at.
///
/// Global, not per game: window size is a property of where the user sits,
/// not of which game they loaded, and a per-game key would mean fixing twelve
/// entries after changing a monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hud {
    pub columns: u16,
    pub rows: u16,
}

impl Hud {
    /// Whether the stored size could be a window.
    ///
    /// A zero is what a terminal reports when it does not know its own size,
    /// and it reaches the file when a run ends before the first draw. It is
    /// ignored rather than fatal: a stale config file must never be a reason
    /// gbs will not start.
    pub fn is_sane(&self) -> bool {
        self.columns > 0 && self.rows > 0
    }
}

/// What is remembered between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The game picked last time, the game menu's Enter default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_game: Option<String>,
    /// A permanent emulator override. Hand-edited for now; `--dosbox`
    /// overrides it for one run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dosbox: Option<String>,
    /// Which install each game uses: game id to install key. Set the first
    /// time the directory question is answered, then the question is skipped.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub chosen: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub installs: BTreeMap<String, Install>,
    /// Extra folders install discovery searches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_roots: Vec<String>,
    /// The game ids the last discovery ran with. A build that adds a game
    /// changes this list, and that difference is what triggers the rescan
    /// that can find the new game's installs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub known_games: Vec<String>,
    /// The size the HUD's window was left at. Recorded rather than asked:
    /// the wizard asks about things Squire cannot observe, and this is not
    /// one of those.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hud: Option<Hud>,
}

/// Every file version at once. The old keys are read so an existing user's
/// file migrates on load; they are never written back. Unknown keys inside an
/// install (v2's confs and friends) are ignored by serde.
#[derive(Debug, Deserialize)]
struct Raw {
    // v1: one flat game folder. Its `dosbox` matches v3's field and lands
    // there; its `conf` has no successor (ADR 0004) and is dropped.
    game_dir: Option<String>,
    conf: Option<String>,
    // v2: installs plus a single last choice.
    last_install: Option<String>,
    // v3.
    last_game: Option<String>,
    dosbox: Option<String>,
    #[serde(default)]
    chosen: BTreeMap<String, String>,
    #[serde(default)]
    installs: BTreeMap<String, Install>,
    #[serde(default)]
    extra_roots: Vec<String>,
    #[serde(default)]
    known_games: Vec<String>,
    /// Left raw so that a hand-edited nonsense value costs the user their
    /// window size and not the rest of the file.
    hud: Option<toml::Value>,
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

    /// Parses any file version. A v1 file becomes one chosen manual install,
    /// and a v2 file's last choice becomes the game's chosen directory, so an
    /// existing user upgrades without typing anything.
    pub fn from_toml(text: &str) -> Result<Config, String> {
        let raw: Raw = toml::from_str(text).map_err(|e| e.to_string())?;
        let _ = raw.conf; // read so v1 files parse; deliberately unused

        let mut config = Config {
            last_game: raw.last_game,
            dosbox: raw.dosbox,
            chosen: raw.chosen,
            installs: raw.installs,
            extra_roots: raw.extra_roots,
            known_games: raw.known_games,
            hud: raw
                .hud
                .and_then(|v| v.try_into::<Hud>().ok())
                .filter(Hud::is_sane),
        };

        // v2: the single last choice named both the game and its directory.
        if let Some(last) = raw.last_install {
            if let Some(install) = config.installs.get(&last) {
                let game = install.game.clone();
                config.chosen.entry(game.clone()).or_insert(last);
                config.last_game.get_or_insert(game);
            }
        }

        // v1: one flat game folder, Pool of Radiance only, pointing straight
        // at the save folder.
        if config.installs.is_empty() {
            if let Some(game_dir) = raw.game_dir {
                let game = "pool-of-radiance".to_string();
                let key = format!("manual:{game}");
                config.installs.insert(
                    key.clone(),
                    Install {
                        game: game.clone(),
                        kind: InstallKind::Manual,
                        root: game_dir,
                        saves: String::new(),
                    },
                );
                config.chosen.insert(game.clone(), key);
                config.last_game = Some(game);
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

    /// The install a game uses, when one was chosen and still exists.
    pub fn chosen_for(&self, game: &str) -> Option<(&String, &Install)> {
        let key = self.chosen.get(game)?;
        self.installs.get_key_value(key)
    }

    /// Records a user-named game directory and makes it the game's choice.
    /// Returns the install's key.
    pub fn choose_manual_dir(&mut self, game: &str, root: &str, saves: &str) -> String {
        let key = self.key_for(root, game, InstallKind::Manual);
        self.installs.insert(
            key.clone(),
            Install {
                game: game.to_string(),
                kind: InstallKind::Manual,
                root: root.to_string(),
                saves: saves.to_string(),
            },
        );
        self.chosen.insert(game.to_string(), key.clone());
        key
    }

    /// Writes discovery's results into the config, and reports whether that
    /// changed what is stored.
    ///
    /// The results are a cache: a normal run reads them instead of scanning
    /// the disk. A scan is the authority on discovered installs, so everything
    /// non-manual is replaced by what it found. A manual install is the user's
    /// own words and survives, unless its game folder is one a discovered
    /// install already names, in which case it is the same install twice.
    pub fn absorb(&mut self, found: Vec<squire_core::discover::DiscoveredInstall>) -> bool {
        use squire_core::discover::Publisher;

        let before = self.clone();
        self.installs
            .retain(|_, install| install.kind == InstallKind::Manual);

        for d in found {
            let kind = match d.publisher {
                Some(Publisher::Gog) => InstallKind::Gog,
                Some(Publisher::Steam) => InstallKind::Steam,
                None => InstallKind::Found,
            };
            let root = d.root.to_string_lossy().into_owned();
            let key = self.key_for(&root, &d.game_id, kind);
            self.installs.insert(
                key,
                Install {
                    game: d.game_id,
                    kind,
                    root,
                    saves: d.saves.to_string_lossy().into_owned(),
                },
            );
        }

        // A manual directory a discovered install also names is a duplicate.
        let folders: Vec<(String, PathBuf)> = self
            .installs
            .values()
            .filter(|i| i.kind != InstallKind::Manual)
            .map(|i| (i.game.clone(), i.folder_identity()))
            .collect();
        self.installs.retain(|_, install| {
            install.kind != InstallKind::Manual
                || !folders
                    .iter()
                    .any(|(game, dir)| *game == install.game && *dir == install.folder_identity())
        });

        // A choice or default that no longer exists is no choice at all.
        let installs = &self.installs;
        self.chosen.retain(|_, key| installs.contains_key(key));
        *self != before
    }

    /// Whether the cached discovery results went stale. A manual install's
    /// vanished root never triggers a rescan, because a scan cannot find what
    /// the user named by hand.
    pub fn needs_rediscovery(&self) -> bool {
        let discovered: Vec<&Install> = self
            .installs
            .values()
            .filter(|install| install.kind != InstallKind::Manual)
            .collect();
        // Knowing only manual installs is not knowing the disk: a migrated v1
        // config holds one manual entry and must not hide the real installs.
        if discovered.is_empty() {
            return true;
        }
        if discovered
            .iter()
            .any(|install| !PathBuf::from(&install.root).is_dir())
        {
            return true;
        }
        // Two readings of one game folder (discovered twice, or a manual
        // entry naming a discovered folder) are a leftover that only a
        // rescan's dedup can collapse.
        let mut folders: Vec<PathBuf> = self
            .installs
            .values()
            .map(|install| install.folder_identity())
            .collect();
        folders.sort();
        folders.windows(2).any(|pair| pair[0] == pair[1])
    }

    /// The stable key for one install. The same root keeps its key across
    /// rediscoveries; a second install of the same game gets a suffix.
    fn key_for(&self, root: &str, game: &str, kind: InstallKind) -> String {
        if let Some((key, _)) = self
            .installs
            .iter()
            .find(|(_, i)| i.root == root && i.game == game)
        {
            return key.clone();
        }
        let slug = match kind {
            InstallKind::Gog => "gog",
            InstallKind::Steam => "steam",
            InstallKind::Found => "found",
            InstallKind::Manual => "manual",
        };
        let base = format!("{slug}:{game}");
        if !self.installs.contains_key(&base) {
            return base;
        }
        (2..)
            .map(|n| format!("{base}-{n}"))
            .find(|key| !self.installs.contains_key(key))
            .expect("some suffix is free")
    }
}
