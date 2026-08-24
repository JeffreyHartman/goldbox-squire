//! The registry of compiled-in games.
//!
//! One TOML file per game holds everything Squire knows about it: the id used
//! in configuration and on the command line, the display name, where its saves
//! live, where its own DOS config sits, and the character record table.
//! Adding a game is adding a file here, not adding code.

use serde::Deserialize;

use crate::table::Table;
use crate::Error;

/// Where a game's own DOS configuration pins its data path.
///
/// Pool of Radiance ships `POOL.CFG`, whose third line reads `C:\POOLRAD\`.
/// The manual-path check reads this to explain a folder-name mismatch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DosConfig {
    /// The config file's name, as the game shipped it.
    pub file: String,
    /// The one-based line holding the DOS data path.
    pub path_line: usize,
}

/// One compiled-in game.
#[derive(Debug, Clone)]
pub struct Game {
    /// The key used in configuration and by `--game`.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The name of the folder the game's saves live in, `POOLRAD` for Pool of
    /// Radiance. Discovery identifies which game an install holds by it.
    pub save_folder: String,
    /// The DOS command that starts the game, run inside `save_folder`.
    pub start: String,
    pub dos_config: DosConfig,
    /// The character record layout.
    pub table: Table,
}

/// The registry half of a game file. The rest is the record table, which
/// [`Table::from_toml`] parses from the same text.
#[derive(Debug, Deserialize)]
struct Meta {
    id: String,
    /// The display name; the record table uses the same key.
    game: String,
    save_folder: String,
    start: String,
    dos_config: DosConfig,
}

const TABLES: &[&str] = &[include_str!("../tables/pool-of-radiance.toml")];

/// Every compiled-in game.
///
/// This cannot fail on a correct build. `Game::from_toml` runs over the same
/// files in the test suite, so a typo fails the tests rather than reaching a
/// user.
pub fn games() -> Vec<Game> {
    TABLES
        .iter()
        .map(|text| Game::from_toml(text).expect("the built-in tables are valid"))
        .collect()
}

/// The game with this id, when it is compiled in.
pub fn find(id: &str) -> Option<Game> {
    games().into_iter().find(|g| g.id == id)
}

impl Game {
    /// Parses one game file and checks that it can drive the tool.
    pub fn from_toml(text: &str) -> Result<Game, Error> {
        let meta: Meta = toml::from_str(text).map_err(|e| Error::Table(e.to_string()))?;
        if meta.id.is_empty() {
            return Err(Error::Table("`id` is empty".into()));
        }
        if meta.save_folder.is_empty() {
            return Err(Error::Table(format!(
                "game `{}` has an empty `save_folder`",
                meta.id
            )));
        }
        if meta.start.is_empty() {
            return Err(Error::Table(format!(
                "game `{}` has an empty `start` command",
                meta.id
            )));
        }
        if meta.dos_config.path_line == 0 {
            return Err(Error::Table(format!(
                "game `{}`: `path_line` is one-based, 0 names no line",
                meta.id
            )));
        }
        let table = Table::from_toml(text)?;
        Ok(Game {
            id: meta.id,
            name: meta.game,
            save_folder: meta.save_folder,
            start: meta.start,
            dos_config: meta.dos_config,
            table,
        })
    }
}
