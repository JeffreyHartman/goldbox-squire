//! The `--pid` path: attach to a running emulator, resolving everything from
//! arguments or remembered config, never by asking.

use std::path::{Path, PathBuf};

use squire_cli::args::Args;
use squire_cli::attach;
use squire_cli::config::{Config, Install, InstallKind};

fn parse(argv: &[&str]) -> Args {
    Args::parse(argv.iter().map(|s| s.to_string())).unwrap()
}

#[test]
fn all_flags_given_resolves_with_no_interaction() {
    // resolve() takes no input stream, so it cannot prompt by construction.
    let dir = saves_dir("flags", &["CHRDATJ1.SAV"]);
    let args = parse(&[
        "--pid", "1234",
        "--game", "pool-of-radiance",
        "--slot", "J",
        "--game-dir", dir.to_str().unwrap(),
    ]);

    let resolved = attach::resolve(&args, &Config::default()).unwrap();

    assert_eq!(resolved.game_id, "pool-of-radiance");
    assert_eq!(resolved.save_dir, dir);
    assert_eq!(resolved.slot, 'J');
    assert_eq!(resolved.names, vec!["JULIET"]);
}

#[test]
fn the_remembered_install_fills_in_game_and_folder() {
    let dir = saves_dir("remembered", &["CHRDATA1.SAV"]);
    let config = config_with_install(&dir);
    let args = parse(&["--pid", "1234", "--slot", "A"]);

    let resolved = attach::resolve(&args, &config).unwrap();

    assert_eq!(resolved.game_id, "pool-of-radiance");
    assert_eq!(resolved.save_dir, dir);
}

#[test]
fn a_missing_game_names_the_flag() {
    let args = parse(&["--pid", "1234", "--slot", "A"]);

    let err = attach::resolve(&args, &Config::default()).unwrap_err();

    assert!(err.contains("--game"), "got: {err}");
}

#[test]
fn a_missing_game_dir_names_the_flag() {
    let args = parse(&["--pid", "1234", "--game", "pool-of-radiance", "--slot", "A"]);

    let err = attach::resolve(&args, &Config::default()).unwrap_err();

    assert!(err.contains("--game-dir"), "got: {err}");
}

#[test]
fn a_missing_slot_with_several_populated_names_the_flag_and_the_slots() {
    let dir = saves_dir("multi", &["CHRDATA1.SAV", "CHRDATJ1.SAV"]);
    let args = parse(&[
        "--pid", "1234",
        "--game", "pool-of-radiance",
        "--game-dir", dir.to_str().unwrap(),
    ]);

    let err = attach::resolve(&args, &Config::default()).unwrap_err();

    assert!(err.contains("--slot"), "got: {err}");
    assert!(err.contains('A') && err.contains('J'), "got: {err}");
}

#[test]
fn a_lone_populated_slot_resolves_without_the_flag() {
    let dir = saves_dir("lone", &["CHRDATJ1.SAV"]);
    let args = parse(&[
        "--pid", "1234",
        "--game", "pool-of-radiance",
        "--game-dir", dir.to_str().unwrap(),
    ]);

    let resolved = attach::resolve(&args, &Config::default()).unwrap();

    assert_eq!(resolved.slot, 'J');
}

#[test]
fn an_empty_named_slot_errors_with_the_populated_list() {
    let dir = saves_dir("empty-slot", &["CHRDATA1.SAV"]);
    let args = parse(&[
        "--pid", "1234",
        "--game", "pool-of-radiance",
        "--slot", "B",
        "--game-dir", dir.to_str().unwrap(),
    ]);

    let err = attach::resolve(&args, &Config::default()).unwrap_err();

    assert!(err.contains('B') && err.contains('A'), "got: {err}");
}

#[test]
fn an_unknown_game_is_an_error() {
    let args = parse(&["--pid", "1234", "--game", "wizardry", "--slot", "A"]);

    let err = attach::resolve(&args, &Config::default()).unwrap_err();

    assert!(err.contains("wizardry"), "got: {err}");
}

// --- helpers -----------------------------------------------------------------

fn config_with_install(dir: &Path) -> Config {
    let mut config = Config::default();
    config.installs.insert(
        "gog:pool-of-radiance".into(),
        Install {
            game: "pool-of-radiance".into(),
            kind: InstallKind::Gog,
            root: dir.to_string_lossy().into_owned(),
            saves: String::new(),
            confs: vec!["a.conf".into()],
            emulator: None,
            introduced: true,
        },
    );
    config.last_install = Some("gog:pool-of-radiance".into());
    config
}

fn saves_dir(tag: &str, files: &[&str]) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-attach-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    for file in files {
        let name = if file.contains('J') { "JULIET" } else { "ALPHA" };
        let mut bytes = vec![0u8; 285];
        bytes[0] = name.len() as u8;
        bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
        std::fs::write(base.join(file), bytes).unwrap();
    }
    base
}
