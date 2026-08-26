mod common;

use squire_core::{games, saves};

// A save file is CHRDAT{slot}{index}.SAV: the save slot is a letter A through
// J, the character index is 1 through 6. The committed fixtures are slot A of
// a real game folder, so their names are CHRDATA1.SAV through CHRDATA6.SAV.
// Unlimited Adventures instead writes the whole party into one
// SAVGAM{slot}.CSV per design; its tests sit at the end.

fn por() -> games::Game {
    games::find("pool-of-radiance").expect("Pool of Radiance is compiled in")
}

fn frua() -> games::Game {
    games::find("unlimited-adventures").expect("Unlimited Adventures is compiled in")
}

#[test]
fn reads_slot_a_from_a_real_game_folder() {
    let party = saves::slot_party_names(&por(), common::fixture_dir(), 'A').unwrap();

    assert_eq!(
        party,
        vec![
            "THRENDER GRONE",
            "BAKSHI",
            "RHIANNON",
            "BROTHER SEAN",
            "DARKSTAR",
            "PHINEAS",
        ]
    );
}

#[test]
fn reads_slot_j_when_a_and_j_are_populated() {
    // GOG ships Pool of Radiance with slots A and J populated. Slot J was the
    // slot the old code could not read: it held the letter at A and walked the
    // character index instead.
    let dir = tempdir();
    write_save(&dir, 'A', 1, "SLOT A HERO");
    write_save(&dir, 'J', 1, "SLOT J HERO");
    write_save(&dir, 'J', 2, "SLOT J FRIEND");

    let party = saves::slot_party_names(&por(), &dir, 'J').unwrap();

    assert_eq!(party, vec!["SLOT J HERO", "SLOT J FRIEND"]);
}

#[test]
fn the_party_is_in_marching_order() {
    let dir = tempdir();
    // Written out of order; the character index, not file order, decides.
    write_save(&dir, 'B', 3, "THIRD");
    write_save(&dir, 'B', 1, "FIRST");
    write_save(&dir, 'B', 2, "SECOND");

    let party = saves::slot_party_names(&por(), &dir, 'B').unwrap();

    assert_eq!(party, vec!["FIRST", "SECOND", "THIRD"]);
}

#[test]
fn a_slot_reads_at_most_six_characters() {
    let dir = tempdir();
    for i in 1..=6 {
        write_save(&dir, 'A', i, &format!("CHAR{i}"));
    }
    // A seventh file matching the pattern is not a party member. Six is the
    // most any Gold Box party holds.
    write_save_named(&dir, "CHRDATA7.SAV", "IMPOSTOR");

    let party = saves::slot_party_names(&por(), &dir, 'A').unwrap();

    assert_eq!(party.len(), 6);
    assert!(!party.contains(&"IMPOSTOR".to_string()));
}

#[test]
fn a_lowercase_slot_letter_is_accepted() {
    let dir = tempdir();
    write_save(&dir, 'J', 1, "HERO");

    let party = saves::slot_party_names(&por(), &dir, 'j').unwrap();

    assert_eq!(party, vec!["HERO"]);
}

#[test]
fn a_letter_outside_a_through_j_is_rejected() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "HERO");

    let err = saves::slot_party_names(&por(), &dir, 'K')
        .unwrap_err()
        .to_string();

    assert!(err.contains('K'), "the error names the bad letter: {err}");
}

#[test]
fn an_empty_slot_names_the_populated_ones() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "HERO");
    write_save(&dir, 'J', 1, "OTHER");

    let err = saves::slot_party_names(&por(), &dir, 'B')
        .unwrap_err()
        .to_string();

    assert!(err.contains('B'), "the error names the empty slot: {err}");
    assert!(
        err.contains('A') && err.contains('J'),
        "the error names the populated slots: {err}"
    );
}

#[test]
fn enumerates_the_populated_slots_with_their_names() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "ALPHA ONE");
    write_save(&dir, 'A', 2, "ALPHA TWO");
    write_save(&dir, 'J', 1, "JULIET ONE");

    let slots = saves::populated_slots(&por(), &dir).unwrap();

    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].letter, 'A');
    assert_eq!(slots[0].names, vec!["ALPHA ONE", "ALPHA TWO"]);
    assert_eq!(slots[1].letter, 'J');
    assert_eq!(slots[1].names, vec!["JULIET ONE"]);
}

#[test]
fn a_slot_with_no_parseable_file_is_not_listed() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "REAL");
    // Slot B's only file is too short to be a record.
    std::fs::write(format!("{dir}/CHRDATB1.SAV"), b"too short").unwrap();

    let slots = saves::populated_slots(&por(), &dir).unwrap();

    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].letter, 'A');
}

#[test]
fn a_missing_savgam_file_does_not_hide_a_slot() {
    // SAVGAM{slot}.DAT is the game's own bookkeeping. A slot without it still
    // holds readable characters, and refusing it would be guessing.
    let dir = tempdir();
    write_save(&dir, 'C', 1, "HERO");

    let slots = saves::populated_slots(&por(), &dir).unwrap();

    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].letter, 'C');
}

#[test]
fn a_folder_with_no_saves_enumerates_to_a_clear_error() {
    let dir = tempdir();

    let err = saves::populated_slots(&por(), &dir)
        .unwrap_err()
        .to_string();

    assert!(
        err.contains("CHRDAT"),
        "the error says what it looked for: {err}"
    );
}

#[test]
fn a_folder_that_does_not_exist_is_a_clear_error() {
    let err = saves::slot_party_names(&por(), "/no/such/folder/anywhere", 'A')
        .unwrap_err()
        .to_string();

    assert!(err.contains("/no/such/folder/anywhere"), "got: {err}");
}

#[test]
fn a_save_file_too_short_to_be_a_record_is_skipped() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "REALCHAR");
    std::fs::write(format!("{dir}/CHRDATA2.SAV"), b"too short").unwrap();

    let party = saves::slot_party_names(&por(), &dir, 'A').unwrap();

    assert_eq!(party, vec!["REALCHAR"]);
}

#[test]
fn finds_saves_whatever_the_case_of_the_file_name() {
    // DOS wrote upper case. A file copied through other systems can be lower.
    let dir = tempdir();
    write_save_named(&dir, "chrdatj1.sav", "LOWERCASE");

    let party = saves::slot_party_names(&por(), &dir, 'J').unwrap();

    assert_eq!(party, vec!["LOWERCASE"]);
}

// --- Unlimited Adventures: the whole party in one SAVGAM file ---------------

#[test]
fn reads_a_party_out_of_one_savgam_file() {
    let dir = tempdir();
    let party = savgam_bytes(&["LIZABELL", "BEORN"]);
    std::fs::write(format!("{dir}/SAVGAMA.CSV"), party).unwrap();

    let names = saves::slot_party_names(&frua(), &dir, 'A').unwrap();

    assert_eq!(names, vec!["LIZABELL", "BEORN"]);
}

#[test]
fn the_savgam_walk_survives_variable_item_tails() {
    // Each record carries a tail of item data whose length the file does not
    // state reliably, so the reader validates instead of striding. The junk
    // between records here is what an item tail looks like to the walk.
    let dir = tempdir();
    let mut bytes = savgam_header(3);
    for (name, junk) in [("FIRST", 100), ("SECOND", 120), ("THIRD", 0)] {
        bytes.extend_from_slice(&frua_record(name));
        bytes.extend(std::iter::repeat_n(0x42u8, junk));
    }
    std::fs::write(format!("{dir}/SAVGAMB.CSV"), bytes).unwrap();

    let names = saves::slot_party_names(&frua(), &dir, 'B').unwrap();

    assert_eq!(names, vec!["FIRST", "SECOND", "THIRD"]);
}

#[test]
fn the_savgam_party_size_caps_the_walk() {
    // The tail of a real file holds stale copies of earlier records. The
    // party size at its known offset is what keeps them out.
    let dir = tempdir();
    let mut bytes = savgam_header(1);
    bytes.extend_from_slice(&frua_record("ONLYONE"));
    bytes.extend_from_slice(&frua_record("LEFTOVER"));
    std::fs::write(format!("{dir}/SAVGAMA.CSV"), bytes).unwrap();

    let names = saves::slot_party_names(&frua(), &dir, 'A').unwrap();

    assert_eq!(names, vec!["ONLYONE"]);
}

#[test]
fn a_zero_filled_savgam_is_not_a_populated_slot() {
    // A fresh design ships a SAVGAMA.CSV full of zeroes. The error must not
    // claim the files are absent: the user can see them.
    let dir = tempdir();
    std::fs::write(format!("{dir}/SAVGAMA.CSV"), vec![0u8; 10285]).unwrap();

    let err = saves::populated_slots(&frua(), &dir)
        .unwrap_err()
        .to_string();

    assert!(err.contains("no readable character"), "got: {err}");
    assert!(!err.starts_with("game folder: no SAVGAM"), "got: {err}");
}

#[test]
fn two_party_members_with_one_name_both_survive_when_the_size_is_known() {
    // A party may legitimately hold two characters with the same name. The
    // party size, not name uniqueness, is what guards against the stale tail.
    let dir = tempdir();
    let mut bytes = savgam_header(2);
    bytes.extend_from_slice(&frua_record("AXEL"));
    bytes.extend_from_slice(&frua_record("AXEL"));
    bytes.extend_from_slice(&frua_record("STALETAIL"));
    std::fs::write(format!("{dir}/SAVGAMA.CSV"), bytes).unwrap();

    let names = saves::slot_party_names(&frua(), &dir, 'A').unwrap();

    assert_eq!(names, vec!["AXEL", "AXEL"]);
}

#[test]
fn slot_party_records_hands_back_the_bytes_the_walk_accepted() {
    // The verification tool decodes exactly what a live session would read,
    // so the record bytes come from the same walk as the names.
    let dir = tempdir();
    let party = savgam_bytes(&["LIZABELL"]);
    std::fs::write(format!("{dir}/SAVGAMA.CSV"), party).unwrap();

    let records = saves::slot_party_records(&frua(), &dir, 'A').unwrap();

    assert_eq!(records.len(), 1);
    let (name, bytes) = &records[0];
    assert_eq!(name, "LIZABELL");
    let decoded = squire_core::record::decode(&frua().table, bytes).unwrap();
    assert_eq!(decoded.name, "LIZABELL");
}

#[test]
fn designs_lists_only_designs_with_a_saved_party_newest_first() {
    let dir = tempdir();
    let old = format!("{dir}/OLDEST.DSN/SAVE");
    let new = format!("{dir}/CURRENT.DSN/SAVE");
    let fresh = format!("{dir}/FRESH.DSN/SAVE");
    for d in [&old, &new, &fresh] {
        std::fs::create_dir_all(d).unwrap();
    }
    let mut party = savgam_header(1);
    party.extend_from_slice(&frua_record("HERO"));
    std::fs::write(format!("{old}/SAVGAMA.CSV"), &party).unwrap();
    std::fs::write(format!("{new}/SAVGAMA.CSV"), &party).unwrap();
    // FRESH ships the zero-filled file a design starts with: not listed.
    std::fs::write(format!("{fresh}/SAVGAMA.CSV"), vec![0u8; 10285]).unwrap();
    // Push OLDEST's save into the past so the order is deterministic.
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    set_mtime(&format!("{old}/SAVGAMA.CSV"), past);

    let designs = saves::designs(&frua(), &dir).unwrap();

    let names: Vec<&str> = designs.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, vec!["CURRENT", "OLDEST"]);
}

#[test]
fn a_game_without_designs_refuses_the_designs_question() {
    let dir = tempdir();

    let err = saves::designs(&por(), &dir).unwrap_err().to_string();

    assert!(err.contains("not per design"), "got: {err}");
}

// --- helpers ---------------------------------------------------------------

/// A SAVGAM file up to the first record: the party size at its offset, the
/// first record starting right after, zeroes elsewhere.
fn savgam_header(party_size: u8) -> Vec<u8> {
    let mut bytes = vec![0u8; 1039];
    bytes[1037] = party_size;
    bytes
}

fn savgam_bytes(names: &[&str]) -> Vec<u8> {
    let mut bytes = savgam_header(names.len() as u8);
    for name in names {
        bytes.extend_from_slice(&frua_record(name));
    }
    bytes
}

/// One record that passes validation against the Unlimited Adventures table:
/// a human fighter with legal scores, alive and level 1.
fn frua_record(name: &str) -> Vec<u8> {
    let mut r = vec![0u8; 398];
    r[0x60..0x60 + name.len()].copy_from_slice(name.as_bytes());
    // The terminator is the zero already there.
    for offset in [0x71, 0x73, 0x75, 0x77, 0x79, 0x7B] {
        r[offset] = 12; // every ability a legal 12
    }
    r[0x58] = 0x05; // race: human
    r[0x59] = 0x02; // class: fighter
    r[0x5C] = 0x00; // gender: male
    r[0x5D] = 0x00; // alignment: lawful good
    r[0x5E] = 0x00; // status: okay
    r[0x9F] = 1; // level_fighter
    r[0x81] = 9; // hit_points_maximum
    r[0x18B] = 9; // hit_points_current
    r
}

fn set_mtime(path: &str, to: std::time::SystemTime) {
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_modified(to).unwrap();
}

fn tempdir() -> String {
    let base = std::env::temp_dir().join(format!(
        "gbs-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base.to_string_lossy().into_owned()
}

fn write_save(dir: &str, slot: char, index: usize, name: &str) {
    write_save_named(dir, &format!("CHRDAT{slot}{index}.SAV"), name);
}

fn write_save_named(dir: &str, file: &str, name: &str) {
    let mut bytes = vec![0u8; 285];
    bytes[0] = name.len() as u8;
    bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
    std::fs::write(format!("{dir}/{file}"), bytes).unwrap();
}
