mod common;

use squire_core::record::{self, Character};
use squire_core::table::Table;

/// Builds a record byte-by-byte, so the expected values in a test come from the
/// test itself and never from the code under test.
struct RecordBuilder {
    bytes: Vec<u8>,
}

impl RecordBuilder {
    /// A record that passes validation, which each test then varies.
    fn valid() -> Self {
        let mut b = RecordBuilder {
            bytes: vec![0u8; 285],
        };
        b.name("THRENDER GRONE");
        b.at(0x010, 17); // strength
        b.at(0x011, 12); // intelligence
        b.at(0x012, 12); // wisdom
        b.at(0x013, 17); // dexterity
        b.at(0x014, 16); // constitution
        b.at(0x015, 15); // charisma
        b.at(0x02E, 0x01); // race: dwarf
        b.at(0x02F, 0x02); // class: fighter
        b.at(0x030, 52); // age, low byte
        b.at(0x032, 11); // maximum hit points
        b.at(0x098, 1); // fighter level
        b.at(0x09E, 0x00); // gender: male
        b.at(0x0A0, 0x00); // alignment: lawful good
        b.at(0x10C, 0x00); // status: okay
        b.at(0x11B, 11); // current hit points
        b
    }

    fn at(&mut self, offset: usize, value: u8) -> &mut Self {
        self.bytes[offset] = value;
        self
    }

    fn name(&mut self, name: &str) -> &mut Self {
        self.bytes[0] = name.len() as u8;
        self.bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
        self
    }

    fn build(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

fn decode(bytes: &[u8]) -> Character {
    record::decode(&Table::pool_of_radiance(), bytes).unwrap()
}

#[test]
fn reads_the_name_up_to_its_length_byte() {
    let c = decode(&RecordBuilder::valid().build());

    assert_eq!(c.name, "THRENDER GRONE");
}

#[test]
fn reads_current_and_maximum_hit_points() {
    let mut b = RecordBuilder::valid();
    b.at(0x032, 11).at(0x11B, 3);

    let c = decode(&b.build());

    assert_eq!(c.hit_points_current, 3);
    assert_eq!(c.hit_points_maximum, 11);
}

#[test]
fn names_the_race_class_gender_alignment_and_status() {
    let c = decode(&RecordBuilder::valid().build());

    assert_eq!(c.race.as_deref(), Some("dwarf"));
    assert_eq!(c.class.as_deref(), Some("fighter"));
    assert_eq!(c.gender.as_deref(), Some("male"));
    assert_eq!(c.alignment.as_deref(), Some("lawful good"));
    assert_eq!(c.status.as_deref(), Some("okay"));
}

#[test]
fn reports_an_unknown_enumeration_value_as_unknown_rather_than_guessing() {
    let mut b = RecordBuilder::valid();
    b.at(0x02F, 0x7F); // no class has this value

    let c = decode(&b.build());

    assert_eq!(c.class, None);
    assert_eq!(c.class_raw, 0x7F, "the raw byte is still available");
}

#[test]
fn reports_the_highest_class_level() {
    let mut b = RecordBuilder::valid();
    b.at(0x096, 3).at(0x098, 5).at(0x09B, 2); // cleric 3, fighter 5, mage 2

    let c = decode(&b.build());

    assert_eq!(c.level, 5);
}

#[test]
fn reads_a_two_byte_age_least_significant_byte_first() {
    let mut b = RecordBuilder::valid();
    b.at(0x030, 0xB4).at(0x031, 0x00); // 180, as an elf can be

    let c = decode(&b.build());

    assert_eq!(c.age, 180);
}

#[test]
fn a_slice_shorter_than_the_record_is_an_error_not_a_panic() {
    let short = vec![0u8; 100];

    let err = record::decode(&Table::pool_of_radiance(), &short).unwrap_err();

    assert!(err.to_string().contains("285"), "got: {err}");
}

// --- validation invariants -------------------------------------------------
// A name match alone is not proof. These are the checks that promote a
// candidate to a confirmed record.

fn is_valid(bytes: &[u8]) -> bool {
    record::validate(&Table::pool_of_radiance(), bytes).is_ok()
}

#[test]
fn accepts_a_plausible_record() {
    assert!(is_valid(&RecordBuilder::valid().build()));
}

#[test]
fn rejects_a_name_length_that_cannot_be_right() {
    let mut b = RecordBuilder::valid();
    b.at(0x000, 16); // the field holds fifteen characters at most

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_an_empty_name() {
    let mut b = RecordBuilder::valid();
    b.at(0x000, 0);

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_a_name_holding_bytes_that_are_not_printable() {
    let mut b = RecordBuilder::valid();
    b.at(0x003, 0x07); // a bell character, inside the name

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_an_ability_score_outside_the_rules() {
    let mut b = RecordBuilder::valid();
    b.at(0x010, 30); // strength cannot exceed 25 in these games

    assert!(!is_valid(&b.build()));

    let mut b = RecordBuilder::valid();
    b.at(0x014, 0); // constitution cannot be zero on a living character

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_a_class_value_the_game_never_writes() {
    let mut b = RecordBuilder::valid();
    b.at(0x02F, 0x40);

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_current_hit_points_above_the_maximum() {
    let mut b = RecordBuilder::valid();
    b.at(0x032, 11).at(0x11B, 40);

    assert!(!is_valid(&b.build()));
}

#[test]
fn accepts_current_hit_points_below_zero_because_a_dying_character_has_them() {
    // The game stores current hit points as a signed byte. A dying character
    // sits between -1 and -9, which reads as 247 to 255 unsigned.
    let mut b = RecordBuilder::valid();
    b.at(0x032, 11).at(0x11B, 0xFC).at(0x10C, 0x05); // -4, dying

    assert!(is_valid(&b.build()));

    let c = decode(&b.build());
    assert_eq!(c.hit_points_current, -4);
}

#[test]
fn rejects_a_character_with_no_class_level_at_all() {
    let mut b = RecordBuilder::valid();
    b.at(0x098, 0); // the only level this record had

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_a_class_level_beyond_what_the_game_allows() {
    let mut b = RecordBuilder::valid();
    b.at(0x098, 50);

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_a_status_value_the_game_never_writes() {
    let mut b = RecordBuilder::valid();
    b.at(0x10C, 0x20);

    assert!(!is_valid(&b.build()));
}

#[test]
fn rejects_a_run_of_zero_bytes() {
    // An all-zero buffer matches nothing real, and unallocated memory is full
    // of it. This is the cheapest rejection available.
    assert!(!is_valid(&vec![0u8; 285]));
}

// --- the real save files ---------------------------------------------------

/// The committed saves. This is the check that the table matches a real game,
/// not only the test's idea of one.
use common::saves as real_saves;

#[test]
fn decodes_every_real_save_file() {
    let saves = real_saves();
    assert_eq!(saves.len(), 6, "a full party is six characters");

    let names: Vec<String> = saves.iter().map(|s| decode(s).name).collect();

    assert_eq!(
        names,
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
fn every_real_save_file_passes_validation() {
    let saves = real_saves();

    for (i, bytes) in saves.iter().enumerate() {
        let result = record::validate(&Table::pool_of_radiance(), bytes);
        assert!(
            result.is_ok(),
            "CHRDATA{}.SAV rejected: {:?}",
            i + 1,
            result
        );
    }
}

#[test]
fn decodes_a_multi_class_character_from_a_real_save() {
    let saves = real_saves();

    let bakshi = decode(&saves[1]);

    assert_eq!(bakshi.name, "BAKSHI");
    assert_eq!(bakshi.race.as_deref(), Some("half-elf"));
    assert_eq!(bakshi.class.as_deref(), Some("cleric/fighter/mage"));
    assert_eq!(bakshi.level, 1, "all three classes are level 1");
}
