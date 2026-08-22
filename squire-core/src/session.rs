//! The public interface of the crate.
//!
//! A front end holds a `Session` and calls [`Session::party`] whenever it wants
//! fresh numbers. The session owns the anchor and hides it. Nothing above this
//! module ever learns an address or an offset.

use crate::mem::{self, Reader};
use crate::record::{self, Character};
use crate::scan;
use crate::table::Table;
use crate::Error;

/// How much of the party the session can see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyState {
    /// Every expected character was found and read.
    Live,
    /// Some were found. The game is probably mid-load, or the party changed.
    Partial,
    /// None were found. The game is running but no party is in memory yet.
    NotFound,
}

/// The party at one moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub state: PartyState,
    /// The characters found, in the order their names were given.
    pub characters: Vec<Character>,
}

/// One character's location in the emulator's memory.
#[derive(Debug, Clone)]
struct Anchor {
    name: String,
    addr: usize,
}

/// A live view of one running game.
pub struct Session<R: Reader> {
    reader: R,
    table: Table,
    /// The names to look for, taken from the save files.
    names: Vec<String>,
    anchors: Vec<Anchor>,
}

impl<R: Reader> Session<R> {
    /// Starts a session against a reader, looking for these character names.
    ///
    /// Nothing is read until [`Session::party`] is called.
    pub fn new(reader: R, table: Table, names: Vec<String>) -> Self {
        Session {
            reader,
            table,
            names,
            anchors: Vec::new(),
        }
    }

    pub fn reader(&self) -> &R {
        &self.reader
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    /// Reads the party as it is right now.
    ///
    /// The first call scans memory. Later calls read the addresses already
    /// found, which is why polling several times a second is cheap. Each call
    /// first checks that the name is still where it was. When it is not, the
    /// anchor is stale and the session scans again rather than reporting old
    /// numbers as if they were live.
    pub fn party(&mut self) -> Result<Party, Error> {
        if !self.anchors_still_valid()? {
            self.anchors = self.scan_for_party()?;
        }

        let mut characters = Vec::new();
        for anchor in &self.anchors {
            let mut buf = vec![0u8; self.table.record_len];
            self.reader.read(anchor.addr, &mut buf)?;
            if record::validate(&self.table, &buf).is_ok() {
                characters.push(record::decode(&self.table, &buf)?);
            }
        }

        let state = if characters.is_empty() {
            PartyState::NotFound
        } else if characters.len() == self.names.len() {
            PartyState::Live
        } else {
            PartyState::Partial
        };

        Ok(Party { state, characters })
    }

    /// Checks that each anchor still points at the character it named.
    ///
    /// This reads sixteen bytes per character, so it costs far less than a
    /// scan. A level up, a wound or a spell all leave the name untouched, which
    /// is why the name is the anchor. Loading a save or restarting the emulator
    /// moves the record, and this is what notices.
    fn anchors_still_valid(&self) -> Result<bool, Error> {
        if self.anchors.is_empty() {
            return Ok(false);
        }
        let field = self
            .table
            .field("name")
            .ok_or_else(|| Error::Table("the table has no field `name`".into()))?;

        for anchor in &self.anchors {
            let mut buf = vec![0u8; field.len];
            match self.reader.read(anchor.addr + field.offset, &mut buf) {
                Ok(_) => {}
                // The address went away. That is staleness, not a failure.
                Err(Error::Unmapped { .. }) => return Ok(false),
                Err(e) => return Err(e),
            }
            let len = buf[0] as usize;
            if len == 0 || len > field.len - 1 {
                return Ok(false);
            }
            if &buf[1..1 + len] != anchor.name.as_bytes() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Searches the emulator's memory for every named character.
    fn scan_for_party(&self) -> Result<Vec<Anchor>, Error> {
        let regions = self.reader.regions()?;
        let mut anchors: Vec<Anchor> = Vec::new();

        for region in mem::searchable(&regions) {
            let mut buf = vec![0u8; region.len()];
            // A region can go away between listing and reading, and one that
            // starts on an inaccessible page fails as a whole. Neither is a
            // reason to abandon the scan.
            if self.reader.read(region.start, &mut buf).is_err() {
                continue;
            }
            for hit in scan::find_records(&self.table, &buf, &self.names) {
                // The first copy of a name wins. A second copy is usually the
                // save file still sitting in a buffer.
                if anchors.iter().any(|a| a.name == hit.name) {
                    continue;
                }
                anchors.push(Anchor {
                    name: hit.name,
                    addr: region.start + hit.offset,
                });
            }
        }

        // Report characters in the order the caller named them, which is the
        // party's marching order, not the order they happen to sit in memory.
        anchors.sort_by_key(|a| {
            self.names
                .iter()
                .position(|n| *n == a.name)
                .unwrap_or(usize::MAX)
        });
        Ok(anchors)
    }
}
