mod common;

use squire_core::saves;

// A save file is CHRDAT{slot}{index}.SAV: the save slot is a letter A through
// J, the character index is 1 through 6. The committed fixtures are slot A of
// a real game folder, so their names are CHRDATA1.SAV through CHRDATA6.SAV.

#[test]
fn reads_slot_a_from_a_real_game_folder() {
    let party = saves::slot_party_names(common::fixture_dir(), 'A').unwrap();

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

    let party = saves::slot_party_names(&dir, 'J').unwrap();

    assert_eq!(party, vec!["SLOT J HERO", "SLOT J FRIEND"]);
}

#[test]
fn the_party_is_in_marching_order() {
    let dir = tempdir();
    // Written out of order; the character index, not file order, decides.
    write_save(&dir, 'B', 3, "THIRD");
    write_save(&dir, 'B', 1, "FIRST");
    write_save(&dir, 'B', 2, "SECOND");

    let party = saves::slot_party_names(&dir, 'B').unwrap();

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

    let party = saves::slot_party_names(&dir, 'A').unwrap();

    assert_eq!(party.len(), 6);
    assert!(!party.contains(&"IMPOSTOR".to_string()));
}

#[test]
fn a_lowercase_slot_letter_is_accepted() {
    let dir = tempdir();
    write_save(&dir, 'J', 1, "HERO");

    let party = saves::slot_party_names(&dir, 'j').unwrap();

    assert_eq!(party, vec!["HERO"]);
}

#[test]
fn a_letter_outside_a_through_j_is_rejected() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "HERO");

    let err = saves::slot_party_names(&dir, 'K').unwrap_err().to_string();

    assert!(err.contains('K'), "the error names the bad letter: {err}");
}

#[test]
fn an_empty_slot_names_the_populated_ones() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "HERO");
    write_save(&dir, 'J', 1, "OTHER");

    let err = saves::slot_party_names(&dir, 'B').unwrap_err().to_string();

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

    let slots = saves::populated_slots(&dir).unwrap();

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

    let slots = saves::populated_slots(&dir).unwrap();

    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].letter, 'A');
}

#[test]
fn a_missing_savgam_file_does_not_hide_a_slot() {
    // SAVGAM{slot}.DAT is the game's own bookkeeping. A slot without it still
    // holds readable characters, and refusing it would be guessing.
    let dir = tempdir();
    write_save(&dir, 'C', 1, "HERO");

    let slots = saves::populated_slots(&dir).unwrap();

    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].letter, 'C');
}

#[test]
fn a_folder_with_no_saves_enumerates_to_a_clear_error() {
    let dir = tempdir();

    let err = saves::populated_slots(&dir).unwrap_err().to_string();

    assert!(
        err.contains("CHRDAT"),
        "the error says what it looked for: {err}"
    );
}

#[test]
fn a_folder_that_does_not_exist_is_a_clear_error() {
    let err = saves::slot_party_names("/no/such/folder/anywhere", 'A')
        .unwrap_err()
        .to_string();

    assert!(err.contains("/no/such/folder/anywhere"), "got: {err}");
}

#[test]
fn a_save_file_too_short_to_be_a_record_is_skipped() {
    let dir = tempdir();
    write_save(&dir, 'A', 1, "REALCHAR");
    std::fs::write(format!("{dir}/CHRDATA2.SAV"), b"too short").unwrap();

    let party = saves::slot_party_names(&dir, 'A').unwrap();

    assert_eq!(party, vec!["REALCHAR"]);
}

#[test]
fn finds_saves_whatever_the_case_of_the_file_name() {
    // DOS wrote upper case. A file copied through other systems can be lower.
    let dir = tempdir();
    write_save_named(&dir, "chrdatj1.sav", "LOWERCASE");

    let party = saves::slot_party_names(&dir, 'J').unwrap();

    assert_eq!(party, vec!["LOWERCASE"]);
}

// --- helpers ---------------------------------------------------------------

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
