//! The manual install path's guards: the folder-name check and the
//! first-run note.

use std::path::{Path, PathBuf};

use squire_cli::args::Args;
use squire_cli::config::{Config, Install, InstallKind};
use squire_cli::manual;
use squire_core::games;

// --- the pair feeds the wizard ----------------------------------------------

#[test]
fn conf_and_game_dir_become_a_remembered_manual_install() {
    let args = Args::parse(
        ["--conf", "/hand/por.conf", "--game-dir", "/hand/POOLRAD"]
            .iter()
            .map(|s| s.to_string()),
    )
    .unwrap();
    let mut config = Config::default();

    let changed = config.remember_manual(&args);

    assert!(changed);
    let install = &config.installs["manual:pool-of-radiance"];
    assert_eq!(install.kind, InstallKind::Manual);
    assert_eq!(install.root, "/hand/POOLRAD");
    assert_eq!(install.confs, vec!["/hand/por.conf"]);
    assert_eq!(
        config.last_install.as_deref(),
        Some("manual:pool-of-radiance"),
        "the pair is the default next run, like any other install"
    );
}

#[test]
fn the_migrated_flat_config_matches_a_fresh_pair() {
    // An existing user's v1 file and a fresh --conf/--game-dir must land in
    // the same place, so both paths behave identically from here on.
    let old = Config::from_toml(
        r#"
        game_dir = "/hand/POOLRAD"
        conf = "/hand/por.conf"
        "#,
    )
    .unwrap();

    let args = Args::parse(
        ["--conf", "/hand/por.conf", "--game-dir", "/hand/POOLRAD"]
            .iter()
            .map(|s| s.to_string()),
    )
    .unwrap();
    let mut fresh = Config::default();
    fresh.remember_manual(&args);

    assert_eq!(old, fresh);
}

// --- the folder-name check ----------------------------------------------------

fn por() -> games::Game {
    games::find("pool-of-radiance").unwrap()
}

fn manual_install(root: &Path) -> Install {
    Install {
        game: "pool-of-radiance".into(),
        kind: InstallKind::Manual,
        root: root.to_string_lossy().into_owned(),
        saves: String::new(),
        confs: vec!["/hand/por.conf".into()],
        emulator: None,
        introduced: false,
    }
}

#[test]
fn a_folder_matching_the_dos_config_passes() {
    let dir = tempdir("match").join("POOLRAD");
    std::fs::create_dir_all(&dir).unwrap();
    // POOL.CFG line 3 holds the DOS data path.
    std::fs::write(dir.join("POOL.CFG"), "A\nB\nC:\\POOLRAD\\\n").unwrap();

    assert!(manual::folder_name_check(&manual_install(&dir), &por()).is_ok());
}

#[test]
fn a_mismatched_folder_is_refused_naming_both_and_both_fixes() {
    // The game reads and writes where its own config points. A folder named
    // `por` with a config saying C:\POOLRAD\ silently writes saves elsewhere.
    let dir = tempdir("mismatch").join("por");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("POOL.CFG"), "A\nB\nC:\\POOLRAD\\\n").unwrap();

    let err = manual::folder_name_check(&manual_install(&dir), &por()).unwrap_err();

    assert!(err.contains("por"), "names the folder: {err}");
    assert!(err.contains("POOLRAD"), "names the config's path: {err}");
    assert!(err.contains("POOL.CFG"), "names the config file: {err}");
    assert!(
        err.to_lowercase().contains("rename"),
        "offers the rename fix: {err}"
    );
    assert!(
        err.to_lowercase().contains("mount"),
        "offers the conf mount fix: {err}"
    );
}

#[test]
fn a_missing_dos_config_is_not_checked() {
    // Nothing to compare against is not a mismatch. Refusing here would be
    // guessing.
    let dir = tempdir("nocfg").join("por");
    std::fs::create_dir_all(&dir).unwrap();

    assert!(manual::folder_name_check(&manual_install(&dir), &por()).is_ok());
}

#[test]
fn the_case_of_the_folder_name_does_not_matter() {
    // DOS is case-insensitive; a folder copied as lowercase still works.
    let dir = tempdir("case").join("poolrad");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pool.cfg"), "A\nB\nC:\\POOLRAD\\\n").unwrap();

    assert!(manual::folder_name_check(&manual_install(&dir), &por()).is_ok());
}

#[test]
fn a_discovered_install_is_not_checked() {
    // Discovery matched the folder name to find the install, so a mismatch
    // cannot happen there; the check runs only on the manual path.
    let dir = tempdir("gogkind").join("por");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("POOL.CFG"), "A\nB\nC:\\POOLRAD\\\n").unwrap();
    let mut install = manual_install(&dir);
    install.kind = InstallKind::Gog;

    assert!(manual::folder_name_check(&install, &por()).is_ok());
}

// --- the first-run note --------------------------------------------------------

#[test]
fn the_first_run_note_names_the_conf_files_with_full_paths() {
    let install = Install {
        game: "pool-of-radiance".into(),
        kind: InstallKind::Manual,
        root: "/hand/POOLRAD".into(),
        saves: String::new(),
        confs: vec!["/hand/por.conf".into()],
        emulator: None,
        introduced: false,
    };

    let note = manual::first_run_note(&install).unwrap();

    assert!(note.contains("/hand/por.conf"), "got: {note}");
    assert!(
        note.to_lowercase().contains("settings"),
        "says the settings live there: {note}"
    );
}

#[test]
fn an_introduced_install_gets_no_note() {
    let mut install = manual_install(Path::new("/hand/POOLRAD"));
    install.introduced = true;

    assert_eq!(manual::first_run_note(&install), None);
}

#[test]
fn a_relative_conf_is_shown_under_its_root() {
    let mut install = manual_install(Path::new("/games/por"));
    install.confs = vec!["dosbox_por.conf".into()];

    let note = manual::first_run_note(&install).unwrap();

    assert!(note.contains("/games/por/dosbox_por.conf"), "got: {note}");
}

// --- helpers -----------------------------------------------------------------

fn tempdir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-manual-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}
