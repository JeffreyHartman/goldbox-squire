//! The wizard: asks which install and which save slot, then gets out of the
//! way.
//!
//! Two rules keep it unobtrusive. An argument answers its question in advance
//! and the question is skipped. Every question with a remembered or
//! discoverable default accepts Enter, so a returning user is two Enters from
//! a running game.
//!
//! No raw terminal mode: input is plain lines ended by Enter, which is what
//! makes the wizard testable without a terminal.

use std::io::{BufRead, Write};

use squire_core::{games, saves};

use crate::config::Config;

/// What one question came back with.
enum Answer<T> {
    Picked(T),
    /// An empty line with no default, or a lone `b`.
    Back,
}

/// Asks whatever the arguments left unanswered and returns the install key
/// and the save slot.
///
/// The save slot is asked every run and never remembered: a slot describes
/// one sitting.
pub fn choose<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &Config,
    game_arg: Option<&str>,
    slot_arg: Option<char>,
) -> Result<(String, char), String> {
    if config.installs.is_empty() {
        return Err("no installs are known. Point gbs at one with --conf and --game-dir.".into());
    }

    loop {
        let key = match game_arg {
            Some(game) => resolve_game(config, game)?,
            None => match ask_install(input, output, config)? {
                Answer::Picked(key) => key,
                // There is no question before this one; ask it again.
                Answer::Back => continue,
            },
        };

        let install = &config.installs[&key];
        let slots =
            saves::populated_slots(install.save_dir()).map_err(|e| e.to_string())?;

        let slot = match slot_arg {
            Some(letter) => {
                if slots.iter().any(|s| s.letter == letter) {
                    letter
                } else {
                    let populated: Vec<String> =
                        slots.iter().map(|s| s.letter.to_string()).collect();
                    return Err(format!(
                        "save slot {letter} is empty. Populated slots: {}",
                        populated.join(", ")
                    ));
                }
            }
            None => match ask_slot(input, output, &slots)? {
                Answer::Picked(letter) => letter,
                Answer::Back => continue,
            },
        };

        return Ok((key, slot));
    }
}

/// Re-asks the slot question mid-watch.
///
/// Returns the new slot and its party names, or `None` when the user backed
/// out, which means keep watching the slot already chosen.
pub fn repick_slot<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    game_dir: &std::path::Path,
) -> Result<Option<(char, Vec<String>)>, String> {
    let slots = saves::populated_slots(game_dir).map_err(|e| e.to_string())?;
    match ask_slot(input, output, &slots)? {
        Answer::Picked(letter) => {
            let names = slots
                .into_iter()
                .find(|s| s.letter == letter)
                .expect("ask_slot only returns populated letters")
                .names;
            Ok(Some((letter, names)))
        }
        Answer::Back => Ok(None),
    }
}

/// Turns `--game` into an install key without asking anything.
fn resolve_game(config: &Config, game: &str) -> Result<String, String> {
    if games::find(game).is_none() {
        let known: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
        return Err(format!(
            "unknown game `{game}`. Compiled-in games: {}",
            known.join(", ")
        ));
    }
    // The remembered install wins when it holds this game.
    if let Some((key, install)) = config.last() {
        if install.game == game {
            return Ok(key.clone());
        }
    }
    config
        .installs
        .iter()
        .find(|(_, install)| install.game == game)
        .map(|(key, _)| key.clone())
        .ok_or_else(|| format!("no install of `{game}` is known. Run `gbs` to set one up."))
}

/// Question 1: which game? One numbered entry per install.
fn ask_install<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &Config,
) -> Result<Answer<String>, String> {
    let entries: Vec<(&String, &crate::config::Install)> = config.installs.iter().collect();
    let default = config
        .last_install
        .as_ref()
        .and_then(|last| entries.iter().position(|(key, _)| *key == last))
        .or((entries.len() == 1).then_some(0));

    let say = |e: &mut W| -> std::io::Result<()> {
        writeln!(e, "Which game?")?;
        for (n, (_, install)) in entries.iter().enumerate() {
            let name = games::find(&install.game)
                .map(|g| g.name)
                .unwrap_or_else(|| install.game.clone());
            writeln!(e, "  {}. {name} — {} — {}", n + 1, install.kind, install.root)?;
        }
        match default {
            Some(d) => write!(e, "Press Enter for {}, or type a number: ", d + 1),
            None => write!(e, "Type a number: "),
        }?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        let line = read_line(input)?;
        let line = line.trim();
        if line.eq_ignore_ascii_case("b") {
            return Ok(Answer::Back);
        }
        if line.is_empty() {
            match default {
                Some(d) => return Ok(Answer::Picked(entries[d].0.clone())),
                None => return Ok(Answer::Back),
            }
        }
        if let Ok(n) = line.parse::<usize>() {
            if (1..=entries.len()).contains(&n) {
                return Ok(Answer::Picked(entries[n - 1].0.clone()));
            }
        }
        write!(output, "Pick a number between 1 and {}: ", entries.len())
            .and_then(|_| output.flush())
            .map_err(|e| e.to_string())?;
    }
}

/// Question 2: which save slot? One entry per populated slot, with the
/// party's names. Picking by recognising your party beats remembering a
/// letter (ADR 0002).
fn ask_slot<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    slots: &[saves::PopulatedSlot],
) -> Result<Answer<char>, String> {
    let default = slots[0].letter;

    let say = |e: &mut W| -> std::io::Result<()> {
        writeln!(e, "Which save slot?")?;
        for slot in slots {
            writeln!(e, "  {} — {}", slot.letter, slot.names.join(", "))?;
        }
        write!(
            e,
            "Press Enter for {default}, type a letter, or b to go back: "
        )?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        let line = read_line(input)?;
        let line = line.trim();
        if line.eq_ignore_ascii_case("b") {
            return Ok(Answer::Back);
        }
        if line.is_empty() {
            return Ok(Answer::Picked(default));
        }
        let mut chars = line.chars();
        if let (Some(letter), None) = (chars.next(), chars.next()) {
            let letter = letter.to_ascii_uppercase();
            if slots.iter().any(|s| s.letter == letter) {
                return Ok(Answer::Picked(letter));
            }
        }
        let populated: Vec<String> = slots.iter().map(|s| s.letter.to_string()).collect();
        write!(output, "Pick one of {}: ", populated.join(", "))
            .and_then(|_| output.flush())
            .map_err(|e| e.to_string())?;
    }
}

fn read_line<R: BufRead>(input: &mut R) -> Result<String, String> {
    let mut line = String::new();
    let n = input
        .read_line(&mut line)
        .map_err(|e| format!("reading the answer: {e}"))?;
    if n == 0 {
        return Err("the input ended before the questions were answered".into());
    }
    Ok(line)
}
