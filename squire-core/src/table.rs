//! The character record layout, described by data rather than by code.
//!
//! Gold Box Companion compiles its offsets into its executable. Goldbox Squire
//! keeps them in a TOML file, one file per game, so that support for another
//! game is a new table rather than new code.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::Error;

/// How the bytes of a field are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    /// One unsigned byte.
    U8,
    /// Two unsigned bytes, least significant first.
    U16le,
    /// Four unsigned bytes, least significant first.
    U32le,
    /// One length byte, then that many characters.
    PascalString,
    /// One byte whose meaning comes from a named enumeration.
    Enum,
}

impl FieldKind {
    /// The width this kind must have, when the kind fixes the width.
    fn required_len(self) -> Option<usize> {
        match self {
            FieldKind::U8 | FieldKind::Enum => Some(1),
            FieldKind::U16le => Some(2),
            FieldKind::U32le => Some(4),
            FieldKind::PascalString => None,
        }
    }
}

/// A stored value that is not the shown value.
///
/// The Gold Box engine stores armor class and THAC0 as sixty minus the real
/// number, so the byte never goes negative even when the armor class does.
/// `CCHFORM.TXT` documents the THAC0 field as literally "60 - Base THAC0".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transform {
    /// The shown value is sixty minus the stored byte.
    SixtyMinus,
}

/// One field of the character record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Field {
    pub name: String,
    pub offset: usize,
    pub len: usize,
    pub kind: FieldKind,
    /// The enumeration this field's value is looked up in, for `FieldKind::Enum`.
    #[serde(rename = "enum")]
    pub enum_name: Option<String>,
    /// How the stored byte becomes the shown value, when they differ.
    #[serde(default)]
    pub transform: Option<Transform>,
}

/// The record layout of one game.
#[derive(Debug, Clone, Deserialize)]
pub struct Table {
    pub game: String,
    pub record_len: usize,
    #[serde(rename = "field", default)]
    pub fields: Vec<Field>,
    #[serde(default)]
    pub enums: BTreeMap<String, BTreeMap<String, String>>,
}

impl Table {
    /// Parses a table and checks that it describes a possible record.
    pub fn from_toml(text: &str) -> Result<Table, Error> {
        let table: Table = toml::from_str(text).map_err(|e| Error::Table(e.to_string()))?;
        table.validate()?;
        Ok(table)
    }

    /// Rejects a table that cannot describe a real record.
    ///
    /// A wrong offset produces wrong numbers silently, which is the worst
    /// failure this tool can have. Every check here turns that into a loud one.
    fn validate(&self) -> Result<(), Error> {
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();

        for f in &self.fields {
            // `checked_add`, because a table is untrusted input. A plain `+`
            // overflows before the comparison, which lets a bad offset through.
            let end = f.offset.checked_add(f.len).ok_or_else(|| {
                Error::Table(format!(
                    "field `{}` at offset {:#X} with length {} overflows an address",
                    f.name, f.offset, f.len
                ))
            })?;
            if end > self.record_len {
                return Err(Error::Table(format!(
                    "field `{}` at offset {:#05X} with length {} runs past the {}-byte record",
                    f.name, f.offset, f.len, self.record_len
                )));
            }
            if f.len == 0 {
                return Err(Error::Table(format!("field `{}` has zero length", f.name)));
            }
            if let Some(want) = f.kind.required_len() {
                if f.len != want {
                    return Err(Error::Table(format!(
                        "field `{}` is {:?}, which is {} byte(s), but the table says {}",
                        f.name, f.kind, want, f.len
                    )));
                }
            }
            if f.kind == FieldKind::Enum {
                let named = f.enum_name.as_deref().ok_or_else(|| {
                    Error::Table(format!(
                        "field `{}` is an enum but names no enumeration",
                        f.name
                    ))
                })?;
                if !self.enums.contains_key(named) {
                    return Err(Error::Table(format!(
                        "field `{}` names enumeration `{named}`, which the table does not define",
                        f.name
                    )));
                }
            }
            if f.transform.is_some() && f.kind != FieldKind::U8 {
                return Err(Error::Table(format!(
                    "field `{}` has a transform, which only a one-byte number supports",
                    f.name
                )));
            }
            if seen.insert(f.name.as_str(), ()).is_some() {
                return Err(Error::Table(format!("field `{}` is defined twice", f.name)));
            }
        }

        let mut sorted: Vec<&Field> = self.fields.iter().collect();
        sorted.sort_by_key(|f| f.offset);
        for pair in sorted.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.offset.saturating_add(a.len) > b.offset {
                return Err(Error::Table(format!(
                    "field `{}` overlaps field `{}`",
                    a.name, b.name
                )));
            }
        }

        Ok(())
    }

    /// Looks a field up by name.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// The name of one value of one enumeration, when the table defines it.
    ///
    /// The game writes values the table does not list, so an unknown value is
    /// normal and is reported as unknown rather than guessed at.
    pub fn enum_name(&self, enumeration: &str, value: u8) -> Option<&str> {
        let set = self.enums.get(enumeration)?;
        // TOML keys are strings, and the table writes them in hexadecimal.
        set.iter()
            .find(|(k, _)| parse_key(k) == Some(value))
            .map(|(_, v)| v.as_str())
    }
}

fn parse_key(key: &str) -> Option<u8> {
    let key = key.trim();
    if let Some(hex) = key.strip_prefix("0x").or_else(|| key.strip_prefix("0X")) {
        u8::from_str_radix(hex, 16).ok()
    } else {
        key.parse().ok()
    }
}
