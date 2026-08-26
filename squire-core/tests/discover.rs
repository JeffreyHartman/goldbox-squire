//! Install discovery: finding a game install by its shape.
//!
//! An install is a directory holding a conf with an `[autoexec]` mount and a
//! save folder (named per game) holding `CHRDAT*.SAV` files. See ADR 0001.

use std::path::{Path, PathBuf};

use squire_core::discover::{self, Publisher};
use squire_core::games;

fn por() -> games::Game {
    games::find("pool-of-radiance").expect("Pool of Radiance is compiled in")
}

fn frua() -> games::Game {
    games::find("unlimited-adventures").expect("Unlimited Adventures is compiled in")
}

// --- the two real layouts, in miniature -------------------------------------

/// GOG's Pool of Radiance layout: two confs run in start.sh order, saves under
/// `data/POOLRAD`, a bundled emulator under `dosbox/`.
fn gog_tree(base: &Path) -> PathBuf {
    let root = base.join("Pool of Radiance");
    mkdir(&root.join("data/POOLRAD"));
    std::fs::write(root.join("data/POOLRAD/START.EXE"), b"MZ").unwrap();
    write_save(&root.join("data/POOLRAD"), "CHRDATA1.SAV", "HERO");
    // Alphabetical order and launch order disagree on purpose: the launch
    // script, not the directory listing, decides.
    conf(&root, "dosbox_por_single.conf", false);
    conf(&root, "dosbox_por.conf", true);
    // The real script names the confs as quoted arguments to a helper, not
    // as -conf flags. Taken verbatim from a 2024 GOG install.
    std::fs::write(
        root.join("start.sh"),
        "#!/bin/bash\n\
         run_dosbox \"dosbox_por.conf\" \"dosbox_por_single.conf\" \"${@}\"\n",
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
    mkdir(&root.join("GAME/POOLRAD/SAVE"));
    std::fs::write(root.join("GAME/POOLRAD/START.EXE"), b"MZ").unwrap();
    write_save(&root.join("GAME/POOLRAD/SAVE"), "CHRDATA1.SAV", "HERO");
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
        install.saves,
        PathBuf::from("GAME/POOLRAD/SAVE"),
        "Steam keeps the save files one level inside POOLRAD"
    );
}

#[test]
fn a_tree_with_no_launch_script_has_no_publisher() {
    let base = tempdir();
    let root = base.join("por");
    mkdir(&root.join("data/POOLRAD"));
    std::fs::write(root.join("data/POOLRAD/START.EXE"), b"MZ").unwrap();
    write_save(&root.join("data/POOLRAD"), "CHRDATA1.SAV", "HERO");
    conf(&root, "game.conf", true);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].publisher, None);
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
fn a_hand_conf_above_a_publisher_install_collapses_to_the_publisher_one() {
    // A hand-written conf next to the GOG folder makes the parent directory
    // look like an install of the same game folder. That is one install,
    // not two, and the publisher-scripted reading of it wins (ticket 026).
    let base = tempdir();
    let goldbox = base.join("goldbox");
    mkdir(&goldbox);
    gog_tree(&goldbox);
    conf(&goldbox, "por.conf", true);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].publisher, Some(Publisher::Gog));
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

#[test]
fn two_roots_reaching_the_same_install_yield_one_install() {
    // ~/.steam/steam is a symlink into ~/.local/share/Steam, and both are
    // search roots, so the same install is reachable twice.
    let base = tempdir();
    let real = base.join("real");
    mkdir(&real);
    gog_tree(&real);
    let alias = base.join("alias");
    std::os::unix::fs::symlink(&real, &alias).unwrap();

    let found = discover::discover(&[alias, real]);

    assert_eq!(found.len(), 1, "{:?}", found);
}

#[test]
fn saves_within_finds_chrdat_files_in_the_folder_itself() {
    let base = tempdir();
    let dir = base.join("POOLRAD");
    mkdir(&dir);
    write_save(&dir, "CHRDATA1.SAV", "HERO");

    assert_eq!(discover::saves_within(&dir, &por()), Some(PathBuf::new()));
}

#[test]
fn saves_within_finds_chrdat_files_one_child_down() {
    // Steam keeps them in a SAVE child inside the game folder.
    let base = tempdir();
    let dir = base.join("POOLRAD");
    mkdir(&dir.join("SAVE"));
    write_save(&dir.join("SAVE"), "CHRDATA1.SAV", "HERO");

    assert_eq!(
        discover::saves_within(&dir, &por()),
        Some(PathBuf::from("SAVE"))
    );
}

#[test]
fn saves_within_is_none_when_there_are_no_saves() {
    let base = tempdir();
    let dir = base.join("POOLRAD");
    mkdir(&dir);

    assert_eq!(discover::saves_within(&dir, &por()), None);
}

#[test]
fn a_predecessors_save_stub_is_not_an_install() {
    // Every sequel ships a stub of its predecessor for party import:
    // Treasures holds GAME/GATEWAY/SAVE with save files and nothing else.
    // A folder with saves but no start file holds no game.
    let base = tempdir();
    let root = base.join("Treasures of the Savage Frontier");
    mkdir(&root.join("GAME/GATEWAY/SAVE"));
    write_save(&root.join("GAME/GATEWAY/SAVE"), "CHRDATA1.SAV", "STUB");
    conf(&root, "game.conf", true);

    let found = discover::discover(std::slice::from_ref(&base));

    assert!(found.is_empty(), "{found:?}");

    // The same folder with the game's start file present is an install.
    std::fs::write(root.join("GAME/GATEWAY/GO.BAT"), b"game\r\n").unwrap();
    let found = discover::discover(std::slice::from_ref(&base));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].game_id, "gateway-to-the-savage-frontier");
}

// --- Unlimited Adventures: saves per design ----------------------------------

/// An Unlimited Adventures tree the way the user's own copy is laid out: a
/// conf next to a UA folder, saves per design under `{name}.DSN/SAVE/`.
fn frua_tree(base: &Path) -> PathBuf {
    let root = base.join("frua");
    let save = root.join("UA/BASILISK.DSN/SAVE");
    mkdir(&save);
    std::fs::write(root.join("UA/START.BAT"), b"ckit\r\n").unwrap();
    std::fs::write(save.join("SAVGAMA.CSV"), vec![0u8; 64]).unwrap();
    std::fs::write(
        root.join("frua.conf"),
        "[autoexec]\nmount c .\nc:\ncd UA\nSTART.BAT\n",
    )
    .unwrap();
    root
}

#[test]
fn finds_a_frua_tree_and_records_the_game_folder_as_the_save_path() {
    let base = tempdir();
    frua_tree(&base);

    let found = discover::discover(std::slice::from_ref(&base));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].game_id, "unlimited-adventures");
    // The design is chosen later, so the recorded path is the game folder.
    assert_eq!(found[0].saves, PathBuf::from("UA"));
}

#[test]
fn saves_within_accepts_a_frua_game_folder_with_a_design() {
    let base = tempdir();
    let root = frua_tree(&base);

    assert_eq!(
        discover::saves_within(&root.join("UA"), &frua()),
        Some(PathBuf::new())
    );
}

#[test]
fn saves_within_rejects_a_frua_folder_with_designs_but_no_save_files() {
    let base = tempdir();
    let dir = base.join("UA/EMPTY.DSN/SAVE");
    mkdir(&dir);

    assert_eq!(discover::saves_within(&base.join("UA"), &frua()), None);
}
