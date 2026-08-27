//! What crosses the socket between the host and its views.
//!
//! One JSON object per line, in both directions. A line is a message, so a
//! view can be written in anything that can open a socket and split on a
//! newline, and a person can watch a run with `socat`.
//!
//! The party's shape is decided here and nowhere else. It is written by hand
//! rather than derived, so that it does not follow whatever the internal
//! types happen to be, and `--json` prints the same shape: one party format
//! in the program, not two that can drift.

use std::path::PathBuf;

use serde_json::{json, Value};

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// What a view is told the moment it connects.
///
/// A view has to caption itself and to ask the slot question, and neither can
/// wait for the next poll: a game sitting at the title screen polls every two
/// seconds and may not have found a party at all yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub game_id: String,
    pub game_name: String,
    /// The slot being watched, when one has been picked. A fresh install has
    /// none until the user saves in game.
    pub slot: Option<char>,
    /// The install's save folder, which is where a repick starts looking.
    /// `None` on the `--pid` path, which never started a game.
    pub save_dir: Option<PathBuf>,
}

impl Hello {
    /// The hello as it goes down the socket.
    pub fn line(&self) -> String {
        json!({
            "kind": "hello",
            "game_id": self.game_id,
            "game_name": self.game_name,
            "slot": self.slot.map(|c| c.to_string()),
            "save_dir": self.save_dir.as_ref().map(|p| p.display().to_string()),
        })
        .to_string()
    }
    /// The hello back out of a JSON value.
    pub fn from_value(value: &Value) -> Result<Hello, String> {
        Ok(Hello {
            game_id: value["game_id"]
                .as_str()
                .ok_or_else(|| "the hello names no game".to_string())?
                .to_string(),
            game_name: value["game_name"].as_str().unwrap_or_default().to_string(),
            slot: value["slot"].as_str().and_then(|s| s.chars().next()),
            save_dir: value["save_dir"].as_str().map(PathBuf::from),
        })
    }
}

/// The party as a JSON value.
pub fn party_value(party: &Party) -> Value {
    let state = match party.state {
        PartyState::Live => "live",
        PartyState::Partial => "partial",
        PartyState::NotFound => "not_found",
    };
    let characters: Vec<Value> = party.characters.iter().map(character_value).collect();
    json!({ "state": state, "characters": characters })
}

fn character_value(c: &Character) -> Value {
    json!({
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

/// The party back out of a JSON value.
///
/// A view is another program reading a socket, so this says what was wrong
/// rather than panicking. Every number is range-checked on the way in: the
/// sender is trusted to be Squire, and a wrong number would be drawn as if it
/// were a reading of the game.
pub fn party_from_value(value: &Value) -> Result<Party, String> {
    let state = match value["state"].as_str() {
        Some("live") => PartyState::Live,
        Some("partial") => PartyState::Partial,
        Some("not_found") => PartyState::NotFound,
        other => return Err(format!("unknown party state {other:?}")),
    };
    let list = value["characters"]
        .as_array()
        .ok_or_else(|| "the party has no characters list".to_string())?;
    let mut characters = Vec::with_capacity(list.len());
    for one in list {
        characters.push(character_from_value(one)?);
    }
    Ok(Party { state, characters })
}

fn character_from_value(v: &Value) -> Result<Character, String> {
    Ok(Character {
        name: text(v, "name")?,
        race: maybe_text(&v["race"]),
        race_raw: number(v, "race_raw")?,
        class: maybe_text(&v["class"]),
        class_raw: number(v, "class_raw")?,
        gender: maybe_text(&v["gender"]),
        alignment: maybe_text(&v["alignment"]),
        status: maybe_text(&v["status"]),
        status_raw: number(v, "status_raw")?,
        level: number(v, "level")?,
        hit_points_current: signed(&v["hit_points"], "current")?,
        hit_points_maximum: number(&v["hit_points"], "maximum")?,
        armor_class: signed(v, "armor_class")?,
        thac0: signed(v, "thac0")?,
        experience: number(v, "experience")?,
        age: number(v, "age")?,
        strength: number(&v["abilities"], "strength")?,
        strength_exceptional: number(&v["abilities"], "strength_exceptional")?,
        intelligence: number(&v["abilities"], "intelligence")?,
        wisdom: number(&v["abilities"], "wisdom")?,
        dexterity: number(&v["abilities"], "dexterity")?,
        constitution: number(&v["abilities"], "constitution")?,
        charisma: number(&v["abilities"], "charisma")?,
    })
}

fn text(v: &Value, field: &str) -> Result<String, String> {
    v[field]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("`{field}` is missing or is not text"))
}

/// A field the table may not have a name for. Absent and null mean the same
/// thing, which is that the game's byte had no name in the table.
fn maybe_text(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}

fn number<T: TryFrom<u64>>(v: &Value, field: &str) -> Result<T, String> {
    v[field]
        .as_u64()
        .and_then(|n| T::try_from(n).ok())
        .ok_or_else(|| format!("`{field}` is missing or out of range"))
}

fn signed(v: &Value, field: &str) -> Result<i16, String> {
    v[field]
        .as_i64()
        .and_then(|n| i16::try_from(n).ok())
        .ok_or_else(|| format!("`{field}` is missing or out of range"))
}
