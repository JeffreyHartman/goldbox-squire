//! The manual directory's guard: the folder-name check.

use std::path::{Path, PathBuf};

use squire_cli::config::{Install, InstallKind};
use squire_cli::manual;
use squire_core::games;

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
