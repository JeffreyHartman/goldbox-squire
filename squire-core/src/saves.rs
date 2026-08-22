//! Reads the party's names from the game's save files.
//!
//! The names are the anchor the scanner searches for. They come from
//! `CHRDATA1.SAV` through `CHRDATA6.SAV`, which the game writes when the player
//! saves. Only the name is taken. Every other number is read live from memory,
//! because a save file goes out of date the moment play continues.

use std::path::Path;

use crate::Error;

/// The most save slots any Gold Box game uses, with room to spare.
const MAX_SLOTS: usize = 10;

/// The party's names, in marching order.
pub fn party_names(game_dir: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    let dir = game_dir.as_ref();
    if !dir.is_dir() {
        return Err(Error::GameFolder(format!(
            "{} is not a folder",
            dir.display()
        )));
    }

    let mut names = Vec::new();
    for slot in 1..=MAX_SLOTS {
        let Some(path) = find_save(dir, slot) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        // A file too short to hold a record is not one. The game writes other
        // CHRDATA files, and a truncated file is not worth failing over.
        if bytes.len() < 16 {
            continue;
        }
        let len = bytes[0] as usize;
        if len == 0 || len > 15 {
            continue;
        }
        names.push(String::from_utf8_lossy(&bytes[1..1 + len]).into_owned());
    }

    if names.is_empty() {
        return Err(Error::GameFolder(format!(
            "no CHRDATA*.SAV files in {}. Save the game once inside it, then try again",
            dir.display()
        )));
    }
    Ok(names)
}

/// Finds one slot's save file, whatever case its name is written in.
fn find_save(dir: &Path, slot: usize) -> Option<std::path::PathBuf> {
    for name in [format!("CHRDATA{slot}.SAV"), format!("chrdata{slot}.sav")] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    // Fall back to a case-insensitive search, for a folder written by something
    // that mixed the case.
    let want = format!("chrdata{slot}.sav");
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        let matches = p.file_name()?.to_str()?.to_ascii_lowercase().eq(&want);
        matches.then_some(p)
    })
}
