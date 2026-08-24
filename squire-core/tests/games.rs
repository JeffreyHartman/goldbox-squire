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
fn the_game_knows_its_save_folder() {
    // Install discovery identifies which game a folder holds by this name.
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.save_folder, "POOLRAD");
}

#[test]
fn the_game_knows_its_dos_config_file_and_data_path_line() {
    // POOL.CFG line 3 holds the DOS data path (C:\POOLRAD\). The manual-path
    // check reads it to explain a folder-name mismatch.
    let por = games::find("pool-of-radiance").unwrap();

    assert_eq!(por.dos_config.file, "POOL.CFG");
    assert_eq!(por.dos_config.path_line, 3);
}

#[test]
fn the_game_carries_its_record_table() {
    let por = games::find("pool-of-radiance").unwrap();

    assert!(por.table.field("name").is_some());
    assert_eq!(por.table.record_len, 285);
}

#[test]
fn a_table_missing_the_registry_data_fails_at_load() {
    // A record table alone is not a game entry. The id, the save folder and
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
fn an_empty_save_folder_fails_at_load() {
    let text = r#"
        id = "broken"
        game = "Broken"
        save_folder = ""
        start = "GO.EXE"
        record_len = 16
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

    assert!(err.contains("save_folder"), "got: {err}");
}

#[test]
fn a_zero_config_line_fails_at_load() {
    // The line is one-based, as a person counts lines in a file.
    let text = r#"
        id = "broken"
        game = "Broken"
        save_folder = "BROKEN"
        start = "GO.EXE"
        record_len = 16
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
        save_folder = "BROKEN"
        start = "GO.EXE"
        record_len = 4
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
