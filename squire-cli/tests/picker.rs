//! The save picker's lines: what tells two slots holding the same party apart.
//!
//! No terminal and no clock: every test hands the builder its own `now`.

use std::time::{Duration, SystemTime};

use squire_cli::picker;
use squire_core::saves::{PopulatedSlot, SlotCharacter};

fn slot(letter: char, party: &[(&str, u8)], ago: u64) -> PopulatedSlot {
    PopulatedSlot {
        letter,
        party: party
            .iter()
            .map(|(name, level)| SlotCharacter {
                name: (*name).into(),
                level: (*level > 0).then_some(*level),
            })
            .collect(),
        modified: Some(now() - Duration::from_secs(ago)),
    }
}

fn now() -> SystemTime {
    // A fixed instant, so a test never races the clock.
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_787_000_000)
}

fn menu(slots: &[PopulatedSlot]) -> String {
    picker::slot_menu(slots, now()).join("\n")
}

#[test]
fn one_party_in_every_slot_is_named_once() {
    let party = [("LIZABELL", 1u8), ("BEORN", 2)];
    let slots = [slot('A', &party, 3600), slot('B', &party, 60)];

    let menu = menu(&slots);

    assert!(
        menu.contains("Every slot holds the same party"),
        "got: {menu}"
    );
    assert_eq!(
        menu.matches("LIZABELL").count(),
        1,
        "the names are said once, not once per slot: {menu}"
    );
}

#[test]
fn the_levels_ride_with_the_names() {
    let slots = [slot('A', &[("LIZABELL", 1), ("BEORN", 2)], 60)];

    let menu = menu(&slots);

    assert!(menu.contains("LIZABELL 1, BEORN 2"), "got: {menu}");
}

#[test]
fn a_level_the_record_did_not_hold_is_left_off() {
    let slots = [slot('A', &[("NOBODY", 0)], 60)];

    let menu = menu(&slots);

    assert!(menu.contains("NOBODY"), "got: {menu}");
    assert!(
        !menu.contains("NOBODY 0"),
        "a zero level is unknown: {menu}"
    );
}

#[test]
fn parties_that_differ_get_a_names_line_each() {
    let slots = [
        slot('A', &[("LIZABELL", 1)], 3600),
        slot('B', &[("GRIMWALD", 7)], 60),
    ];

    let menu = menu(&slots);

    assert!(!menu.contains("Every slot"), "got: {menu}");
    assert!(menu.contains("LIZABELL"), "got: {menu}");
    assert!(menu.contains("GRIMWALD"), "got: {menu}");
    assert!(menu.contains("level 7"), "the level range is shown: {menu}");
}

#[test]
fn a_party_that_levelled_up_between_saves_is_still_one_party() {
    // Cycling saves of one party is the case this whole feature is for, and
    // the levels drift as it goes. The names are still said once; the levels
    // move down to the slots they belong to.
    let slots = [
        slot('A', &[("BEORN", 2)], 3600),
        slot('B', &[("BEORN", 3)], 60),
    ];

    let menu = menu(&slots);

    assert!(menu.contains("Every slot"), "got: {menu}");
    assert_eq!(menu.matches("BEORN").count(), 1, "got: {menu}");
    assert!(menu.contains("level 2"), "got: {menu}");
    assert!(menu.contains("level 3"), "got: {menu}");
}

#[test]
fn the_slots_stay_in_letter_order() {
    let party = [("LIZABELL", 1u8)];
    // B was saved most recently; the list still runs A, B, C.
    let slots = [
        slot('A', &party, 3600),
        slot('B', &party, 10),
        slot('C', &party, 600),
    ];

    let menu = menu(&slots);

    let letters: Vec<char> = menu
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| {
            let mut chars = line.chars();
            let letter = chars.next()?;
            chars.next().filter(|c| *c == ' ')?;
            letter.is_ascii_uppercase().then_some(letter)
        })
        .collect();
    assert_eq!(letters, vec!['A', 'B', 'C'], "got: {menu}");
}

#[test]
fn the_newest_slot_is_tagged() {
    let party = [("LIZABELL", 1u8)];
    let slots = [slot('A', &party, 3600), slot('B', &party, 10)];

    let menu = menu(&slots);

    let tagged: Vec<&str> = menu.lines().filter(|l| l.contains("(newest)")).collect();
    assert_eq!(tagged.len(), 1, "exactly one tag: {menu}");
    assert!(tagged[0].trim_start().starts_with('B'), "got: {menu}");
}

#[test]
fn a_tie_for_newest_tags_nothing() {
    // Two slots written in the same second cannot be ranked, and guessing
    // which is newer would be a lie the user cannot see through.
    let party = [("LIZABELL", 1u8)];
    let slots = [slot('A', &party, 60), slot('B', &party, 60)];

    let menu = menu(&slots);

    assert!(!menu.contains("(newest)"), "got: {menu}");
}

#[test]
fn a_lone_slot_is_not_tagged() {
    let slots = [slot('A', &[("LIZABELL", 1)], 60)];

    let menu = menu(&slots);

    assert!(
        !menu.contains("(newest)"),
        "nothing to compare it to: {menu}"
    );
}

#[test]
fn no_line_passes_eighty_columns() {
    // The longest real party is Pool of Radiance's, and a level reaching two
    // digits is what pushes the heading over on its own.
    let party = [
        ("THRENDER GRONE", 12u8),
        ("BAKSHI", 12),
        ("RHIANNON", 12),
        ("BROTHER SEAN", 12),
        ("DARKSTAR", 12),
        ("PHINEAS", 12),
    ];
    let same = [slot('A', &party, 3600), slot('J', &party, 60)];
    let differing = [
        slot('A', &party, 3600),
        slot('J', &[("SOMEBODY ELSE", 1)], 60),
    ];

    for slots in [&same[..], &differing[..]] {
        for line in picker::slot_menu(slots, now()) {
            assert!(line.len() <= 80, "{} columns: {line}", line.len());
        }
    }
}

#[test]
fn a_folded_names_line_breaks_between_names() {
    let party = [
        ("THRENDER GRONE", 12u8),
        ("BAKSHI", 12),
        ("RHIANNON", 12),
        ("BROTHER SEAN", 12),
        ("DARKSTAR", 12),
        ("PHINEAS", 12),
    ];
    let slots = [slot('A', &party, 60)];

    let menu = menu(&slots);

    assert!(
        menu.contains("PHINEAS 12"),
        "no name is cut in half: {menu}"
    );
    assert!(
        menu.lines().any(|l| l.ends_with(',')),
        "the fold happens after a separator: {menu}"
    );
}

#[test]
fn a_slot_whose_time_cannot_be_read_says_so() {
    let mut slots = [slot('A', &[("LIZABELL", 1)], 60)];
    slots[0].modified = None;

    let menu = menu(&slots);

    assert!(menu.contains("time unknown"), "got: {menu}");
}

#[test]
fn a_save_from_this_year_shows_the_day_and_the_time() {
    let stamp = picker::stamp(Some(now() - Duration::from_secs(3600)), now());

    assert!(stamp.contains(':'), "the time is there: {stamp}");
    assert!(
        !stamp.contains('-'),
        "this year needs no year in it: {stamp}"
    );
}

#[test]
fn an_older_save_shows_the_year() {
    // Four hundred days back is another calendar year, whatever today is.
    let long_ago = now() - Duration::from_secs(400 * 24 * 3600);

    let stamp = picker::stamp(Some(long_ago), now());

    assert!(stamp.contains('-'), "the year is there: {stamp}");
    assert!(stamp.contains(':'), "the time is there: {stamp}");
}

#[test]
fn an_absent_time_says_so() {
    let stamp = picker::stamp(None, now());

    assert_eq!(stamp, "time unknown");
}
