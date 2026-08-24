//! Reads one 285-byte character record, and decides whether it is real.

use crate::table::{FieldKind, Table};
use crate::Error;

/// One character, as the front end sees it.
///
/// The raw bytes stay behind this type. A front end never learns an offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    /// The named race, or `None` when the table does not know the value.
    pub race: Option<String>,
    pub race_raw: u8,
    pub class: Option<String>,
    pub class_raw: u8,
    pub gender: Option<String>,
    pub alignment: Option<String>,
    pub status: Option<String>,
    pub status_raw: u8,
    /// The highest class level. A multi-class character has several.
    pub level: u8,
    /// Current hit points. The game stores this as a signed byte, so a dying
    /// character holds a negative value.
    pub hit_points_current: i16,
    pub hit_points_maximum: u8,
    /// Stored as sixty minus the real value, decoded here. Lower is better,
    /// and a very good armor class is negative.
    pub armor_class: i16,
    /// Stored as sixty minus the real value, decoded here.
    pub thac0: i16,
    pub experience: u32,
    pub age: u16,
    pub strength: u8,
    pub strength_exceptional: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub charisma: u8,
}

/// The names of the class level fields, highest of which is the level. A
/// table holds the ones its game has: the Krynn games and Unlimited
/// Adventures add a knight, and the Buck Rogers games store no per-class
/// levels at all, only `level_highest_1`.
const LEVEL_FIELDS: [&str; 10] = [
    "level_cleric",
    "level_druid",
    "level_fighter",
    "level_paladin",
    "level_ranger",
    "level_mage",
    "level_thief",
    "level_monk",
    "level_knight",
    "level_highest_1",
];

/// The seven fields holding an ability score.
const ABILITY_FIELDS: [&str; 6] = [
    "strength",
    "intelligence",
    "wisdom",
    "dexterity",
    "constitution",
    "charisma",
];

/// The highest level any class reaches in these games, with room to spare.
const MAX_LEVEL: u8 = 40;

/// Reads a character out of a record.
pub fn decode(table: &Table, bytes: &[u8]) -> Result<Character, Error> {
    check_len(table, bytes)?;

    let level = LEVEL_FIELDS
        .iter()
        .filter_map(|f| u8_at(table, bytes, f))
        .max()
        .unwrap_or(0);

    Ok(Character {
        name: read_name(table, bytes)?,
        race: enum_at(table, bytes, "race"),
        race_raw: u8_at(table, bytes, "race").unwrap_or(0),
        class: enum_at(table, bytes, "class"),
        class_raw: u8_at(table, bytes, "class").unwrap_or(0),
        gender: enum_at(table, bytes, "gender"),
        alignment: enum_at(table, bytes, "alignment"),
        status: enum_at(table, bytes, "status"),
        status_raw: u8_at(table, bytes, "status").unwrap_or(0),
        level,
        hit_points_current: signed_u8_at(table, bytes, "hit_points_current").unwrap_or(0),
        hit_points_maximum: u8_at(table, bytes, "hit_points_maximum").unwrap_or(0),
        armor_class: shown_at(table, bytes, "armor_class_current").unwrap_or(0),
        thac0: shown_at(table, bytes, "thac0_current").unwrap_or(0),
        experience: u32_at(table, bytes, "experience").unwrap_or(0),
        age: age_at(table, bytes).unwrap_or(0),
        strength: u8_at(table, bytes, "strength").unwrap_or(0),
        strength_exceptional: u8_at(table, bytes, "strength_exceptional").unwrap_or(0),
        intelligence: u8_at(table, bytes, "intelligence").unwrap_or(0),
        wisdom: u8_at(table, bytes, "wisdom").unwrap_or(0),
        dexterity: u8_at(table, bytes, "dexterity").unwrap_or(0),
        constitution: u8_at(table, bytes, "constitution").unwrap_or(0),
        charisma: u8_at(table, bytes, "charisma").unwrap_or(0),
    })
}

/// Decides whether a run of bytes is a real character record.
///
/// A scanner finds a candidate by matching a name. The same bytes appear in a
/// file buffer, in a second copy of the save, and by chance. These checks are
/// what separate the live record from those.
pub fn validate(table: &Table, bytes: &[u8]) -> Result<(), Error> {
    check_len(table, bytes)?;

    let reject = |why: String| Err(Error::NotARecord(why));

    // The name: at least one character, at most the field's capacity, every
    // character printable ASCII. Both string shapes get the same scrutiny.
    let name_field = field(table, "name")?;
    // Every read here goes through `get`. The bytes come from another
    // process's memory, so this code must never index on trust.
    let name_bytes = name_bytes(name_field, bytes).map_err(Error::NotARecord)?;
    for (i, &b) in name_bytes.iter().enumerate() {
        if !(0x20..=0x7E).contains(&b) {
            return reject(format!("name byte {i} is {b:#04x}, which is not printable"));
        }
    }

    // Ability scores. The rules put every score between 3 and 25.
    for f in ABILITY_FIELDS {
        if let Some(v) = u8_at(table, bytes, f) {
            if !(3..=25).contains(&v) {
                return reject(format!("{f} is {v}"));
            }
        }
    }

    // Enumerations. A value the game never writes means this is not a record.
    for f in ["race", "class", "gender", "alignment", "status"] {
        if let Some(v) = u8_at(table, bytes, f) {
            if enum_at(table, bytes, f).is_none() {
                return reject(format!("{f} is {v:#04x}, which the game never writes"));
            }
        }
    }

    // Levels. Every character holds at least one class level, and none holds an
    // impossible one.
    let levels: Vec<u8> = LEVEL_FIELDS
        .iter()
        .filter_map(|f| u8_at(table, bytes, f))
        .collect();
    if levels.iter().all(|&l| l == 0) {
        return reject("no class has a level".into());
    }
    if let Some(&bad) = levels.iter().find(|&&l| l > MAX_LEVEL) {
        return reject(format!("a class level is {bad}"));
    }

    // Hit points. Current may be negative, because a dying character is. It
    // must never exceed the maximum.
    let max = u8_at(table, bytes, "hit_points_maximum").unwrap_or(0);
    let cur = signed_u8_at(table, bytes, "hit_points_current").unwrap_or(0);
    if max == 0 {
        return reject("maximum hit points is zero".into());
    }
    if cur > max as i16 {
        return reject(format!(
            "current hit points {cur} exceeds the maximum {max}"
        ));
    }

    Ok(())
}

// --- reading one field -----------------------------------------------------

fn check_len(table: &Table, bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() < table.record_len {
        return Err(Error::ShortRecord {
            want: table.record_len,
            got: bytes.len(),
        });
    }
    Ok(())
}

fn field<'t>(table: &'t Table, name: &str) -> Result<&'t crate::table::Field, Error> {
    table
        .field(name)
        .ok_or_else(|| Error::Table(format!("the table has no field `{name}`")))
}

/// The characters of the name field, without prefix or terminator.
///
/// The error is the reason the bytes cannot be a name, worded for
/// [`validate`]'s rejection message.
fn name_bytes<'b>(f: &crate::table::Field, bytes: &'b [u8]) -> Result<&'b [u8], String> {
    match f.kind {
        FieldKind::PascalString => {
            let len = *bytes
                .get(f.offset)
                .ok_or_else(|| "the name field is past the end".to_string())?
                as usize;
            if len == 0 || len > f.len - 1 {
                return Err(format!("name length byte is {len}"));
            }
            bytes
                .get(f.offset + 1..f.offset + 1 + len)
                .ok_or_else(|| "the name runs past the end".to_string())
        }
        FieldKind::TerminatedString => {
            let raw = bytes
                .get(f.offset..f.offset + f.len)
                .ok_or_else(|| "the name field is past the end".to_string())?;
            let len = raw
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| "the name has no terminator".to_string())?;
            if len == 0 {
                return Err("the name is empty".to_string());
            }
            Ok(&raw[..len])
        }
        _ => Err("the name field is not a string".to_string()),
    }
}

/// The bytes that sit at the name field's offset when it holds `name`.
///
/// This is what a scanner searches for: the length prefix or the terminator
/// comes along, so a prefix of a longer name never matches. `None` when the
/// name cannot fit the field, which makes searching for it pointless.
pub fn name_needle(f: &crate::table::Field, name: &str) -> Option<Vec<u8>> {
    if name.is_empty() || name.len() > f.len - 1 {
        return None;
    }
    match f.kind {
        FieldKind::PascalString => {
            let mut needle = Vec::with_capacity(name.len() + 1);
            needle.push(name.len() as u8);
            needle.extend_from_slice(name.as_bytes());
            Some(needle)
        }
        FieldKind::TerminatedString => {
            let mut needle = Vec::with_capacity(name.len() + 1);
            needle.extend_from_slice(name.as_bytes());
            needle.push(0);
            Some(needle)
        }
        _ => None,
    }
}

fn read_name(table: &Table, bytes: &[u8]) -> Result<String, Error> {
    let f = field(table, "name")?;
    let raw = name_bytes(f, bytes).unwrap_or(&[]);
    Ok(String::from_utf8_lossy(raw).into_owned())
}

/// The name stored at the record's name field, when the bytes hold a
/// plausible one: present, non-empty, printable ASCII throughout.
///
/// This is the save-file readers' way in: they take only the name from a
/// record on disk, because every other number goes stale during play.
pub fn name_at(table: &Table, bytes: &[u8]) -> Option<String> {
    let f = table.field("name")?;
    let raw = name_bytes(f, bytes).ok()?;
    raw.iter()
        .all(|&b| (0x20..=0x7E).contains(&b))
        .then(|| String::from_utf8_lossy(raw).into_owned())
}

fn u8_at(table: &Table, bytes: &[u8], name: &str) -> Option<u8> {
    let f = table.field(name)?;
    bytes.get(f.offset).copied()
}

fn signed_u8_at(table: &Table, bytes: &[u8], name: &str) -> Option<i16> {
    u8_at(table, bytes, name).map(|v| v as i8 as i16)
}

/// Reads one byte and applies the field's transform, when it has one.
fn shown_at(table: &Table, bytes: &[u8], name: &str) -> Option<i16> {
    let f = table.field(name)?;
    let raw = *bytes.get(f.offset)? as i16;
    Some(match f.transform {
        Some(crate::table::Transform::SixtyMinus) => 60 - raw,
        None => raw,
    })
}

/// The age, whatever width this game stores it at. The Buck Rogers games
/// keep it in one byte; every other game uses two.
fn age_at(table: &Table, bytes: &[u8]) -> Option<u16> {
    let f = table.field("age")?;
    match f.kind {
        FieldKind::U8 => u8_at(table, bytes, "age").map(u16::from),
        _ => u16_at(table, bytes, "age"),
    }
}

fn u16_at(table: &Table, bytes: &[u8], name: &str) -> Option<u16> {
    let f = table.field(name)?;
    let s = bytes.get(f.offset..f.offset + 2)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(table: &Table, bytes: &[u8], name: &str) -> Option<u32> {
    let f = table.field(name)?;
    let s = bytes.get(f.offset..f.offset + 4)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn enum_at(table: &Table, bytes: &[u8], name: &str) -> Option<String> {
    let f = table.field(name)?;
    if f.kind != FieldKind::Enum {
        return None;
    }
    let value = *bytes.get(f.offset)?;
    table
        .enum_name(f.enum_name.as_deref()?, value)
        .map(str::to_owned)
}
