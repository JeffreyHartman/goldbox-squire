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
    pub armor_class: u8,
    pub thac0: u8,
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

/// The names of the seven class level fields, highest of which is the level.
const LEVEL_FIELDS: [&str; 7] = [
    "level_cleric",
    "level_fighter",
    "level_paladin",
    "level_ranger",
    "level_mage",
    "level_thief",
    "level_monk",
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
        armor_class: u8_at(table, bytes, "armor_class_current").unwrap_or(0),
        thac0: u8_at(table, bytes, "thac0_current").unwrap_or(0),
        experience: u32_at(table, bytes, "experience").unwrap_or(0),
        age: u16_at(table, bytes, "age").unwrap_or(0),
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

    // The name. A length byte of 0 or above 15 cannot be right, and the
    // characters are upper-case ASCII, digits, spaces and punctuation.
    let name_field = field(table, "name")?;
    // Every read here goes through `get`. The bytes come from another
    // process's memory, so this code must never index on trust.
    let len = *bytes
        .get(name_field.offset)
        .ok_or_else(|| Error::NotARecord("the name field is past the end".into()))?
        as usize;
    if len == 0 || len > name_field.len - 1 {
        return reject(format!("name length byte is {len}"));
    }
    let name_bytes = bytes
        .get(name_field.offset + 1..name_field.offset + 1 + len)
        .ok_or_else(|| Error::NotARecord("the name runs past the end".into()))?;
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

fn read_name(table: &Table, bytes: &[u8]) -> Result<String, Error> {
    let f = field(table, "name")?;
    let len = bytes
        .get(f.offset)
        .map(|&n| (n as usize).min(f.len - 1))
        .unwrap_or(0);
    let raw = bytes.get(f.offset + 1..f.offset + 1 + len).unwrap_or(&[]);
    Ok(String::from_utf8_lossy(raw).into_owned())
}

fn u8_at(table: &Table, bytes: &[u8], name: &str) -> Option<u8> {
    let f = table.field(name)?;
    bytes.get(f.offset).copied()
}

fn signed_u8_at(table: &Table, bytes: &[u8], name: &str) -> Option<i16> {
    u8_at(table, bytes, name).map(|v| v as i8 as i16)
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
