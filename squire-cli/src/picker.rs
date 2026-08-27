//! What the save picker shows about each save slot.
//!
//! The problem this solves is the one the games never did: several slots hold
//! the same party, and a letter says nothing about which one you were playing.
//! So each slot carries when it was last written, and the party carries its
//! levels.
//!
//! The letters stay in their own order, A through J. Sorting by time would
//! move the entry under the reader's finger between one run and the next, and
//! the slot letters are the one thing about a Gold Box save that never moves.
//!
//! This module writes no output. It builds lines, so the tests read them back
//! with no terminal anywhere.

use std::time::SystemTime;

use squire_core::saves::{Design, PopulatedSlot};

/// The narrowest terminal the wizard is written for. A names line longer than
/// this is folded rather than left to the terminal, which would wrap it into
/// the left margin and lose the indent that groups it with its letter.
const COLUMNS: usize = 80;

/// The save slot question, from its heading to its last entry. The caller
/// writes the prompt that follows.
///
/// Two shapes. When every slot holds the same party, the names are said once
/// at the top, because repeating six identical names ten times is what made
/// the list unreadable to begin with. Otherwise each slot gets its own names
/// line, because then the names are the answer.
pub fn slot_menu(slots: &[PopulatedSlot], now: SystemTime) -> Vec<String> {
    let stamps: Vec<String> = slots.iter().map(|s| stamp(s.modified, now)).collect();
    let width = stamps.iter().map(String::len).max().unwrap_or(0);
    let newest = newest_letter(slots);

    let mut lines = Vec::new();
    let shared = shared_names(slots);
    match &shared {
        Some(names) => {
            lines.push("Which save slot? Every slot holds the same party:".to_string());
            // The levels belong up here only when they are the same in every
            // slot. When they drift, saying one slot's levels for all of them
            // would be wrong, so each entry states its own range instead.
            let heading = match shared_party(slots) {
                Some(levelled) => levelled,
                None => names.join(", "),
            };
            lines.extend(fold(&heading, 2));
            lines.push(String::new());
        }
        None => lines.push("Which save slot?".to_string()),
    }

    for (slot, stamp) in slots.iter().zip(&stamps) {
        let levels = match (shared_party(slots).is_some(), level_range(slot)) {
            // Already said in the heading.
            (true, _) => String::new(),
            (false, Some(range)) => format!("  {range}"),
            (false, None) => String::new(),
        };
        let tag = if Some(slot.letter) == newest {
            "   (newest)"
        } else {
            ""
        };
        let entry = format!("  {}  {stamp:width$}{levels}{tag}", slot.letter);
        lines.push(entry.trim_end().to_string());
        if shared.is_none() {
            lines.extend(fold(&slot.names().join(", "), 5));
        }
    }
    lines
}

/// The design question, from its heading to its last entry. The order is the
/// caller's, which is newest first.
pub fn design_lines(designs: &[Design], now: SystemTime) -> Vec<String> {
    let width = designs.iter().map(|d| d.name.len()).max().unwrap_or(0);
    let mut lines = vec!["Which adventure?".to_string()];
    lines.extend(designs.iter().enumerate().map(|(n, design)| {
        format!(
            "  {}. {:width$}   played {}",
            n + 1,
            design.name,
            stamp(design.modified, now)
        )
    }));
    lines
}

/// When a save was written, in local time.
///
/// A save from this year needs no year to place it, and a save from an earlier
/// one is nearly always a copy the publisher shipped, which the full date says
/// plainly.
pub fn stamp(when: Option<SystemTime>, now: SystemTime) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (Some(t), Some(today)) = (when.and_then(local), local(now)) else {
        return "time unknown".to_string();
    };
    let month = MONTHS.get(t.tm_mon as usize).copied().unwrap_or("???");
    if t.tm_year == today.tm_year {
        format!("{month} {:02} {:02}:{:02}", t.tm_mday, t.tm_hour, t.tm_min)
    } else {
        format!(
            "{}-{:02}-{:02} {:02}:{:02}",
            t.tm_year + 1900,
            t.tm_mon + 1,
            t.tm_mday,
            t.tm_hour,
            t.tm_min
        )
    }
}

/// One list of names as one or more indented lines, none of them wider than
/// the terminal the wizard is written for. It breaks between names, because
/// half a name at the end of a line reads as a different character.
fn fold(text: &str, indent: usize) -> Vec<String> {
    let pad = " ".repeat(indent);
    let mut lines = Vec::new();
    let mut line = pad.clone();
    for (n, part) in text.split(", ").enumerate() {
        let separator = if n == 0 { "" } else { ", " };
        if line.len() + separator.len() + part.len() > COLUMNS && line.len() > indent {
            lines.push(format!("{line},"));
            line = format!("{pad}{part}");
        } else {
            line = format!("{line}{separator}{part}");
        }
    }
    lines.push(line);
    lines
}

/// The letter of the one slot written most recently, when there is exactly
/// one. Two slots written in the same second cannot be ranked, and the tag is
/// left off rather than guessed at.
fn newest_letter(slots: &[PopulatedSlot]) -> Option<char> {
    if slots.len() < 2 {
        return None;
    }
    let newest = slots.iter().max_by_key(|s| s.modified)?;
    newest.modified?;
    let ties = slots
        .iter()
        .filter(|s| s.modified == newest.modified)
        .count();
    (ties == 1).then_some(newest.letter)
}

/// The names every slot holds, when they all hold the same ones. This is what
/// decides the two shapes: a party that levelled up between two saves is still
/// the same party, and repeating its names is still noise.
fn shared_names(slots: &[PopulatedSlot]) -> Option<Vec<String>> {
    let first = slots.first()?.names();
    slots.iter().all(|s| s.names() == first).then_some(first)
}

/// The party every slot holds, named with its levels, when the slots agree on
/// the levels too.
fn shared_party(slots: &[PopulatedSlot]) -> Option<String> {
    let first = slots.first()?;
    slots
        .iter()
        .all(|s| s.party == first.party)
        .then(|| join_levelled(first))
}

/// `LIZABELL 1, BEORN 2`, leaving off a level the record did not hold.
fn join_levelled(slot: &PopulatedSlot) -> String {
    slot.party
        .iter()
        .map(|c| match c.level {
            Some(level) => format!("{} {level}", c.name),
            None => c.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `level 7` or `levels 1-2`, and nothing at all when no record held a level.
fn level_range(slot: &PopulatedSlot) -> Option<String> {
    let levels: Vec<u8> = slot.party.iter().filter_map(|c| c.level).collect();
    let low = *levels.iter().min()?;
    let high = *levels.iter().max()?;
    Some(if low == high {
        format!("level {low}")
    } else {
        format!("levels {low}-{high}")
    })
}

/// A moment as the local calendar reads it.
///
/// The C library is what knows the machine's time zone, and pulling in a date
/// crate to ask it would be a heavier answer than the question deserves.
fn local(when: SystemTime) -> Option<libc::tm> {
    let secs = when
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs()
        .try_into()
        .ok()?;
    // SAFETY: an all-zero tm is a valid one. Every field is an integer but
    // tm_zone, which is a pointer C is allowed to leave null.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: both pointers are to live locals, and localtime_r writes only
    // the tm it is given. It is the reentrant call precisely so that one
    // thread cannot clobber another's result.
    let filled = unsafe { libc::localtime_r(&secs, &mut tm) };
    (!filled.is_null()).then_some(tm)
}
