//! The game registry: every compiled-in game, found by its id.

use squire_core::games;

#[test]
fn the_registry_lists_pool_of_radiance() {
    let all = games::games();

    let por = all
        .iter()
        .find(|g| g.id == "pool-of-radiance")
        .expect("Pool of Radiance is compiled in");
    assert_eq!(por.name, "Pool of Radiance");
}

#[test]
fn a_game_is_found_by_its_id() {
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.name, "Pool of Radiance");
}

#[test]
fn an_unknown_id_is_none() {
    assert!(games::find("no-such-game").is_none());
}

#[test]
fn the_game_knows_its_game_folder() {
    // Install discovery identifies which game a folder holds by this name.
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.game_folder, "POOLRAD");
}

#[test]
fn the_game_knows_its_dos_config_file_and_data_path_line() {
    // POOL.CFG line 3 holds the DOS data path (C:\POOLRAD\). The manual-path
    // check reads it to explain a folder-name mismatch.
    let por = games::find("pool-of-radiance").unwrap();

    let dos_config = por.dos_config.expect("Pool of Radiance ships POOL.CFG");
    assert_eq!(dos_config.file, "POOL.CFG");
    assert_eq!(dos_config.path_line, 3);
}

#[test]
fn the_registry_lists_unlimited_adventures() {
    let frua = games::find("unlimited-adventures").expect("FRUA is compiled in");

    assert_eq!(frua.name, "Unlimited Adventures");
    assert_eq!(frua.game_folder, "UA");
    assert_eq!(frua.start, "START.BAT");
    // No known game-owned config pins its data path, so no check runs.
    assert!(frua.dos_config.is_none());
}

#[test]
fn unlimited_adventures_saves_per_design_in_party_files() {
    let frua = games::find("unlimited-adventures").unwrap();

    assert_eq!(frua.saves.shape, games::SaveShape::PartyFile);
    assert_eq!(frua.saves.extension, "CSV");
    assert!(frua.saves.designs);
    assert_eq!(frua.saves.party_size_offset, Some(1037));
    assert_eq!(frua.saves.first_record_offset, Some(1039));
}

#[test]
fn pool_of_radiance_saves_one_chrdat_file_per_character() {
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.saves.shape, games::SaveShape::Chrdat);
    assert_eq!(por.saves.extension, "SAV");
    assert!(!por.saves.designs);
}

#[test]
fn the_game_carries_its_record_table() {
    let por = games::find("pool-of-radiance").unwrap();

    assert!(por.table.field("name").is_some());
    assert_eq!(por.table.record_len, 285);
}

#[test]
fn a_table_missing_the_registry_data_fails_at_load() {
    // A record table alone is not a game entry. The id, the game folder and
    // the DOS config location are what the wizard and discovery run on.
    let text = r#"
        game = "Half a Game"
        record_len = 16
        [[field]]
        name = "name"
        offset = 0
        len = 16
        kind = "pascal_string"
    "#;

    let err = games::Game::from_toml(text).unwrap_err().to_string();

    assert!(err.contains("id"), "the error names the missing key: {err}");
}

#[test]
fn an_empty_game_folder_fails_at_load() {
    let text = r#"
        id = "broken"
        game = "Broken"
        game_folder = ""
        start = "GO.EXE"
        machine = "ega"
        record_len = 16
        [saves]
        shape = "chrdat"
        extension = "SAV"
        [dos_config]
        file = "X.CFG"
        path_line = 1
        [[field]]
        name = "name"
        offset = 0
        len = 16
        kind = "pascal_string"
    "#;

    let err = games::Game::from_toml(text).unwrap_err().to_string();

    assert!(err.contains("game_folder"), "got: {err}");
}

#[test]
fn a_zero_config_line_fails_at_load() {
    // The line is one-based, as a person counts lines in a file.
    let text = r#"
        id = "broken"
        game = "Broken"
        game_folder = "BROKEN"
        start = "GO.EXE"
        machine = "ega"
        record_len = 16
        [saves]
        shape = "chrdat"
        extension = "SAV"
        [dos_config]
        file = "X.CFG"
        path_line = 0
        [[field]]
        name = "name"
        offset = 0
        len = 16
        kind = "pascal_string"
    "#;

    let err = games::Game::from_toml(text).unwrap_err().to_string();

    assert!(err.contains("path_line"), "got: {err}");
}

#[test]
fn a_malformed_record_table_still_fails_at_load() {
    // The registry wraps the table loader; its validation must keep firing.
    let text = r#"
        id = "broken"
        game = "Broken"
        game_folder = "BROKEN"
        start = "GO.EXE"
        machine = "ega"
        record_len = 4
        [saves]
        shape = "chrdat"
        extension = "SAV"
        [dos_config]
        file = "X.CFG"
        path_line = 1
        [[field]]
        name = "name"
        offset = 0
        len = 16
        kind = "pascal_string"
    "#;

    let err = games::Game::from_toml(text).unwrap_err().to_string();

    assert!(
        err.contains("runs past"),
        "the table validation fired: {err}"
    );
}

#[test]
fn the_registry_names_the_dos_start_command() {
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.start, "START.EXE");
}

#[test]
fn a_game_without_a_start_command_fails_to_parse() {
    let text = include_str!("../tables/pool-of-radiance.toml").replace(
        "start = \"START.EXE\"",
        "",
    );

    let err = games::Game::from_toml(&text).unwrap_err();

    assert!(err.to_string().contains("start"), "got: {err}");
}
