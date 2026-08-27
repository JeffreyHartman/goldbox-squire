//! Reads the party's names from the game's save files.
//!
//! The names are the anchor the scanner searches for. Only the name is taken.
//! Every other number is read live from memory, because a save file goes out
//! of date the moment play continues.
//!
//! Two shapes exist ([`games::SaveShape`]). Most games write one file per
//! character, `CHRDAT{slot}{index}.{ext}`: the save slot is a letter A
//! through J, the character index is the marching-order position 1 through 6.
//! Unlimited Adventures and The Dark Queen of Krynn write the whole party
//! into one `SAVGAM{slot}.{ext}` file, and Unlimited Adventures keeps one
//! save folder per design: `{design}.DSN/SAVE/` inside the game folder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::games::{Game, SaveShape};
use crate::record;
use crate::Error;

/// The save slot letters every Gold Box game uses.
pub const SLOT_LETTERS: [char; 10] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];

/// The most characters a save slot holds.
const MAX_CHARACTERS: usize = 6;

/// One character of a save slot, as the save picker shows it.
///
/// The level is decoded from the file. That is safe here and nowhere else:
/// the picker describes the save on disk, while the HUD describes the running
/// game, and only the name survives the trip between the two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotCharacter {
    pub name: String,
    /// The highest class level. `None` when the record does not hold one: the
    /// Buck Rogers tables have no per-class levels, and a damaged file decodes
    /// to nothing.
    pub level: Option<u8>,
}

/// One populated save slot: its letter, its party in marching order, and when
/// the slot was last written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopulatedSlot {
    pub letter: char,
    pub party: Vec<SlotCharacter>,
    /// When any of this slot's own save files last changed. This is what tells
    /// two slots holding the same party apart. `None` when no file's time can
    /// be read, which a front end must say rather than dress up as a date.
    pub modified: Option<SystemTime>,
}

impl PopulatedSlot {
    /// The party's names in marching order, which is what the scanner anchors
    /// on.
    pub fn names(&self) -> Vec<String> {
        self.party.iter().map(|c| c.name.clone()).collect()
    }
}

/// One design (adventure module) holding at least one populated save slot.
/// Only Unlimited Adventures has these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Design {
    /// The design's name, `BASILISK` for `BASILISK.DSN`.
    pub name: String,
    /// The design's save folder, absolute.
    pub save_dir: PathBuf,
    /// When a save file in it last changed. Orders the wizard's list, newest
    /// first, so Enter picks the design played last. A design whose time
    /// cannot be read sorts last, because a guess would move the Enter
    /// default onto it.
    pub modified: Option<SystemTime>,
}

/// The party names of one save slot, in marching order.
///
/// An empty slot is an error that names the populated slots, so the caller can
/// pass the message straight to the user.
pub fn slot_party_names(
    game: &Game,
    save_dir: impl AsRef<Path>,
    letter: char,
) -> Result<Vec<String>, Error> {
    let records = slot_party_records(game, save_dir, letter)?;
    Ok(records.into_iter().map(|(name, _)| name).collect())
}

/// The party of one save slot as (name, record bytes) pairs, in marching
/// order. This is the walk everything else builds on, public so a
/// verification tool decodes exactly the records a live session would.
pub fn slot_party_records(
    game: &Game,
    save_dir: impl AsRef<Path>,
    letter: char,
) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let letter = normalize_letter(letter)?;
    let files = save_files(save_dir.as_ref())?;

    let records = read_slot_records(game, &files, letter);
    if records.is_empty() {
        let populated: Vec<String> = SLOT_LETTERS
            .iter()
            .filter(|l| slot_is_populated(game, &files, **l))
            .map(|l| l.to_string())
            .collect();
        let hint = if populated.is_empty() {
            no_saves_here(game, save_dir.as_ref())
        } else {
            format!(
                "save slot {letter} is empty. Populated slots: {}",
                populated.join(", ")
            )
        };
        return Err(Error::GameFolder(hint));
    }
    Ok(records)
}

/// Every populated save slot of a save folder, in letter order.
///
/// A slot counts as populated when at least one character record in it
/// parses. The game's own bookkeeping files are deliberately not required: a
/// slot missing them still holds readable characters, and refusing it would
/// be guessing.
pub fn populated_slots(
    game: &Game,
    save_dir: impl AsRef<Path>,
) -> Result<Vec<PopulatedSlot>, Error> {
    let files = save_files(save_dir.as_ref())?;

    let slots: Vec<PopulatedSlot> = SLOT_LETTERS
        .iter()
        .filter_map(|&letter| {
            let party = read_slot_party(game, &files, letter);
            (!party.is_empty()).then(|| PopulatedSlot {
                letter,
                party,
                modified: slot_modified(game, &files, letter),
            })
        })
        .collect();

    if slots.is_empty() {
        return Err(Error::GameFolder(no_saves_here(game, save_dir.as_ref())));
    }
    Ok(slots)
}

/// Every design with at least one populated save slot, newest save first.
///
/// `game_dir` is the game folder, the one holding the `.DSN` directories.
/// Only a game whose saves live per design has designs; asking any other game
/// is a caller bug worded as an error rather than a panic.
pub fn designs(game: &Game, game_dir: impl AsRef<Path>) -> Result<Vec<Design>, Error> {
    let game_dir = game_dir.as_ref();
    if !game.saves.designs {
        return Err(Error::GameFolder(format!(
            "{} keeps its saves in one folder, not per design",
            game.name
        )));
    }
    if !game_dir.is_dir() {
        return Err(Error::GameFolder(format!(
            "{} is not a folder",
            game_dir.display()
        )));
    }

    let mut found = Vec::new();
    let entries = std::fs::read_dir(game_dir)
        .map_err(|e| Error::GameFolder(format!("cannot list {}: {e}", game_dir.display())))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(name) = file_name
            .to_ascii_uppercase()
            .strip_suffix(".DSN")
            .map(str::to_owned)
        else {
            continue;
        };
        let save_dir = match dir_named(&path, "SAVE") {
            Some(dir) => dir,
            None => continue,
        };
        let Ok(files) = save_files(&save_dir) else {
            continue;
        };
        let populated = SLOT_LETTERS
            .iter()
            .any(|&letter| slot_is_populated(game, &files, letter));
        if !populated {
            continue;
        }
        let modified = newest_save(game, &files);
        found.push(Design {
            name,
            save_dir,
            modified,
        });
    }

    if found.is_empty() {
        return Err(Error::GameFolder(format!(
            "no design under {} holds a saved game. Save the game once inside \
             an adventure, then try again",
            game_dir.display()
        )));
    }
    // Newest first: the design played last is the one the player most likely
    // wants, so it becomes the wizard's Enter default.
    found.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    Ok(found)
}

/// The design with this name, matched case-insensitively, among the designs
/// holding a saved game. The error names the ones that do, so a typo's
/// message is also the answer.
pub fn design_named(game: &Game, game_dir: impl AsRef<Path>, name: &str) -> Result<Design, Error> {
    let designs = designs(game, game_dir)?;
    designs
        .iter()
        .find(|d| d.name.eq_ignore_ascii_case(name))
        .cloned()
        .ok_or_else(|| {
            let known: Vec<&str> = designs.iter().map(|d| d.name.as_str()).collect();
            Error::GameFolder(format!(
                "no design named `{name}` holds a saved game. Designs with \
                 saves: {}",
                known.join(", ")
            ))
        })
}

/// Whether this folder directly holds any save file of this game's shape.
pub fn holds_save_files(game: &Game, dir: impl AsRef<Path>) -> bool {
    let Ok(files) = save_files(dir.as_ref()) else {
        return false;
    };
    let (prefix, ext) = shape_pattern(game);
    files
        .keys()
        .any(|name| name.starts_with(prefix) && name.ends_with(&ext))
}

/// What "no saves" means for this game's shape, worded for the user.
///
/// Files that exist but yield no character are a different fact from no
/// files at all, and telling a user the files are absent when they can see
/// them sends the fix in the wrong direction.
fn no_saves_here(game: &Game, dir: &Path) -> String {
    let (prefix, ext) = shape_pattern(game);
    let pattern = format!(
        "{}*.{}",
        prefix.to_uppercase(),
        ext.trim_start_matches('.').to_uppercase()
    );
    if holds_save_files(game, dir) {
        format!(
            "the {pattern} files in {} hold no readable character. They may \
             be from a fresh install that was never saved in, or damaged. \
             Save the game once inside it, then try again",
            dir.display()
        )
    } else {
        format!(
            "no {pattern} files in {}. Save the game once inside it, then try again",
            dir.display()
        )
    }
}

/// The lowercased filename prefix and dotted extension of this game's saves.
fn shape_pattern(game: &Game) -> (&'static str, String) {
    let ext = format!(".{}", game.saves.extension.to_ascii_lowercase());
    match game.saves.shape {
        SaveShape::Chrdat => ("chrdat", ext),
        SaveShape::PartyFile => ("savgam", ext),
    }
}

fn normalize_letter(letter: char) -> Result<char, Error> {
    let letter = letter.to_ascii_uppercase();
    if !SLOT_LETTERS.contains(&letter) {
        return Err(Error::GameFolder(format!(
            "save slot {letter} does not exist. Slots are the letters A through J"
        )));
    }
    Ok(letter)
}

/// Whether this slot holds at least one readable character.
fn slot_is_populated(game: &Game, files: &HashMap<String, PathBuf>, letter: char) -> bool {
    !read_slot_records(game, files, letter).is_empty()
}

/// One slot's party, in marching order. Empty when the slot is not populated.
///
/// A record too damaged to decode still counts: its name parsed, which is all
/// the slot needs to be real, and the level falls back to zero.
fn read_slot_party(
    game: &Game,
    files: &HashMap<String, PathBuf>,
    letter: char,
) -> Vec<SlotCharacter> {
    read_slot_records(game, files, letter)
        .into_iter()
        .map(|(name, bytes)| SlotCharacter {
            name,
            // A decoded zero means the table found no level field, not a
            // level-zero character: there is no such thing.
            level: record::decode(&game.table, &bytes)
                .ok()
                .map(|c| c.level)
                .filter(|level| *level > 0),
        })
        .collect()
}

/// When this slot's own save files last changed.
///
/// Every file the slot owns counts, whatever its extension: the games rewrite
/// the item and spell files alongside the character record on a save.
fn slot_modified(
    game: &Game,
    files: &HashMap<String, PathBuf>,
    letter: char,
) -> Option<SystemTime> {
    let (prefix, _) = shape_pattern(game);
    let own = format!("{prefix}{}", letter.to_ascii_lowercase());
    newest_of(files, |name| name.starts_with(&own))
}

/// One slot as (name, record bytes) pairs, in marching order. Empty when the
/// slot is not populated.
fn read_slot_records(
    game: &Game,
    files: &HashMap<String, PathBuf>,
    letter: char,
) -> Vec<(String, Vec<u8>)> {
    match game.saves.shape {
        SaveShape::Chrdat => read_chrdat_slot(game, files, letter),
        SaveShape::PartyFile => read_party_file_slot(game, files, letter),
    }
}

/// One slot of the one-file-per-character shape.
fn read_chrdat_slot(
    game: &Game,
    files: &HashMap<String, PathBuf>,
    letter: char,
) -> Vec<(String, Vec<u8>)> {
    let ext = game.saves.extension.to_ascii_lowercase();
    let mut records = Vec::new();
    for index in 1..=MAX_CHARACTERS {
        let key = format!("chrdat{}{index}.{ext}", letter.to_ascii_lowercase());
        let Some(path) = files.get(&key) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        // Only the name is trusted from disk, through the table, so the games
        // that store it mid-record work the same as the ones that start with
        // it. A file too damaged to hold one is skipped, not failed over: the
        // game writes other files with these names, and one bad file is not
        // worth losing the slot.
        if let Some(name) = record::name_at(&game.table, &bytes) {
            records.push((name, bytes));
        }
    }
    records
}

/// One slot of the whole-party-in-one-file shape.
fn read_party_file_slot(
    game: &Game,
    files: &HashMap<String, PathBuf>,
    letter: char,
) -> Vec<(String, Vec<u8>)> {
    let ext = game.saves.extension.to_ascii_lowercase();
    let key = format!("savgam{}.{ext}", letter.to_ascii_lowercase());
    let Some(path) = files.get(&key) else {
        return Vec::new();
    };
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    party_file_records(game, &bytes)
}

/// The party out of one `SAVGAM` file, in marching order.
///
/// The records sit back to back from `first_record_offset`, but each carries
/// a variable tail of item data whose length the file does not state
/// reliably. So the walk never computes a stride: it validates at the current
/// position, and on a miss slides forward one byte. Validation is what keeps
/// the item bytes from ever reading as a character.
///
/// The tail of the file holds stale copies of earlier records. When the
/// table knows where the party size is stored, the count is the guard, and a
/// party may legitimately hold two characters with one name. Only when the
/// size is unknown does the walk fall back to deduplicating by name, which
/// is the best remaining defense against the tail.
fn party_file_records(game: &Game, bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let table = &game.table;
    let size_known = game.saves.party_size_offset.is_some();
    let want = match game.saves.party_size_offset {
        Some(offset) => match bytes.get(offset) {
            Some(&size) => (size as usize).min(MAX_CHARACTERS),
            // A file too short to state its party size holds no party.
            None => return Vec::new(),
        },
        None => MAX_CHARACTERS,
    };

    let mut records: Vec<(String, Vec<u8>)> = Vec::new();
    let mut pos = game.saves.first_record_offset.unwrap_or(0);
    while records.len() < want && pos + table.record_len <= bytes.len() {
        let candidate = &bytes[pos..pos + table.record_len];
        if record::validate(table, candidate).is_ok() {
            if let Some(name) = record::name_at(table, candidate) {
                if size_known || !records.iter().any(|(n, _)| *n == name) {
                    records.push((name, candidate.to_vec()));
                }
            }
            pos += table.record_len;
        } else {
            pos += 1;
        }
    }
    records
}

/// When any of this game's save files in `files` last changed.
fn newest_save(game: &Game, files: &HashMap<String, PathBuf>) -> Option<SystemTime> {
    let (prefix, ext) = shape_pattern(game);
    newest_of(files, |name| {
        name.starts_with(prefix) && name.ends_with(&ext)
    })
}

/// The newest modification time among the files whose name `wanted` accepts,
/// or `None` when nothing matches and when no time can be read.
fn newest_of(
    files: &HashMap<String, PathBuf>,
    wanted: impl Fn(&str) -> bool,
) -> Option<SystemTime> {
    files
        .iter()
        .filter(|(name, _)| wanted(name))
        .filter_map(|(_, path)| path.metadata().ok()?.modified().ok())
        .max()
}

/// A direct child directory with this name, whatever case it is written in.
fn dir_named(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        let matches = path.file_name()?.to_str()?.eq_ignore_ascii_case(name);
        (matches && path.is_dir()).then_some(path)
    })
}

/// Every file in the folder, keyed by its lowercased name.
///
/// DOS wrote upper case, and a folder copied through other systems can be
/// lower or mixed. One directory listing serves every slot lookup.
fn save_files(dir: &Path) -> Result<HashMap<String, PathBuf>, Error> {
    if !dir.is_dir() {
        return Err(Error::GameFolder(format!(
            "{} is not a folder",
            dir.display()
        )));
    }
    let mut files = HashMap::new();
    for entry in std::fs::read_dir(dir)
        .map_err(|e| Error::GameFolder(format!("cannot list {}: {e}", dir.display())))?
        .flatten()
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            files.insert(name.to_ascii_lowercase(), path.clone());
        }
    }
    Ok(files)
}
