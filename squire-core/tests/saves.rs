use squire_core::saves;

const REAL: &str = "/home/jeff/goldbox/pool-of-radiance/data/POOLRAD";

#[test]
fn reads_the_party_names_from_a_game_folder() {
    if !std::path::Path::new(REAL).exists() {
        return;
    }

    let party = saves::party_names(REAL).unwrap();

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
fn reads_the_files_in_numeric_order_not_alphabetical_order() {
    let dir = tempdir();
    // Ten files, so that alphabetical order and numeric order disagree.
    for i in 1..=10 {
        write_save(&dir, i, &format!("CHAR{i}"));
    }

    let party = saves::party_names(&dir).unwrap();

    assert_eq!(party[0], "CHAR1");
    assert_eq!(
        party[1], "CHAR2",
        "CHRDATA10.SAV must not sort before CHRDATA2.SAV"
    );
    assert_eq!(party[9], "CHAR10");
}

#[test]
fn a_folder_with_no_saves_is_a_clear_error() {
    let dir = tempdir();

    let err = saves::party_names(&dir).unwrap_err().to_string();

    assert!(
        err.contains("CHRDATA"),
        "the error says what it looked for: {err}"
    );
}

#[test]
fn a_folder_that_does_not_exist_is_a_clear_error() {
    let err = saves::party_names("/no/such/folder/anywhere")
        .unwrap_err()
        .to_string();

    assert!(err.contains("/no/such/folder/anywhere"), "got: {err}");
}

#[test]
fn a_save_file_too_short_to_be_a_record_is_skipped() {
    let dir = tempdir();
    write_save(&dir, 1, "REALCHAR");
    std::fs::write(format!("{dir}/CHRDATA2.SAV"), b"too short").unwrap();

    let party = saves::party_names(&dir).unwrap();

    assert_eq!(party, vec!["REALCHAR"]);
}

#[test]
fn finds_saves_whatever_the_case_of_the_file_name() {
    // DOS wrote upper case. A file copied through other systems can be lower.
    let dir = tempdir();
    write_save_named(&dir, "chrdata1.sav", "LOWERCASE");

    let party = saves::party_names(&dir).unwrap();

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

fn write_save(dir: &str, n: usize, name: &str) {
    write_save_named(dir, &format!("CHRDATA{n}.SAV"), name);
}

fn write_save_named(dir: &str, file: &str, name: &str) {
    let mut bytes = vec![0u8; 285];
    bytes[0] = name.len() as u8;
    bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
    std::fs::write(format!("{dir}/{file}"), bytes).unwrap();
}
