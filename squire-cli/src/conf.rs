//! The launch configuration gbs owns (ADR 0003).
//!
//! A discovered install is launched with two things gbs controls: a per-game
//! settings conf the user is free to edit, and an autoexec computed fresh
//! each launch and passed as `-c` commands. Publisher conf files are never
//! read; the manual `--conf` path is the way to launch a hand-written one.

use std::path::{Path, PathBuf};

use squire_core::games::Game;

use crate::config::Install;

/// Creates the game's settings conf from the template when it is missing.
///
/// Returns the conf's path and whether this call created it. An existing
/// file is never touched: it belongs to the user from the moment it exists.
pub fn ensure(config_dir: &Path, game: &Game) -> Result<(PathBuf, bool), String> {
    let path = config_dir.join(format!("{}.conf", game.id));
    if path.exists() {
        return Ok((path, false));
    }
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("cannot create {}: {e}", config_dir.display()))?;
    let text = format!(
        "# {} — emulator settings, owned by you.\n\
         #\n\
         # gbs created this file once and will never change it. It layers on\n\
         # top of your emulator's own default configuration, so only put\n\
         # settings here that this game should override.\n\
         #\n\
         # The launch commands (mount, start) are not in here: gbs computes\n\
         # them each run from where it found the game.\n\n\
         [sdl]\n\
         fullscreen = false\n",
        game.name
    );
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok((path, true))
}

/// The DOS commands that start the game, in order.
///
/// The game's own config hardcodes `C:\{folder}\`, so the folder above the
/// game folder is mounted as C. The mount target is computed from the
/// install's recorded save path each launch, so a moved install never
/// launches against a stale path. The final `exit` is what makes quitting
/// the game end the emulator, and with it the watch session.
pub fn autoexec(install: &Install, game: &Game) -> Result<Vec<String>, String> {
    let mount = mount_dir(install, &game.game_folder).ok_or_else(|| {
        format!(
            "the install at {} has no `{}` folder in its recorded save path `{}`. \
             Rerun discovery, or fix the config by hand.",
            install.root, game.game_folder, install.saves
        )
    })?;
    Ok(vec![
        format!("mount c \"{}\"", mount.display()),
        "c:".into(),
        format!("cd {}", game.game_folder),
        game.start.clone(),
        "exit".into(),
    ])
}

/// The folder above the game folder: everything in the install's save path
/// before the component named like the game's own DOS folder.
fn mount_dir(install: &Install, game_folder: &str) -> Option<PathBuf> {
    let saves = Path::new(&install.saves);
    let mut mount = PathBuf::from(&install.root);
    for component in saves.components() {
        let name = component.as_os_str().to_str()?;
        if name.eq_ignore_ascii_case(game_folder) {
            return Some(mount);
        }
        mount.push(name);
    }
    None
}
