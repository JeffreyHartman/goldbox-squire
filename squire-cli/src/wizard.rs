//! The wizard: game, directory, slot (ADR 0004). Unlimited Adventures gets
//! one more question between directory and slot: which design (adventure
//! module), because each design keeps its own saves.
//!
//! Three rules keep it unobtrusive. An argument answers its question in
//! advance and the question is skipped. The game is asked every run (Enter
//! repeats the last one); the directory is asked only until a game has one;
//! the design and the slot are asked every run and never remembered
//! (ADR 0002): both describe one sitting. Back is `0` everywhere: `b` would
//! collide with save slot B.
//!
//! No raw terminal mode: input is plain lines ended by Enter, which is what
//! makes the wizard testable without a terminal.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use squire_core::{discover, games, saves};

use crate::config::{Config, InstallKind};

/// What one question came back with.
enum Answer<T> {
    Picked(T),
    /// A lone `0`, or an empty line with no default.
    Back,
}

/// One sitting: what the wizard resolves beyond the install. The save folder
/// is a design's for a designs game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sitting {
    pub save_dir: PathBuf,
    pub slot: char,
}

/// What the fresh-install escape decided.
enum Escape {
    /// Start the game with no sitting; the save is picked mid-watch.
    Launch,
    /// Back to the game question.
    Back,
}

/// Asks whatever the arguments left unanswered and returns the install key
/// and the sitting. `None` for the sitting means no saved game exists yet
/// and the user chose to launch anyway; the sitting is picked mid-watch
/// after the first in-game save. Records the picks (`last_game`, `chosen`,
/// a typed directory) in the config; the caller saves it.
pub fn choose<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &mut Config,
    game_arg: Option<&str>,
    game_dir_arg: Option<&str>,
    design_arg: Option<&str>,
    slot_arg: Option<char>,
) -> Result<(String, Option<Sitting>), String> {
    loop {
        let game_id = match game_arg {
            Some(id) => validate_game(id)?,
            None => match ask_game(input, output, config)? {
                Answer::Picked(id) => id,
                // There is no question before this one; ask it again.
                Answer::Back => continue,
            },
        };
        config.last_game = Some(game_id.clone());
        let game = games::find(&game_id).expect("the game id was validated");

        let key = match game_dir_arg {
            // --game-dir re-points the game permanently, chosen or not.
            Some(dir) => point_at(config, &game, dir)?,
            None => match config.chosen_for(&game_id) {
                Some((key, _)) => key.clone(),
                None => match ask_dir(input, output, config, &game)? {
                    Answer::Picked(key) => key,
                    Answer::Back => continue,
                },
            },
        };

        // A manual install recorded before its first save may since have
        // saved into a child folder; look again before trusting the record.
        refresh_manual_saves(config, &key, &game);
        let install_save_dir = config.installs[&key].save_dir();

        let save_dir = if game.saves.designs {
            match design_arg {
                // An explicit --design that cannot be honored is an error:
                // a script must not silently launch without its party.
                Some(name) => saves::design_named(&game, &install_save_dir, name)
                    .map(|d| d.save_dir)
                    .map_err(|e| e.to_string())?,
                None => match saves::designs(&game, &install_save_dir) {
                    Ok(designs) => match ask_design(input, output, &designs)? {
                        Answer::Picked(dir) => dir,
                        Answer::Back => continue,
                    },
                    // A fresh install has designs but no saved game in any.
                    Err(reason) => {
                        match no_saves_escape(input, output, slot_arg, &reason.to_string())? {
                            Escape::Launch => return Ok((key, None)),
                            Escape::Back => continue,
                        }
                    }
                },
            }
        } else if let Some(name) = design_arg {
            return Err(format!(
                "--design means nothing to {}: only Unlimited Adventures keeps \
                 saves per design (got `{name}`)",
                game.name
            ));
        } else {
            install_save_dir
        };
        let slots = match saves::populated_slots(&game, &save_dir) {
            Ok(slots) => slots,
            // The fresh-install escape, chrdat shape.
            Err(reason) => match no_saves_escape(input, output, slot_arg, &reason.to_string())? {
                Escape::Launch => return Ok((key, None)),
                Escape::Back => continue,
            },
        };

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

        return Ok((key, Some(Sitting { save_dir, slot })));
    }
}

/// The shared fresh-install decision: no saved game was found. An explicit
/// `--slot` makes the reason a hard error, because a script that named a
/// slot must not silently run without it. Otherwise the user decides:
/// launch without a sitting, or go back. gbs is the launcher, so refusing
/// outright would leave no way to ever create the first save.
fn no_saves_escape<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    slot_arg: Option<char>,
    reason: &str,
) -> Result<Escape, String> {
    if slot_arg.is_some() {
        return Err(reason.to_string());
    }
    if launch_anyway(input, output, reason)? {
        Ok(Escape::Launch)
    } else {
        Ok(Escape::Back)
    }
}

/// Re-derives a manual install's save folder the way the typed path would
/// find it today, and records the correction. A manual install accepted
/// before its first save has an empty save path, and the game may then
/// write its saves into a child folder (Steam-shaped games nest `SAVE`).
fn refresh_manual_saves(config: &mut Config, key: &str, game: &games::Game) {
    let Some(install) = config.installs.get(key) else {
        return;
    };
    if install.kind != InstallKind::Manual || game.saves.designs {
        return;
    }
    if saves::holds_save_files(game, install.save_dir()) {
        return;
    }
    let root = install.root.clone();
    let Some(rel) = discover::saves_within(Path::new(&root), game) else {
        return;
    };
    let rel = rel.to_string_lossy().into_owned();
    if rel != install.saves {
        config.choose_manual_dir(&game.id, &root, &rel);
    }
}

/// The fresh-install escape: no saved game was found, so offer to start the
/// game without one. Enter launches, 0 goes back to the game question.
fn launch_anyway<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    reason: &str,
) -> Result<bool, String> {
    write!(
        output,
        "{reason}.\nStart the game anyway? After saving in game, press Enter \
         in gbs to pick the save.\nPress Enter to start, or 0 to go back: "
    )
    .and_then(|_| output.flush())
    .map_err(|e| e.to_string())?;
    loop {
        let line = read_line(input)?;
        match line.trim() {
            "0" => return Ok(false),
            "" => return Ok(true),
            _ => {
                write!(output, "Press Enter to start, or 0 to go back: ")
                    .and_then(|_| output.flush())
                    .map_err(|e| e.to_string())?;
            }
        }
    }
}

/// Re-asks the sitting mid-watch: the design first for a designs game, then
/// the slot. `install_save_dir` is the install's save folder, which for a
/// designs game is the game folder itself.
///
/// Returns the new slot and its party names, or `None` to keep watching what
/// was already chosen. Finding no saved game is also `None`, with the reason
/// printed: mid-watch, the game is running, and killing the watch over a
/// repick would take the session down with it.
pub fn repick<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    game: &games::Game,
    install_save_dir: &Path,
) -> Result<Option<(char, Vec<String>)>, String> {
    let save_dir = if game.saves.designs {
        match saves::designs(game, install_save_dir) {
            Ok(designs) => match ask_design(input, output, &designs)? {
                Answer::Picked(dir) => dir,
                Answer::Back => return Ok(None),
            },
            Err(reason) => {
                writeln!(output, "{reason}.").map_err(|e| e.to_string())?;
                return Ok(None);
            }
        }
    } else {
        install_save_dir.to_path_buf()
    };
    let slots = match saves::populated_slots(game, &save_dir) {
        Ok(slots) => slots,
        Err(reason) => {
            // A fresh install launched without a sitting may have written
            // its first save into a child folder just now (Steam-shaped
            // games nest SAVE); look once more before giving up.
            let retry = (!game.saves.designs)
                .then(|| discover::saves_within(&save_dir, game))
                .flatten()
                .filter(|rel| !rel.as_os_str().is_empty())
                .and_then(|rel| saves::populated_slots(game, save_dir.join(rel)).ok());
            match retry {
                Some(slots) => slots,
                None => {
                    writeln!(output, "{reason}.").map_err(|e| e.to_string())?;
                    return Ok(None);
                }
            }
        }
    };
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

/// Checks a `--game` id against the registry.
fn validate_game(id: &str) -> Result<String, String> {
    if games::find(id).is_none() {
        let known: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
        return Err(format!(
            "unknown game `{id}`. Compiled-in games: {}",
            known.join(", ")
        ));
    }
    Ok(id.to_string())
}

/// Records a user-named game directory after checking it holds the game.
///
/// Save files prove it the usual way. A folder holding the game's start
/// file but no saves is a fresh install and is accepted too: gbs is the
/// launcher, so the first save can only ever come from launching it.
fn point_at(config: &mut Config, game: &games::Game, dir: &str) -> Result<String, String> {
    let path = Path::new(dir);
    if !path.is_dir() {
        return Err(format!("{dir} is not a folder"));
    }
    let Some(saves) = discover::saves_within(path, game) else {
        if discover::holds_the_game(path, game) {
            return Ok(config.choose_manual_dir(&game.id, dir, ""));
        }
        let looked_for = if game.saves.designs {
            format!(
                "no design with a save file ({{name}}.DSN/SAVE/SAVGAM*.{}) in {dir}",
                game.saves.extension
            )
        } else {
            let prefix = match game.saves.shape {
                games::SaveShape::Chrdat => "CHRDAT*",
                games::SaveShape::PartyFile => "SAVGAM*",
            };
            format!(
                "no {prefix}.{} files in {dir} or one folder inside it",
                game.saves.extension
            )
        };
        return Err(format!(
            "{looked_for}, and no {} either. Point at the game's {} folder.",
            game.start, game.game_folder
        ));
    };
    Ok(config.choose_manual_dir(&game.id, dir, &saves.to_string_lossy()))
}

/// The designs question, asked every run for the one game that has them:
/// which adventure? One entry per design holding a saved game, the one
/// played most recently first, so Enter continues it.
fn ask_design<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    designs: &[saves::Design],
) -> Result<Answer<std::path::PathBuf>, String> {
    let say = |e: &mut W| -> std::io::Result<()> {
        writeln!(e, "Which adventure?")?;
        for (n, design) in designs.iter().enumerate() {
            writeln!(e, "  {}. {}", n + 1, design.name)?;
        }
        write!(
            e,
            "Press Enter for 1, type a number, or 0 to go back: "
        )?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        match read_choice(input, output, designs.len(), Some(0))? {
            Choice::Number(n) => return Ok(Answer::Picked(designs[n - 1].save_dir.clone())),
            Choice::Back => return Ok(Answer::Back),
            Choice::Other => {
                write!(output, "Pick a number between 1 and {}: ", designs.len())
                    .and_then(|_| output.flush())
                    .map_err(|e| e.to_string())?;
            }
        }
    }
}

/// Question 1, every run: which game? Every compiled-in game is listed, even
/// one with no directory yet.
fn ask_game<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &Config,
) -> Result<Answer<String>, String> {
    let entries = games::games();
    let default = config
        .last_game
        .as_ref()
        .and_then(|last| entries.iter().position(|g| g.id == *last))
        .or((entries.len() == 1).then_some(0));

    let say = |e: &mut W| -> std::io::Result<()> {
        writeln!(e, "Which game?")?;
        for (n, game) in entries.iter().enumerate() {
            writeln!(e, "  {}. {}", n + 1, game.name)?;
        }
        match default {
            Some(d) => write!(e, "Press Enter for {}, or type a number: ", d + 1),
            None => write!(e, "Type a number: "),
        }?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        match read_choice(input, output, entries.len(), default)? {
            Choice::Number(n) => return Ok(Answer::Picked(entries[n - 1].id.clone())),
            Choice::Back => return Ok(Answer::Back),
            Choice::Other => {
                write!(output, "Pick a number between 1 and {}: ", entries.len())
                    .and_then(|_| output.flush())
                    .map_err(|e| e.to_string())?;
            }
        }
    }
}

/// Question 2, until a game has a directory: where is the game? The
/// discovered directories, plus a typed path for one discovery missed.
/// The pick is remembered, and this question is then skipped.
fn ask_dir<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &mut Config,
    game: &games::Game,
) -> Result<Answer<String>, String> {
    let dirs: Vec<(String, String)> = config
        .installs
        .iter()
        .filter(|(_, install)| install.game == game.id)
        .map(|(key, install)| (key.clone(), format!("{} — {}", install.kind, install.root)))
        .collect();
    let elsewhere = dirs.len() + 1;
    let default = (!dirs.is_empty()).then_some(0);

    let say = |e: &mut W| -> std::io::Result<()> {
        writeln!(e, "Where is {}? (gbs remembers this)", game.name)?;
        for (n, (_, label)) in dirs.iter().enumerate() {
            writeln!(e, "  {}. {label}", n + 1)?;
        }
        writeln!(e, "  {elsewhere}. Somewhere else (type a path)")?;
        match default {
            Some(d) => write!(e, "Press Enter for {}, type a number, or 0 to go back: ", d + 1),
            None => write!(e, "Type a number, or 0 to go back: "),
        }?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        match read_choice(input, output, elsewhere, default)? {
            Choice::Number(n) if n < elsewhere => {
                let key = dirs[n - 1].0.clone();
                config.chosen.insert(game.id.clone(), key.clone());
                return Ok(Answer::Picked(key));
            }
            Choice::Number(_) => {
                // The typed path. A bad one is explained and asked again.
                write!(output, "Path to the game's {} folder (0 goes back): ", game.game_folder)
                    .and_then(|_| output.flush())
                    .map_err(|e| e.to_string())?;
                loop {
                    let line = read_line(input)?;
                    let line = line.trim();
                    if line == "0" {
                        return Ok(Answer::Back);
                    }
                    match point_at(config, game, line) {
                        Ok(key) => return Ok(Answer::Picked(key)),
                        Err(reason) => {
                            write!(output, "{reason}\nTry another path (0 goes back): ")
                                .and_then(|_| output.flush())
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
            Choice::Back => return Ok(Answer::Back),
            Choice::Other => {
                write!(output, "Pick a number between 1 and {elsewhere}: ")
                    .and_then(|_| output.flush())
                    .map_err(|e| e.to_string())?;
            }
        }
    }
}

/// Question 3, every run: which save slot? One entry per populated slot,
/// with the party's names. Picking by recognising your party beats
/// remembering a letter (ADR 0002).
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
            "Press Enter for {default}, type a letter, or 0 to go back: "
        )?;
        e.flush()
    };
    say(output).map_err(|e| e.to_string())?;

    loop {
        let line = read_line(input)?;
        let line = line.trim();
        if line == "0" {
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

/// One line read as a menu answer: a number in `1..=max`, back, or noise.
/// An empty line takes the default when there is one, else goes back.
enum Choice {
    Number(usize),
    Back,
    Other,
}

fn read_choice<R: BufRead>(
    input: &mut R,
    _output: &mut impl Write,
    max: usize,
    default: Option<usize>,
) -> Result<Choice, String> {
    let line = read_line(input)?;
    let line = line.trim();
    if line == "0" {
        return Ok(Choice::Back);
    }
    if line.is_empty() {
        return Ok(match default {
            Some(d) => Choice::Number(d + 1),
            None => Choice::Back,
        });
    }
    match line.parse::<usize>() {
        Ok(n) if (1..=max).contains(&n) => Ok(Choice::Number(n)),
        _ => Ok(Choice::Other),
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
