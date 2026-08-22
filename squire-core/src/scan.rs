//! Finds character records in a block of memory.
//!
//! This is the idea Gold Box Companion does not use, and the reason Goldbox
//! Squire works with any emulator build. GBC looks for a marker at an offset it
//! assumes DOS memory has, then derives the base address from it. A different
//! emulator lays memory out differently, so the assumption fails and GBC finds
//! nothing.
//!
//! Squire looks for the character's name instead. The name is written by the
//! player, it never moves inside the record, and it never changes during play.
//! Nothing about the emulator's layout needs to be known.

use crate::record;
use crate::table::Table;

/// A record found in a block of memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Where the record starts, counted from the start of the block searched.
    pub offset: usize,
    pub name: String,
}

/// Finds every record whose name is one of `names`.
///
/// A name match alone is never enough. Every candidate is put through
/// [`record::validate`] before it is reported, which is what rejects the copy
/// of the name that sits in a file buffer or in a stale save.
///
/// Hits come back in ascending offset order. A name that appears twice produces
/// two hits, because deciding which copy is live is the caller's job.
pub fn find_records(table: &Table, haystack: &[u8], names: &[String]) -> Vec<Hit> {
    let mut hits = Vec::new();
    if haystack.len() < table.record_len {
        return hits;
    }

    let name_field = match table.field("name") {
        Some(f) => f,
        None => return hits,
    };

    for name in names {
        // The record starts with a length byte, then the name. Searching for
        // both together costs nothing and removes most false matches early.
        let mut needle = Vec::with_capacity(name.len() + 1);
        needle.push(name.len() as u8);
        needle.extend_from_slice(name.as_bytes());

        // The name does not always sit at the very start of the record, so the
        // record start is found by stepping back over the name field.
        let last_start = haystack.len() - table.record_len;
        for start in 0..=last_start {
            let at = start + name_field.offset;
            if at + needle.len() > haystack.len() {
                break;
            }
            if &haystack[at..at + needle.len()] != needle.as_slice() {
                continue;
            }
            let candidate = &haystack[start..start + table.record_len];
            if record::validate(table, candidate).is_ok() {
                hits.push(Hit {
                    offset: start,
                    name: name.clone(),
                });
            }
        }
    }

    hits.sort_by_key(|h| h.offset);
    hits
}
