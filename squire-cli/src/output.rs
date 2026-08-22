//! Turns a party into text.
//!
//! Two formats. The table is for a person reading a terminal. The JSON is the
//! seam a later interface depends on, so it changes far more slowly.

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// The party as a table a person can read at a glance during play.
pub fn table(party: &Party) -> String {
    if party.characters.is_empty() {
        return match party.state {
            PartyState::NotFound => {
                "No party in memory. Load a save and begin the game.".to_string()
            }
            _ => "No characters to show.".to_string(),
        };
    }

    let header = ["NAME", "CLASS", "LVL", "HP", "AC", "STATUS"];
    let rows: Vec<[String; 6]> = party.characters.iter().map(row).collect();

    // Every column is as wide as its widest cell, so the table lines up.
    let mut width = header.map(str::len);
    for r in &rows {
        for (i, cell) in r.iter().enumerate() {
            width[i] = width[i].max(cell.chars().count());
        }
    }

    let line = |cells: &[String; 6]| -> String {
        let mut s = String::from("|");
        for (i, cell) in cells.iter().enumerate() {
            let pad = width[i] - cell.chars().count();
            s.push(' ');
            s.push_str(cell);
            s.push_str(&" ".repeat(pad));
            s.push_str(" |");
        }
        s
    };

    let mut out = String::new();
    out.push_str(&line(&header.map(String::from)));
    out.push('\n');
    let rule: [String; 6] = std::array::from_fn(|i| "-".repeat(width[i]));
    out.push_str(&line(&rule));
    out.push('\n');
    for r in &rows {
        out.push_str(&line(r));
        out.push('\n');
    }

    if party.state == PartyState::Partial {
        out.push_str("\nPartial party. Some characters are not in memory yet.\n");
    }
    out
}

fn row(c: &Character) -> [String; 6] {
    [
        c.name.clone(),
        c.class
            .clone()
            .unwrap_or_else(|| format!("? {:#04x}", c.class_raw)),
        c.level.to_string(),
        format!("{}/{}", c.hit_points_current, c.hit_points_maximum),
        c.armor_class.to_string(),
        c.status
            .clone()
            .unwrap_or_else(|| format!("? {:#04x}", c.status_raw)),
    ]
}

/// The party as JSON.
///
/// Written by hand rather than derived, so that the shape of the output is
/// decided here and does not follow whatever the internal types happen to be.
pub fn json(party: &Party) -> String {
    let state = match party.state {
        PartyState::Live => "live",
        PartyState::Partial => "partial",
        PartyState::NotFound => "not_found",
    };
    let characters: Vec<serde_json::Value> = party.characters.iter().map(character_json).collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "state": state,
        "characters": characters,
    }))
    .expect("this structure always serialises")
}

fn character_json(c: &Character) -> serde_json::Value {
    serde_json::json!({
        "name": c.name,
        "race": c.race,
        "race_raw": c.race_raw,
        "class": c.class,
        "class_raw": c.class_raw,
        "gender": c.gender,
        "alignment": c.alignment,
        "status": c.status,
        "status_raw": c.status_raw,
        "level": c.level,
        "hit_points": { "current": c.hit_points_current, "maximum": c.hit_points_maximum },
        "armor_class": c.armor_class,
        "thac0": c.thac0,
        "experience": c.experience,
        "age": c.age,
        "abilities": {
            "strength": c.strength,
            "strength_exceptional": c.strength_exceptional,
            "intelligence": c.intelligence,
            "wisdom": c.wisdom,
            "dexterity": c.dexterity,
            "constitution": c.constitution,
            "charisma": c.charisma,
        },
    })
}
