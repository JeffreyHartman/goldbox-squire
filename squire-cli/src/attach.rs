//! Resolves what the `--pid` path needs without ever asking.
//!
//! `--pid` reads an emulator this tool did not start. It works only where the
//! system already permits it, and it is the path automation and tests use, so
//! it must never prompt: everything comes from arguments or from the config's
//! remembered values, and a gap is an error naming the missing flag. It also
//! never launches, never touches confs, and never runs install discovery,
//! which is why this module resolves the three values and nothing else.

use std::path::PathBuf;

use squire_core::{games, saves};

use crate::args::Args;
use crate::config::Config;

/// Everything a session against a foreign emulator needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub game_id: String,
    pub game_dir: PathBuf,
    pub slot: char,
    /// The slot's party names, in marching order.
    pub names: Vec<String>,
}

/// Resolves the game, the save folder and the slot, or errors naming the
/// missing flag.
pub fn resolve(args: &Args, config: &Config) -> Result<Resolved, String> {
    let game_id = match &args.game {
        Some(game) => game.clone(),
        None => config
            .last()
            .map(|(_, install)| install.game.clone())
            .ok_or("--pid cannot guess the game. Pass --game <ID>.")?,
    };
    if games::find(&game_id).is_none() {
        let known: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
        return Err(format!(
            "unknown game `{game_id}`. Compiled-in games: {}",
            known.join(", ")
        ));
    }

    let game_dir = match &args.game_dir {
        Some(dir) => PathBuf::from(dir),
        None => install_of(config, &game_id)
            .ok_or("--pid cannot guess the save folder. Pass --game-dir <DIR>.")?,
    };

    let slots = saves::populated_slots(&game_dir).map_err(|e| e.to_string())?;
    let slot = match args.slot {
        Some(letter) if slots.iter().any(|s| s.letter == letter) => letter,
        Some(letter) => {
            return Err(format!(
                "save slot {letter} is empty. Populated slots: {}",
                letters(&slots)
            ))
        }
        // A lone populated slot is the only one the player can have loaded.
        None if slots.len() == 1 => slots[0].letter,
        None => {
            return Err(format!(
                "--pid cannot guess the save slot. Populated slots: {}. \
                 Pass --slot <LETTER>.",
                letters(&slots)
            ))
        }
    };

    let names = slots
        .into_iter()
        .find(|s| s.letter == slot)
        .expect("the slot was validated against this list")
        .names;

    Ok(Resolved {
        game_id,
        game_dir,
        slot,
        names,
    })
}

/// The save folder of a remembered install of this game, preferring the last
/// choice.
fn install_of(config: &Config, game_id: &str) -> Option<PathBuf> {
    if let Some((_, install)) = config.last() {
        if install.game == game_id {
            return Some(install.save_dir());
        }
    }
    config
        .installs
        .values()
        .find(|install| install.game == game_id)
        .map(|install| install.save_dir())
}

fn letters(slots: &[saves::PopulatedSlot]) -> String {
    slots
        .iter()
        .map(|s| s.letter.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
