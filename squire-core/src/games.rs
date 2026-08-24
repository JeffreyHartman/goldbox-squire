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

/// How a game writes its saves to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveShape {
    /// One file per character: `CHRDAT{slot}{index}.{extension}`. Ten of the
    /// twelve games save this way.
    Chrdat,
    /// The whole party in one file per slot: `SAVGAM{slot}.{extension}`.
    /// Unlimited Adventures and The Dark Queen of Krynn save this way.
    PartyFile,
}

/// The save files of one game: their shape and where they sit.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Saves {
    pub shape: SaveShape,
    /// The save files' extension, `SAV` for most games. Death Knights of
    /// Krynn writes `GAM`, Unlimited Adventures `CSV`, Dark Queen `QSV`.
    pub extension: String,
    /// Whether saves live per design: `{design}.DSN/SAVE/` inside the game
    /// folder. Only Unlimited Adventures, whose adventures are modules.
    #[serde(default)]
    pub designs: bool,
    /// Where a party file stores the party's size, when that is known.
    #[serde(default)]
    pub party_size_offset: Option<usize>,
    /// Where a party file's first character record starts, when known.
    /// Without it the whole file is scanned and validation decides.
    #[serde(default)]
    pub first_record_offset: Option<usize>,
}

/// One compiled-in game.
#[derive(Debug, Clone)]
pub struct Game {
    /// The key used in configuration and by `--game`.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The game's own DOS folder name, `POOLRAD` for Pool of Radiance.
    /// Discovery identifies which game an install holds by it, and the saves
    /// live in it or one level under it.
    pub game_folder: String,
    /// The DOS command that starts the game, run inside `game_folder`.
    pub start: String,
    /// The emulated hardware the game expects, for the settings conf gbs
    /// creates once: `ega` for the EGA-era titles, `svga_s3` for the VGA ones.
    pub machine: String,
    /// The game's own DOS config, when the game ships one this tool knows.
    /// `None` skips the manual-path folder-name check, which is honest:
    /// no check beats a guessed one.
    pub dos_config: Option<DosConfig>,
    pub saves: Saves,
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
    game_folder: String,
    start: String,
    machine: String,
    dos_config: Option<DosConfig>,
    saves: Saves,
}

// In release order, which is also the wizard's menu order.
const TABLES: &[&str] = &[
    include_str!("../tables/pool-of-radiance.toml"),
    include_str!("../tables/curse-of-the-azure-bonds.toml"),
    include_str!("../tables/secret-of-the-silver-blades.toml"),
    include_str!("../tables/pools-of-darkness.toml"),
    include_str!("../tables/champions-of-krynn.toml"),
    include_str!("../tables/death-knights-of-krynn.toml"),
    include_str!("../tables/the-dark-queen-of-krynn.toml"),
    include_str!("../tables/gateway-to-the-savage-frontier.toml"),
    include_str!("../tables/treasures-of-the-savage-frontier.toml"),
    include_str!("../tables/countdown-to-doomsday.toml"),
    include_str!("../tables/matrix-cubed.toml"),
    include_str!("../tables/unlimited-adventures.toml"),
];

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
        if meta.game_folder.is_empty() {
            return Err(Error::Table(format!(
                "game `{}` has an empty `game_folder`",
                meta.id
            )));
        }
        if meta.start.is_empty() {
            return Err(Error::Table(format!(
                "game `{}` has an empty `start` command",
                meta.id
            )));
        }
        if meta.machine.is_empty() {
            return Err(Error::Table(format!(
                "game `{}` has an empty `machine`",
                meta.id
            )));
        }
        if let Some(dos_config) = &meta.dos_config {
            if dos_config.path_line == 0 {
                return Err(Error::Table(format!(
                    "game `{}`: `path_line` is one-based, 0 names no line",
                    meta.id
                )));
            }
        }
        let ext = &meta.saves.extension;
        if ext.is_empty() || ext.contains('.') {
            return Err(Error::Table(format!(
                "game `{}`: the saves `extension` is written without a dot, like `SAV`",
                meta.id
            )));
        }
        if meta.saves.designs && meta.saves.shape != SaveShape::PartyFile {
            return Err(Error::Table(format!(
                "game `{}`: `designs` needs the `party_file` shape",
                meta.id
            )));
        }
        if meta.saves.shape == SaveShape::Chrdat
            && (meta.saves.party_size_offset.is_some() || meta.saves.first_record_offset.is_some())
        {
            return Err(Error::Table(format!(
                "game `{}`: the party file offsets mean nothing to the `chrdat` shape",
                meta.id
            )));
        }
        let table = Table::from_toml(text)?;
        Ok(Game {
            id: meta.id,
            name: meta.game,
            game_folder: meta.game_folder,
            start: meta.start,
            machine: meta.machine,
            dos_config: meta.dos_config,
            saves: meta.saves,
            table,
        })
    }
}
