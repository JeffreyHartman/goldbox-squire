//! The gbs-owned launch configuration for discovered installs (ADR 0003).
//!
//! Two parts: a per-game settings conf the user edits, and a computed
//! autoexec gbs passes as `-c` commands.

use std::path::PathBuf;

use squire_cli::conf;
use squire_cli::config::{Install, InstallKind};
use squire_core::games;

#[test]
fn a_missing_settings_conf_is_created_from_the_template() {
    let dir = tempdir("create");
    let game = games::find("pool-of-radiance").unwrap();

    let (path, created) = conf::ensure(&dir, &game).unwrap();

    assert!(created);
    assert_eq!(path, dir.join("pool-of-radiance.conf"));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("Pool of Radiance"), "got: {text}");
    // The defaults proven in play (ticket 029), plus both sound devices an
    // install's own DOS config can name: GOG ships PC speaker, Steam Tandy.
    for setting in [
        "fullscreen = false",
        "machine = ega",
        "pcspeaker = impulse",
        "pcspeaker_filter = on",
        "tandy = on",
        "compressor = off",
        "mouse_capture = seamless",
    ] {
        assert!(text.contains(setting), "missing {setting} in: {text}");
    }
}

#[test]
fn an_existing_settings_conf_is_left_alone() {
    let dir = tempdir("keep");
    let game = games::find("pool-of-radiance").unwrap();
    let mine = "[sdl]\nfullscreen = true\n";
    std::fs::write(dir.join("pool-of-radiance.conf"), mine).unwrap();

    let (path, created) = conf::ensure(&dir, &game).unwrap();

    assert!(!created);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), mine);
}

#[test]
fn the_autoexec_mounts_the_folder_above_the_game_folder_gog_shape() {
    let install = install("/home/x/gog-por", "data/POOLRAD");
    let game = games::find("pool-of-radiance").unwrap();

    let commands = conf::autoexec(&install, &game).unwrap();

    assert_eq!(
        commands,
        vec![
            "mount c \"/home/x/gog-por/data\"".to_string(),
            "c:".into(),
            "cd POOLRAD".into(),
            "START.EXE".into(),
            "exit".into(),
        ]
    );
}

#[test]
fn the_autoexec_handles_the_steam_shape_where_saves_sit_deeper() {
    // Steam keeps CHRDAT files in GAME/POOLRAD/SAVE; the game folder is
    // still GAME/POOLRAD and the mount is GAME. Spaces in the install path
    // ride inside the quotes.
    let install = install(
        "/home/x/steamapps/common/Forgotten Realms",
        "GAME/POOLRAD/SAVE",
    );
    let game = games::find("pool-of-radiance").unwrap();

    let commands = conf::autoexec(&install, &game).unwrap();

    assert_eq!(
        commands[0],
        "mount c \"/home/x/steamapps/common/Forgotten Realms/GAME\""
    );
    assert_eq!(commands[2], "cd POOLRAD");
}

#[test]
fn the_autoexec_handles_an_install_whose_root_is_the_game_folder() {
    // A typed path points straight at the game folder, so the recorded save
    // path is empty and the folder to mount is the root's parent. This is
    // every manual Unlimited Adventures install, and any chrdat game whose
    // typed folder holds the saves directly.
    let install = Install {
        game: "unlimited-adventures".into(),
        kind: InstallKind::Manual,
        root: "/home/x/frua/UA".into(),
        saves: String::new(),
    };
    let game = games::find("unlimited-adventures").unwrap();

    let commands = conf::autoexec(&install, &game).unwrap();

    assert_eq!(
        commands,
        vec![
            "mount c \"/home/x/frua\"".to_string(),
            "c:".into(),
            "cd UA".into(),
            "START.BAT".into(),
            "exit".into(),
        ]
    );
}

#[test]
fn a_saves_path_without_the_game_folder_is_an_error_naming_it() {
    let install = install("/home/x/somewhere", "saves");
    let game = games::find("pool-of-radiance").unwrap();

    let err = conf::autoexec(&install, &game).unwrap_err();

    assert!(err.contains("POOLRAD"), "got: {err}");
}

// --- helpers -----------------------------------------------------------------

fn install(root: &str, saves: &str) -> Install {
    Install {
        game: "pool-of-radiance".into(),
        kind: InstallKind::Gog,
        root: root.into(),
        saves: saves.into(),
    }
}

fn tempdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gbs-conf-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
