use squire_core::record;
use squire_core::scan;
use squire_core::table::Table;

fn table() -> Table {
    Table::pool_of_radiance()
}

/// Reads the six real saves. These are the records the game itself wrote, so
/// they are the right needles to hide in a haystack.
fn real_saves() -> Vec<Vec<u8>> {
    let dir = std::path::Path::new("/home/jeff/goldbox/pool-of-radiance/data/POOLRAD");
    (1..=6)
        .filter_map(|i| std::fs::read(dir.join(format!("CHRDATA{i}.SAV"))).ok())
        .collect()
}

/// Buries records in a buffer of filler at the given offsets.
fn haystack(records: &[(usize, &[u8])], size: usize) -> Vec<u8> {
    // Filler is a repeating pattern rather than zeroes, so that a match is not
    // an accident of an empty buffer.
    let mut buf: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    for (at, rec) in records {
        buf[*at..*at + rec.len()].copy_from_slice(rec);
    }
    buf
}

#[test]
fn finds_a_record_by_its_name() {
    let saves = real_saves();
    if saves.is_empty() {
        return;
    }
    let buf = haystack(&[(4096, &saves[0])], 65536);

    let hits = scan::find_records(&table(), &buf, &["THRENDER GRONE".to_string()]);

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].offset, 4096);
    assert_eq!(hits[0].name, "THRENDER GRONE");
}

#[test]
fn finds_every_member_of_a_party() {
    let saves = real_saves();
    if saves.is_empty() {
        return;
    }
    let names: Vec<String> = saves
        .iter()
        .map(|s| record::decode(&table(), s).unwrap().name)
        .collect();
    // The gaps between records are uneven, because each character's inventory
    // follows their record. These are the gaps measured on the real game.
    let gaps = [528usize, 432, 496, 480, 352];
    let mut at = 8192;
    let mut placed = Vec::new();
    for (i, s) in saves.iter().enumerate() {
        placed.push((at, s.as_slice()));
        if i < gaps.len() {
            at += gaps[i];
        }
    }
    let buf = haystack(&placed, 65536);

    let hits = scan::find_records(&table(), &buf, &names);

    assert_eq!(hits.len(), 6, "every party member is found");
    let found: Vec<&str> = hits.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(found, names.iter().map(String::as_str).collect::<Vec<_>>());
}

#[test]
fn returns_hits_in_ascending_address_order() {
    let saves = real_saves();
    if saves.is_empty() {
        return;
    }
    let names: Vec<String> = saves
        .iter()
        .map(|s| record::decode(&table(), s).unwrap().name)
        .collect();
    // Place the party backwards in memory.
    let buf = haystack(&[(20000, &saves[1]), (10000, &saves[0])], 65536);

    let hits = scan::find_records(&table(), &buf, &names);

    assert_eq!(hits.len(), 2);
    assert!(hits[0].offset < hits[1].offset);
}

#[test]
fn ignores_a_name_that_is_not_followed_by_a_valid_record() {
    // The name appears in a text buffer, with nothing behind it. This is the
    // false positive validation exists to kill.
    let mut buf: Vec<u8> = (0..65536).map(|i| (i % 251) as u8).collect();
    let text = b"\x0eTHRENDER GRONE";
    buf[3000..3000 + text.len()].copy_from_slice(text);

    let hits = scan::find_records(&table(), &buf, &["THRENDER GRONE".to_string()]);

    assert!(hits.is_empty(), "a bare name is not a record");
}

#[test]
fn finds_nothing_in_an_empty_buffer() {
    let hits = scan::find_records(&table(), &[], &["THRENDER GRONE".to_string()]);

    assert!(hits.is_empty());
}

#[test]
fn finds_nothing_when_the_record_runs_off_the_end_of_the_buffer() {
    let saves = real_saves();
    if saves.is_empty() {
        return;
    }
    // The name is present, but there is not room for a whole record behind it.
    let mut buf = vec![0u8; 300];
    buf[100..100 + 20].copy_from_slice(&saves[0][..20]);

    let hits = scan::find_records(&table(), &buf, &["THRENDER GRONE".to_string()]);

    assert!(hits.is_empty());
}

#[test]
fn a_duplicate_copy_of_a_record_is_reported_as_two_hits() {
    // The caller decides which copy is live. The scanner reports what it sees.
    let saves = real_saves();
    if saves.is_empty() {
        return;
    }
    let buf = haystack(&[(1000, &saves[0]), (40000, &saves[0])], 65536);

    let hits = scan::find_records(&table(), &buf, &["THRENDER GRONE".to_string()]);

    assert_eq!(hits.len(), 2);
}
