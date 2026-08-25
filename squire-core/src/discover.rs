//! Finds game installs on disk by their shape.
//!
//! Discovery is structural, not a list of publishers (ADR 0001). An install is
//! a directory holding a conf file with an `[autoexec]` section containing a
//! `mount` line, plus a game folder that holds the game's start file and its
//! save files (of whatever shape that game writes), directly or one level
//! down. Which game the install holds comes from that folder's name, via the
//! registry. Requiring the start file is what keeps the save-only stub every
//! sequel ships for party import from reading as an install. A layout this
//! has never seen fails cleanly to the manual path.

use std::path::{Path, PathBuf};

use crate::games;
use crate::saves;

/// How deep below a search root the walk goes. Both real layouts sit one or
/// two levels down; four leaves room without crawling a whole disk.
const MAX_DEPTH: usize = 4;

/// The publisher whose launch script named the conf order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publisher {
    Gog,
    Steam,
}

/// One install found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredInstall {
    /// The registry id of the game the install holds.
    pub game_id: String,
    /// Who laid the install out, when a launch script said so.
    pub publisher: Option<Publisher>,
    /// The folder the install lives in.
    pub root: PathBuf,
    /// The save folder, relative to `root`.
    pub saves: PathBuf,
}

/// The fixed roots discovery always searches, filtered to the ones that exist.
pub fn default_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return vec![PathBuf::from("/opt")];
    };
    [
        home.join(".local/share/Steam/steamapps/common"),
        home.join(".steam/steam/steamapps/common"),
        home.join("GOG Games"),
        home.join("Games"),
        home.join("gog"),
        home.join("goldbox"),
        PathBuf::from("/opt"),
    ]
    .into_iter()
    .collect()
}

/// Searches the roots and returns every install found, in a stable order.
pub fn discover(roots: &[PathBuf]) -> Vec<DiscoveredInstall> {
    let games = games::games();
    let mut found = Vec::new();

    // Two roots can reach the same folder (`~/.steam/steam` is a symlink into
    // `~/.local/share/Steam`, and both are search roots). Resolving each root
    // to its real path lets the dedup below collapse them.
    let mut resolved: Vec<PathBuf> = roots
        .iter()
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .collect();
    resolved.sort();
    resolved.dedup();

    for root in &resolved {
        let mut dirs = Vec::new();
        collect_dirs(root, 0, &mut dirs);
        for dir in dirs {
            found.extend(examine(&dir, &games));
        }
    }

    found.sort_by(|a, b| a.root.cmp(&b.root).then(a.game_id.cmp(&b.game_id)));
    found.dedup();
    dedup_by_game_folder(found)
}

/// Collapses installs that reach the same game folder into one.
///
/// A hand-written conf next to a publisher's folder makes the parent
/// directory look like a second install of the same game. Since ADR 0003 no
/// install's confs matter, two roots reaching one game folder are one
/// install, and the publisher-scripted reading wins over a `found` one.
fn dedup_by_game_folder(found: Vec<DiscoveredInstall>) -> Vec<DiscoveredInstall> {
    let mut best: Vec<DiscoveredInstall> = Vec::new();
    for install in found {
        let dir = game_folder_identity(&install);
        match best
            .iter_mut()
            .find(|b| b.game_id == install.game_id && game_folder_identity(b) == dir)
        {
            Some(existing) => {
                if existing.publisher.is_none() && install.publisher.is_some() {
                    *existing = install;
                }
            }
            None => best.push(install),
        }
    }
    best.sort_by(|a, b| a.root.cmp(&b.root).then(a.game_id.cmp(&b.game_id)));
    best
}

/// One install's game folder, resolved so two paths to it compare equal.
fn game_folder_identity(install: &DiscoveredInstall) -> PathBuf {
    let dir = install.root.join(&install.saves);
    std::fs::canonicalize(&dir).unwrap_or(dir)
}

/// Every directory from `dir` down, at most `MAX_DEPTH` levels below the root.
fn collect_dirs(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    out.push(dir.to_path_buf());
    if depth == MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // A symlinked directory could loop the walk back on itself, so only
        // real directories are entered. `file_type` does not follow links.
        let is_real_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_real_dir {
            collect_dirs(&entry.path(), depth + 1, out);
        }
    }
}

/// Reads one directory as a possible install root.
fn examine(dir: &Path, games: &[games::Game]) -> Vec<DiscoveredInstall> {
    // A conf holding [autoexec] with a mount is what makes this a game
    // install rather than a folder with stray save files. The conf itself is
    // never launched (ADR 0003); it is only the signature.
    if !has_conf_signature(dir) {
        return Vec::new();
    }

    // One walk serves all twelve games; walking once per game multiplied
    // the whole scan for nothing.
    let mut dirs = Vec::new();
    collect_dirs(dir, 0, &mut dirs);

    let mut found = Vec::new();
    for game in games {
        let Some(saves) = find_saves(&dirs, dir, game) else {
            continue;
        };
        found.push(DiscoveredInstall {
            game_id: game.id.clone(),
            publisher: publisher_of(dir),
            root: dir.to_path_buf(),
            saves,
        });
    }
    found
}

/// Whether `dir` directly holds a `.conf` file with an `[autoexec]` mount.
fn has_conf_signature(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let path = e.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };
        // Read lossily: SNEG's confs echo CP437 box art, which is not valid
        // UTF-8, and a strict read would make the whole install invisible.
        name.to_ascii_lowercase().ends_with(".conf")
            && path.is_file()
            && has_autoexec_mount(&String::from_utf8_lossy(
                &std::fs::read(&path).unwrap_or_default(),
            ))
    })
}

/// The publisher whose launch script sits in the install root.
fn publisher_of(dir: &Path) -> Option<Publisher> {
    if dir.join("start.sh").is_file() {
        return Some(Publisher::Gog);
    }
    if dir.join("run-game.bat").is_file() {
        return Some(Publisher::Steam);
    }
    None
}

/// Whether a conf's `[autoexec]` section contains a `mount` line.
fn has_autoexec_mount(text: &str) -> bool {
    let mut in_autoexec = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_autoexec = line.eq_ignore_ascii_case("[autoexec]");
            continue;
        }
        if in_autoexec && line.to_ascii_lowercase().starts_with("mount") {
            return true;
        }
    }
    false
}

/// The folder holding this game's save files inside a folder named like the
/// game's own DOS folder, relative to the install root.
///
/// GOG keeps the saves in the game folder itself (`data/POOLRAD`). Steam
/// keeps them one level inside it (`GAME/POOLRAD/SAVE`), so a direct child
/// holding save files counts too. Children are tried in sorted order, so
/// the pick is stable. A designs game (Unlimited Adventures) keeps one save
/// folder per design, so its recorded path is the game folder itself and the
/// design is chosen later.
fn find_saves(dirs: &[PathBuf], root: &Path, game: &games::Game) -> Option<PathBuf> {
    for dir in dirs {
        let name = dir.file_name()?.to_str()?;
        if !name.eq_ignore_ascii_case(&game.game_folder) {
            continue;
        }
        // The folder must hold the game, not only its saves. Every sequel
        // ships a stub of its predecessor for party import — a GATEWAY
        // folder holding nothing but SAVE inside Treasures — and reporting
        // that as an install would launch a game that is not there.
        if !holds_the_game(dir, game) {
            continue;
        }
        if let Some(within) = saves_within(dir, game) {
            return dir.join(within).strip_prefix(root).ok().map(Path::to_path_buf);
        }
    }
    None
}

/// Whether this folder holds the game itself: its start file, directly.
///
/// Save files alone do not make a game folder — a sequel's import stub has
/// them too — and a game folder does not need save files to hold the game:
/// a fresh install has none yet.
pub fn holds_the_game(dir: &Path, game: &games::Game) -> bool {
    has_file(dir, &game.start)
}

/// Whether `dir` directly holds a file with this name, whatever case its
/// name is written in.
fn has_file(dir: &Path, name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.file_name()
            .to_str()
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
            && e.path().is_file()
    })
}

/// Where a game directory keeps its saves: the directory itself, or one
/// direct child (Steam nests a `SAVE` folder). Relative to `dir`; empty
/// means the directory itself. `None` means no save files at all.
///
/// This serves the typed-path flow (ADR 0004): the user points at the game
/// folder, and this finds the saves the way discovery would. For a designs
/// game the game folder itself is the answer whenever it holds any design
/// with a save file in it.
pub fn saves_within(dir: &Path, game: &games::Game) -> Option<PathBuf> {
    if game.saves.designs {
        return holds_designs(dir, game).then(PathBuf::new);
    }
    if saves::holds_save_files(game, dir) {
        return Some(PathBuf::new());
    }
    let mut children: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    children.sort();
    children
        .into_iter()
        .find(|child| saves::holds_save_files(game, child))
        .and_then(|child| child.file_name().map(PathBuf::from))
}

/// Whether `dir` holds at least one design with a save file: a `.DSN` child
/// with a `SAVE` folder that holds one. Presence of the file is enough here;
/// whether any slot in it parses is the wizard's design question's job.
fn holds_designs(dir: &Path, game: &games::Game) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let path = e.path();
        let is_dsn = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.to_ascii_uppercase().ends_with(".DSN"));
        is_dsn
            && path.is_dir()
            && std::fs::read_dir(&path).ok().is_some_and(|children| {
                children.flatten().any(|c| {
                    c.file_name().to_string_lossy().eq_ignore_ascii_case("save")
                        && saves::holds_save_files(game, c.path())
                })
            })
    })
}

