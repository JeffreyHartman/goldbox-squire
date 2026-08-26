//! The watch loop, driven against a fake emulator and a fake keyboard.
//!
//! The loop used to live in the binary, where none of this could be reached.
//! Every seam it needs is a parameter, so a test can end the loop when it
//! likes and read back exactly what was drawn.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Duration;

use squire_cli::watch::{self, Alive, Interrupt, Keys, Screen, Watch};
use squire_core::maps::Region;
use squire_core::mem::Reader;
use squire_core::session::{Party, Session};
use squire_core::Error;

/// Memory a test writes by hand. Stands in for a running emulator.
struct FakeMemory {
    base: usize,
    bytes: Vec<u8>,
}

impl FakeMemory {
    fn new(size: usize) -> Self {
        FakeMemory {
            base: 0x0100_0000,
            bytes: (0..size).map(|i| (i % 251) as u8).collect(),
        }
    }
}

impl Reader for FakeMemory {
    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let start = addr
            .checked_sub(self.base)
            .ok_or(Error::Unmapped { pid: 1, addr })?;
        let end = start + buf.len();
        if end > self.bytes.len() {
            return Err(Error::Unmapped { pid: 1, addr });
        }
        buf.copy_from_slice(&self.bytes[start..end]);
        Ok(buf.len())
    }

    fn regions(&self) -> Result<Vec<Region>, Error> {
        Ok(vec![Region {
            start: self.base,
            end: self.base + self.bytes.len(),
            readable: true,
            writable: true,
            shared: false,
            path: None,
        }])
    }
}

/// A reader the kernel refuses. The fatal case: the watch must not swallow it.
struct Forbidden;

impl Reader for Forbidden {
    fn read(&self, _addr: usize, _buf: &mut [u8]) -> Result<usize, Error> {
        Err(Error::PermissionDenied { pid: 1 })
    }

    fn regions(&self) -> Result<Vec<Region>, Error> {
        Err(Error::PermissionDenied { pid: 1 })
    }
}

/// A reader whose process has already gone away.
struct GoneProcess;

impl Reader for GoneProcess {
    fn read(&self, _addr: usize, _buf: &mut [u8]) -> Result<usize, Error> {
        Err(Error::NoSuchProcess { pid: 1 })
    }

    fn regions(&self) -> Result<Vec<Region>, Error> {
        Err(Error::NoSuchProcess { pid: 1 })
    }
}

/// The six real character records, borrowed from the core crate's fixtures.
fn saves() -> Vec<Vec<u8>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("squire-core")
        .join("tests")
        .join("fixtures");
    (1..=6)
        .map(|i| {
            let path = dir.join(format!("CHRDATA{i}.SAV"));
            std::fs::read(&path)
                .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()))
        })
        .collect()
}

fn table() -> squire_core::table::Table {
    squire_core::games::find("pool-of-radiance")
        .expect("Pool of Radiance is compiled in")
        .table
}

/// A fake emulator with the real party laid out in it.
fn emulator_with_party() -> (FakeMemory, Vec<String>) {
    let saves = saves();
    let mut mem = FakeMemory::new(256 * 1024);
    let mut at = 40960;
    for s in &saves {
        mem.bytes[at..at + s.len()].copy_from_slice(s);
        at += 528;
    }
    let t = table();
    let names = saves
        .iter()
        .map(|s| squire_core::record::decode(&t, s).unwrap().name)
        .collect();
    (mem, names)
}

/// Everything the loop drew, in order.
#[derive(Default)]
struct Recorder {
    parties: Vec<Party>,
    notices: Vec<String>,
}

impl Screen for Recorder {
    fn party(&mut self, party: &Party) {
        self.parties.push(party.clone());
    }

    fn notice(&mut self, message: &str) {
        self.notices.push(message.to_string());
    }
}

impl Recorder {
    fn said(&self, fragment: &str) -> bool {
        self.notices.iter().any(|n| n.contains(fragment))
    }

    fn drawn(&self) -> usize {
        self.parties.len()
    }
}

/// A keyboard nobody touches.
struct Silence;

impl Keys for Silence {
    fn wait(&mut self, _pause: Duration) -> Result<Interrupt, String> {
        Ok(Interrupt::None)
    }
}

/// A keyboard with its answers written down in advance.
struct Scripted(VecDeque<Interrupt>);

impl Keys for Scripted {
    fn wait(&mut self, _pause: Duration) -> Result<Interrupt, String> {
        Ok(self.0.pop_front().unwrap_or(Interrupt::None))
    }
}

/// An emulator that runs for a fixed number of polls and then stops.
struct Countdown(usize);

impl Alive for Countdown {
    fn is_running(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }
        self.0 -= 1;
        true
    }
}

/// Timings a test never waits out. Real values come from the command line.
fn instant() -> Watch {
    Watch {
        interval: Duration::ZERO,
        waiting_poll: Duration::ZERO,
        hint_after: Duration::from_secs(10),
    }
}

#[test]
fn the_party_is_drawn_once_per_poll() {
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), names.clone());
    let screen = &mut Recorder::default();
    let mut running = Countdown(3);

    watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        Some(&mut running),
        Some('A'),
        names,
    )
    .unwrap();

    assert_eq!(screen.drawn(), 3);
    assert_eq!(screen.parties[0].characters.len(), 6);
}

#[test]
fn the_first_notice_names_the_save_slot_being_waited_for() {
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), names.clone());
    let screen = &mut Recorder::default();

    watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        Some(&mut Countdown(1)),
        Some('C'),
        names,
    )
    .unwrap();

    assert!(
        screen.notices[0].contains("save slot C"),
        "{:?}",
        screen.notices
    );
}

#[test]
fn a_run_with_no_saved_game_yet_says_so_instead_of_naming_a_slot() {
    let mut session = Session::new(FakeMemory::new(1024), table(), Vec::new());
    let screen = &mut Recorder::default();

    watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        Some(&mut Countdown(2)),
        None,
        Vec::new(),
    )
    .unwrap();

    assert!(screen.said("no saved game yet"), "{:?}", screen.notices);
    assert_eq!(screen.drawn(), 0, "nothing to draw before a party is found");
}

#[test]
fn the_hint_names_the_assumption_after_the_hunt_drags_on() {
    // Nothing is in this memory, so the party is never found.
    let mut session = Session::new(FakeMemory::new(64 * 1024), table(), vec!["NOBODY".into()]);
    let screen = &mut Recorder::default();
    let timing = Watch {
        hint_after: Duration::ZERO,
        ..instant()
    };

    watch::watch(
        &mut session,
        &timing,
        screen,
        &mut Silence,
        Some(&mut Countdown(4)),
        Some('B'),
        vec!["NOBODY".into()],
    )
    .unwrap();

    let hints: Vec<&String> = screen
        .notices
        .iter()
        .filter(|n| n.contains("still looking"))
        .collect();
    assert_eq!(
        hints.len(),
        1,
        "the hint is said once: {:?}",
        screen.notices
    );
    assert!(hints[0].contains("NOBODY"));
}

#[test]
fn a_found_party_never_gets_the_hint() {
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), names.clone());
    let screen = &mut Recorder::default();
    let timing = Watch {
        hint_after: Duration::ZERO,
        ..instant()
    };

    watch::watch(
        &mut session,
        &timing,
        screen,
        &mut Silence,
        Some(&mut Countdown(3)),
        Some('A'),
        names,
    )
    .unwrap();

    assert!(!screen.said("still looking"), "{:?}", screen.notices);
}

#[test]
fn the_emulator_ending_ends_the_watch_without_an_error() {
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), names.clone());
    let screen = &mut Recorder::default();

    let result = watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        Some(&mut Countdown(1)),
        Some('A'),
        names,
    );

    assert!(result.is_ok());
    assert!(screen.said("the emulator ended"), "{:?}", screen.notices);
}

#[test]
fn the_process_vanishing_mid_read_ends_the_watch_without_an_error() {
    let mut session = Session::new(GoneProcess, table(), vec!["THRENDER GRONE".into()]);
    let screen = &mut Recorder::default();

    let result = watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        None,
        Some('A'),
        vec!["THRENDER GRONE".into()],
    );

    assert!(result.is_ok());
    assert!(screen.said("the emulator ended"), "{:?}", screen.notices);
}

#[test]
fn a_read_failure_that_is_not_a_missing_process_is_fatal() {
    let mut session = Session::new(Forbidden, table(), vec!["THRENDER GRONE".into()]);
    let screen = &mut Recorder::default();

    let result = watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Silence,
        Some(&mut Countdown(5)),
        Some('A'),
        vec!["THRENDER GRONE".into()],
    );

    assert!(result.is_err(), "a permission error must stay loud");
}

#[test]
fn repicking_points_the_watch_at_the_new_party() {
    // The session starts looking for a name that is not there. The user
    // presses Enter, picks the slot the real party is in, and it appears.
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), vec!["NOBODY".into()]);
    let screen = &mut Recorder::default();
    let mut keys = Scripted(VecDeque::from(vec![Interrupt::Repick {
        slot: 'J',
        names: names.clone(),
    }]));

    watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut keys,
        Some(&mut Countdown(3)),
        Some('A'),
        vec!["NOBODY".into()],
    )
    .unwrap();

    assert!(screen.said("save slot J"), "{:?}", screen.notices);
    assert!(screen.drawn() > 0, "the new party is drawn");
}

/// A keyboard that quits after the first pause.
struct Quits;

impl Keys for Quits {
    fn wait(&mut self, _pause: Duration) -> Result<Interrupt, String> {
        Ok(Interrupt::Quit)
    }
}

#[test]
fn the_user_quitting_ends_the_watch_without_an_error() {
    // The HUD's q key. Ending the run is the loop's job, because the loop is
    // what holds the emulator handle.
    let (mem, names) = emulator_with_party();
    let mut session = Session::new(mem, table(), names.clone());
    let screen = &mut Recorder::default();
    let mut running = Countdown(100);

    let result = watch::watch(
        &mut session,
        &instant(),
        screen,
        &mut Quits,
        Some(&mut running),
        Some('A'),
        names,
    );

    assert!(result.is_ok());
    assert_eq!(screen.drawn(), 1, "one draw, then the user stopped it");
}
