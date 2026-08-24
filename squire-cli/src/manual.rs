//! Guards that only the manual install path needs.
//!
//! A discovered install was matched by its game folder's name, so it cannot
//! be mis-named. A hand-named one can, and the game would then read and write
//! somewhere the user is not looking.

use std::path::{Path, PathBuf};

use squire_core::games::Game;

use crate::config::{Install, InstallKind};

/// Refuses a manual install whose save folder does not match the game's own
/// DOS config.
///
/// The game's config pins where it reads data and writes saves: `POOL.CFG`
/// line 3 says `C:\POOLRAD\`. The file name and line come from the game
/// registry. When the folder's name differs, the launched game would ignore
/// this folder, so the mismatch is refused before launching, with both fixes
/// named. A config that is missing or too short proves nothing and passes.
pub fn folder_name_check(install: &Install, game: &Game) -> Result<(), String> {
    if install.kind != InstallKind::Manual {
        return Ok(());
    }
    let save_dir = install.save_dir();
    let Some(folder) = save_dir.file_name().and_then(|n| n.to_str()) else {
        return Ok(());
    };
    let Some(cfg_path) = find_file(&save_dir, &game.dos_config.file) else {
        return Ok(());
    };
    let Ok(text) = std::fs::read_to_string(&cfg_path) else {
        return Ok(());
    };
    let Some(line) = text.lines().nth(game.dos_config.path_line - 1) else {
        return Ok(());
    };
    // The line reads like `C:\POOLRAD\`; its leaf is the folder the game uses.
    let Some(expected) = dos_path_leaf(line) else {
        return Ok(());
    };

    if folder.eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    Err(format!(
        "the game folder is named `{folder}`, but {} (line {}) says the game \
         uses `{}`. The game would read and write `{expected}`, not this \
         folder. Two fixes: rename the folder to `{expected}`, or adjust the \
         conf's mount and {} to point at `{folder}`.",
        cfg_path.display(),
        game.dos_config.path_line,
        line.trim(),
        game.dos_config.file,
    ))
}

/// Finds a file by name in a folder, whatever case its name is written in.
fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.is_file() {
        return Some(direct);
    }
    std::fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        let matches = path
            .file_name()?
            .to_str()?
            .eq_ignore_ascii_case(name);
        (matches && path.is_file()).then_some(path)
    })
}

/// The last folder of a DOS path like `C:\POOLRAD\`.
fn dos_path_leaf(line: &str) -> Option<&str> {
    let trimmed = line.trim().trim_end_matches('\\');
    let leaf = trimmed.rsplit(['\\', '/']).next()?;
    // A drive letter alone (`C:`) or an empty line names no folder.
    (!leaf.is_empty() && !leaf.ends_with(':')).then_some(leaf)
}
