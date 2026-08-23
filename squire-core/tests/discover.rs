//! Install discovery: finding a game install by its shape.
//!
//! An install is a directory holding a conf with an `[autoexec]` mount and a
//! save folder (named per game) holding `CHRDAT*.SAV` files. See ADR 0001.

use std::path::{Path, PathBuf};

use squire_core::discover::{self, Publisher};

// --- the two real layouts, in miniature -------------------------------------

/// GOG's Pool of Radiance layout: two confs run in start.sh order, saves under
/// `data/POOLRAD`, a bundled emulator under `dosbox/`.
fn gog_tree(base: &Path) -> PathBuf {
    let root = base.join("Pool of Radiance");
    mkdir(&root.join("data/POOLRAD"));
    write_save(&root.join("data/POOLRAD"), "CHRDATA1.SAV", "HERO");
    // Alphabetical order and launch order disagree on purpose: the launch
    // script, not the directory listing, decides.
    conf(&root, "dosbox_por_single.conf", false);
    conf(&root, "dosbox_por.conf", true);
    std::fs::write(
        root.join("start.sh"),
        "#!/bin/bash\n\
         dosbox -conf dosbox_por.conf -conf dosbox_por_single.conf -no-console\n",
    )
    .unwrap();
    mkdir(&root.join("dosbox"));
    executable(&root.join("dosbox/dosbox"));
    root
}

/// Steam's layout: three confs run in run-game.bat order, saves under
/// `GAME/POOLRAD`, no bundled emulator.
fn steam_tree(base: &Path) -> PathBuf {
    let root = base.join("Pool of Radiance");
    mkdir(&root.join("GAME/POOLRAD"));
    write_save(&root.join("GAME/POOLRAD"), "CHRDATA1.SAV", "HERO");
    conf(&root, "game.conf", true);
    conf(&root, "base.conf", false);
    conf(&root, "graphics.conf", false);
    std::fs::write(
        root.join("run-game.bat"),
        "dosbox.exe -conf base.conf -conf graphics.conf -conf game.conf\r\n",
    )
    .unwrap();
    root
}

#[test]
fn finds_a_gog_shaped_tree() {
    let base = tempdir();
    gog_tree(&base);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1);
    let install = &found[0];
    assert_eq!(install.game_id, "pool-of-radiance");
    assert_eq!(install.publisher, Some(Publisher::Gog));
    assert_eq!(
        install.confs,
        vec!["dosbox_por.conf", "dosbox_por_single.conf"],
        "conf order comes from start.sh, not from the directory listing"
    );
    assert_eq!(install.saves, PathBuf::from("data/POOLRAD"));
}

#[test]
fn finds_a_steam_shaped_tree() {
    let base = tempdir();
    steam_tree(&base);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1);
    let install = &found[0];
    assert_eq!(install.publisher, Some(Publisher::Steam));
    assert_eq!(
        install.confs,
        vec!["base.conf", "graphics.conf", "game.conf"],
        "conf order comes from run-game.bat"
    );
    assert_eq!(install.saves, PathBuf::from("GAME/POOLRAD"));
}

#[test]
fn a_tree_with_no_launch_script_puts_the_autoexec_conf_last() {
    // Both publishers follow this rule, so it is the fallback when no script
    // says otherwise.
    let base = tempdir();
    let root = base.join("por");
    mkdir(&root.join("data/POOLRAD"));
    write_save(&root.join("data/POOLRAD"), "CHRDATA1.SAV", "HERO");
    conf(&root, "aaa_game.conf", true); // alphabetically first, runs last
    conf(&root, "zzz_settings.conf", false);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].publisher, None);
    assert_eq!(found[0].confs, vec!["zzz_settings.conf", "aaa_game.conf"]);
}

#[test]
fn a_tree_without_save_files_is_not_an_install() {
    let base = tempdir();
    let root = base.join("empty");
    mkdir(&root.join("data/POOLRAD"));
    conf(&root, "dosbox.conf", true);

    assert!(discover::discover(std::slice::from_ref(&base)).is_empty());
}

#[test]
fn a_tree_whose_conf_has_no_autoexec_mount_is_not_an_install() {
    let base = tempdir();
    let root = base.join("no-mount");
    mkdir(&root.join("data/POOLRAD"));
    write_save(&root.join("data/POOLRAD"), "CHRDATA1.SAV", "HERO");
    std::fs::write(root.join("dosbox.conf"), "[sdl]\nfullscreen=false\n").unwrap();

    assert!(discover::discover(std::slice::from_ref(&base)).is_empty());
}

#[test]
fn the_scan_stops_four_levels_below_a_root() {
    let base = tempdir();
    let deep = base.join("a/b/c/d/e");
    gog_tree(&deep); // the install sits five levels down, one too far

    assert!(discover::discover(std::slice::from_ref(&base)).is_empty());

    let shallow = tempdir();
    let ok = shallow.join("a/b/c");
    gog_tree(&ok); // "Pool of Radiance" is the fourth level: found
    assert_eq!(discover::discover(std::slice::from_ref(&shallow)).len(), 1);
}

#[test]
fn a_bundled_emulator_wins_over_path() {
    let base = tempdir();
    let root = gog_tree(&base);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(
        found[0].emulator.as_deref(),
        Some(root.join("dosbox/dosbox").as_path()),
        "the publisher shipped a build known to run their conf"
    );
}

#[test]
fn an_install_without_a_bundled_emulator_names_none() {
    let base = tempdir();
    steam_tree(&base);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found[0].emulator, None, "PATH is the fallback, decided later");
}

#[test]
fn a_missing_root_is_skipped_not_an_error() {
    let base = tempdir();
    gog_tree(&base);
    let ghost = PathBuf::from("/no/such/root/anywhere");

    let found = discover::discover(&[ghost, base.clone()]);

    assert_eq!(found.len(), 1);
}

// --- helpers -----------------------------------------------------------------

fn tempdir() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-discover-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

fn mkdir(p: &Path) {
    std::fs::create_dir_all(p).unwrap();
}

/// Writes a conf file, with or without the `[autoexec]` mount section.
fn conf(root: &Path, name: &str, autoexec: bool) {
    let text = if autoexec {
        "[sdl]\nfullscreen=false\n\n[autoexec]\nmount c \"data\"\nc:\ncd POOLRAD\nPOOL.EXE\nexit\n"
    } else {
        "[render]\nscaler=none\n"
    };
    std::fs::write(root.join(name), text).unwrap();
}

/// Writes an empty file with the execute bit set, a stand-in binary.
fn executable(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(p, "#!/bin/true\n").unwrap();
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn write_save(dir: &Path, file: &str, name: &str) {
    let mut bytes = vec![0u8; 285];
    bytes[0] = name.len() as u8;
    bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
    std::fs::write(dir.join(file), bytes).unwrap();
}
