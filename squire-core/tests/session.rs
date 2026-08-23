//! The public interface of the crate, tested against memory a test controls.

use std::cell::RefCell;

use squire_core::maps::Region;
use squire_core::mem::Reader;
use squire_core::session::{PartyState, Session};
use squire_core::Error;

/// Memory a test writes by hand. Stands in for a running emulator.
struct FakeMemory {
    base: usize,
    bytes: RefCell<Vec<u8>>,
    /// Counts every read, so a test can show that re-checking is cheap.
    reads: RefCell<usize>,
    fail_with: RefCell<Option<i32>>,
}

impl FakeMemory {
    fn new(size: usize) -> Self {
        FakeMemory {
            base: 0x0100_0000,
            bytes: RefCell::new((0..size).map(|i| (i % 251) as u8).collect()),
            reads: RefCell::new(0),
            fail_with: RefCell::new(None),
        }
    }

    fn put(&self, offset: usize, data: &[u8]) {
        self.bytes.borrow_mut()[offset..offset + data.len()].copy_from_slice(data);
    }

    fn poke(&self, offset: usize, value: u8) {
        self.bytes.borrow_mut()[offset] = value;
    }

    fn reads(&self) -> usize {
        *self.reads.borrow()
    }

    fn start_failing(&self) {
        *self.fail_with.borrow_mut() = Some(1);
    }
}

impl Reader for FakeMemory {
    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error> {
        *self.reads.borrow_mut() += 1;
        if self.fail_with.borrow().is_some() {
            return Err(Error::NoSuchProcess { pid: 1 });
        }
        let bytes = self.bytes.borrow();
        let start = addr
            .checked_sub(self.base)
            .ok_or(Error::Unmapped { pid: 1, addr })?;
        let end = start + buf.len();
        if end > bytes.len() {
            return Err(Error::Unmapped { pid: 1, addr });
        }
        buf.copy_from_slice(&bytes[start..end]);
        Ok(buf.len())
    }

    fn regions(&self) -> Result<Vec<Region>, Error> {
        Ok(vec![Region {
            start: self.base,
            end: self.base + self.bytes.borrow().len(),
            readable: true,
            writable: true,
            shared: false,
            path: None,
        }])
    }
}

mod common;

use common::saves;

/// A fake emulator with a party in it, laid out with the real uneven gaps.
fn emulator_with_party() -> (FakeMemory, Vec<Vec<u8>>, Vec<usize>) {
    let saves = saves();
    let mem = FakeMemory::new(256 * 1024);
    let gaps = [528usize, 432, 496, 480, 352];
    let mut at = 40960;
    let mut offsets = Vec::new();
    for (i, s) in saves.iter().enumerate() {
        mem.put(at, s);
        offsets.push(at);
        at += gaps.get(i).copied().unwrap_or(400);
    }
    (mem, saves, offsets)
}

fn names(saves: &[Vec<u8>]) -> Vec<String> {
    let t = common::table();
    saves
        .iter()
        .map(|s| squire_core::record::decode(&t, s).unwrap().name)
        .collect()
}

#[test]
fn reads_the_whole_party_out_of_a_running_emulator() {
    let (mem, saves, _) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));

    let party = session.party().unwrap();

    assert_eq!(party.state, PartyState::Live);
    assert_eq!(party.characters.len(), 6);
    assert_eq!(party.characters[0].name, "THRENDER GRONE");
    assert_eq!(party.characters[3].name, "BROTHER SEAN");
}

#[test]
fn reports_a_change_in_hit_points_on_the_next_read() {
    let (mem, saves, offsets) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));
    let before = session.party().unwrap().characters[5].hit_points_current;

    // Phineas takes a hit. Offset 0x11B is current hit points.
    session.reader().poke(offsets[5] + 0x11B, 3);
    let after = session.party().unwrap().characters[5].hit_points_current;

    assert_eq!(before, 6);
    assert_eq!(after, 3);
}

#[test]
fn shows_a_dying_character_with_negative_hit_points() {
    let (mem, saves, offsets) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));
    session.party().unwrap();

    session.reader().poke(offsets[5] + 0x11B, 0xFD); // -3
    session.reader().poke(offsets[5] + 0x10C, 0x05); // dying

    let party = session.party().unwrap();

    assert_eq!(party.characters[5].hit_points_current, -3);
    assert_eq!(party.characters[5].status.as_deref(), Some("dying"));
}

#[test]
fn the_second_read_is_far_cheaper_than_the_first() {
    // The first read scans memory. Later reads go straight to the addresses
    // already found. This is what makes polling several times a second sane.
    let (mem, saves, _) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));

    session.party().unwrap();
    let after_scan = session.reader().reads();
    session.party().unwrap();
    let after_second = session.reader().reads();

    let second_read_cost = after_second - after_scan;
    assert!(
        second_read_cost <= 12,
        "the second read took {second_read_cost} reads, so the anchor is not being reused"
    );
}

#[test]
fn finds_the_party_again_when_it_moves_in_memory() {
    // A new save loaded, or the emulator restarted. The old addresses are dead.
    let (mem, saves, offsets) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));
    session.party().unwrap();

    // Wipe the old location and write the party somewhere else.
    for off in &offsets {
        for i in 0..285 {
            session.reader().poke(off + i, 0);
        }
    }
    let mut at = 100_000;
    for s in &saves {
        session.reader().put(at, s);
        at += 400;
    }

    let party = session.party().unwrap();

    assert_eq!(party.state, PartyState::Live);
    assert_eq!(party.characters.len(), 6);
    assert_eq!(party.characters[0].name, "THRENDER GRONE");
}

#[test]
fn reports_that_the_party_is_not_in_memory_rather_than_showing_stale_numbers() {
    // The game is running but no save is loaded yet. Showing the last numbers
    // as if they were live is the failure mode this test exists to prevent.
    let mem = FakeMemory::new(64 * 1024);
    let saves = saves();
    let mut session = Session::new(mem, common::table(), names(&saves));

    let party = session.party().unwrap();

    assert_eq!(party.state, PartyState::NotFound);
    assert!(party.characters.is_empty());
}

#[test]
fn a_partial_party_is_reported_as_partial() {
    let saves = saves();
    let mem = FakeMemory::new(64 * 1024);
    mem.put(10_000, &saves[0]);
    mem.put(11_000, &saves[1]);
    let mut session = Session::new(mem, common::table(), names(&saves));

    let party = session.party().unwrap();

    assert_eq!(party.state, PartyState::Partial);
    assert_eq!(party.characters.len(), 2);
}

#[test]
fn an_emulator_that_went_away_is_an_error_not_stale_data() {
    let (mem, saves, _) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));
    session.party().unwrap();

    session.reader().start_failing();
    let result = session.party();

    assert!(result.is_err(), "got {result:?}");
}

#[test]
fn keeps_reading_the_right_character_after_one_of_them_levels_up() {
    // Levelling changes the class level, the maximum hit points and the
    // experience. It does not change the name, which is why the name is the
    // anchor. The record must not be lost.
    let (mem, saves, offsets) = emulator_with_party();
    let mut session = Session::new(mem, common::table(), names(&saves));
    session.party().unwrap();

    session.reader().poke(offsets[0] + 0x098, 2); // fighter level 2
    session.reader().poke(offsets[0] + 0x032, 19); // new maximum hit points
    session.reader().poke(offsets[0] + 0x11B, 19);

    let party = session.party().unwrap();

    assert_eq!(party.state, PartyState::Live);
    assert_eq!(party.characters[0].name, "THRENDER GRONE");
    assert_eq!(party.characters[0].level, 2);
    assert_eq!(party.characters[0].hit_points_maximum, 19);
}

#[test]
fn retargeting_hunts_the_new_names_and_forgets_the_old_anchors() {
    // The user picked the wrong slot and chose again mid-watch. The session
    // must search for the new party, not keep reporting the old one.
    let (mem, saves, _) = emulator_with_party();
    let mut session = Session::new(
        mem,
        common::table(),
        vec!["NOBODY BY THIS NAME".to_string()],
    );
    assert_eq!(session.party().unwrap().state, PartyState::NotFound);

    session.retarget(names(&saves));

    let party = session.party().unwrap();
    assert_eq!(party.state, PartyState::Live);
    assert_eq!(party.characters.len(), 6);
}
