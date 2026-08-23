//! Reads the party's names from the game's save files.
//!
//! The names are the anchor the scanner searches for. A save file is
//! `CHRDAT{slot}{index}.SAV`: the save slot is a letter A through J, the
//! character index is the marching-order position 1 through 6. Only the name
//! is taken. Every other number is read live from memory, because a save file
//! goes out of date the moment play continues.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::Error;

/// The save slot letters every Gold Box game uses.
pub const SLOT_LETTERS: [char; 10] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];

/// The most characters a save slot holds.
const MAX_CHARACTERS: usize = 6;

/// One populated save slot: its letter and its party's names in marching order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopulatedSlot {
    pub letter: char,
    pub names: Vec<String>,
}

/// The party names of one save slot, in marching order.
///
/// An empty slot is an error that names the populated slots, so the caller can
/// pass the message straight to the user.
pub fn slot_party_names(save_dir: impl AsRef<Path>, letter: char) -> Result<Vec<String>, Error> {
    let letter = normalize_letter(letter)?;
    let files = save_files(save_dir.as_ref())?;

    let names = read_slot(&files, letter);
    if names.is_empty() {
        let populated: Vec<String> = SLOT_LETTERS
            .iter()
            .filter(|l| !read_slot(&files, **l).is_empty())
            .map(|l| l.to_string())
            .collect();
        let hint = if populated.is_empty() {
            format!(
                "no CHRDAT*.SAV files in {}. Save the game once inside it, then try again",
                save_dir.as_ref().display()
            )
        } else {
            format!(
                "save slot {letter} is empty. Populated slots: {}",
                populated.join(", ")
            )
        };
        return Err(Error::GameFolder(hint));
    }
    Ok(names)
}

/// Every populated save slot of a game folder, in letter order.
///
/// A slot counts as populated when at least one of its `CHRDAT` files parses.
/// The game's own `SAVGAM{slot}.DAT` is deliberately not required: a slot
/// missing it still holds readable characters, and refusing it would be
/// guessing.
pub fn populated_slots(save_dir: impl AsRef<Path>) -> Result<Vec<PopulatedSlot>, Error> {
    let files = save_files(save_dir.as_ref())?;

    let slots: Vec<PopulatedSlot> = SLOT_LETTERS
        .iter()
        .filter_map(|&letter| {
            let names = read_slot(&files, letter);
            (!names.is_empty()).then_some(PopulatedSlot { letter, names })
        })
        .collect();

    if slots.is_empty() {
        return Err(Error::GameFolder(format!(
            "no CHRDAT*.SAV files in {}. Save the game once inside it, then try again",
            save_dir.as_ref().display()
        )));
    }
    Ok(slots)
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

/// One slot's names, in marching order. Empty when the slot is not populated.
fn read_slot(files: &HashMap<String, PathBuf>, letter: char) -> Vec<String> {
    let mut names = Vec::new();
    for index in 1..=MAX_CHARACTERS {
        let key = format!("chrdat{}{index}.sav", letter.to_ascii_lowercase());
        let Some(path) = files.get(&key) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        // A file too short to hold a record is not one. The game writes other
        // CHRDAT files, and a truncated file is not worth failing over.
        if bytes.len() < 16 {
            continue;
        }
        let len = bytes[0] as usize;
        if len == 0 || len > 15 {
            continue;
        }
        names.push(String::from_utf8_lossy(&bytes[1..1 + len]).into_owned());
    }
    names
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
