//! The config file: game directories, and which one each game uses.

use std::path::{Path, PathBuf};

use squire_cli::config::{Config, Install, InstallKind};
use squire_core::discover::{DiscoveredInstall, Publisher};

// --- migration ----------------------------------------------------------------

#[test]
fn a_v3_config_round_trips() {
    let mut config = Config {
        last_game: Some("pool-of-radiance".into()),
        dosbox: Some("dosbox-staging".into()),
        ..Default::default()
    };
    config.installs.insert(
        "gog:pool-of-radiance".into(),
        Install {
            game: "pool-of-radiance".into(),
            kind: InstallKind::Gog,
            root: "/games/por".into(),
            saves: "data/POOLRAD".into(),
        },
    );
    config
        .chosen
        .insert("pool-of-radiance".into(), "gog:pool-of-radiance".into());

    let text = config.to_toml().unwrap();
    let back = Config::from_toml(&text).unwrap();

    assert_eq!(back, config);
}

#[test]
fn a_v2_config_migrates_the_last_choice_to_a_chosen_directory() {
    // v2 carried confs, emulators and a single last_install. The directories
    // survive; the launch details are gbs's own now (ADR 0004).
    let text = r#"
        last_install = "gog:pool-of-radiance"

        [installs."gog:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "gog"
        root = "/games/por"
        saves = "data/POOLRAD"
        confs = ["dosbox_por.conf", "dosbox_por_single.conf"]
        emulator = "/games/por/dosbox/dosbox"
        introduced = true

        [installs."manual:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "manual"
        root = "/hand/POOLRAD"
        saves = ""
        confs = ["/hand/por.conf"]
    "#;

    let config = Config::from_toml(text).unwrap();

    assert_eq!(config.last_game.as_deref(), Some("pool-of-radiance"));
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("gog:pool-of-radiance")
    );
    assert_eq!(config.installs.len(), 2);
    assert_eq!(
        config.installs["manual:pool-of-radiance"].root,
        "/hand/POOLRAD"
    );
}

#[test]
fn a_v1_config_migrates_to_one_chosen_manual_install() {
    let text = r#"
        game_dir = "/home/x/goldbox/POOLRAD"
        dosbox = "dosbox-staging"
        conf = "/home/x/goldbox/por.conf"
    "#;

    let config = Config::from_toml(text).unwrap();

    let install = &config.installs["manual:pool-of-radiance"];
    assert_eq!(install.kind, InstallKind::Manual);
    assert_eq!(install.root, "/home/x/goldbox/POOLRAD");
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("manual:pool-of-radiance")
    );
    assert_eq!(config.last_game.as_deref(), Some("pool-of-radiance"));
    // The v1 emulator choice becomes the permanent override; the conf has no
    // successor and is dropped.
    assert_eq!(config.dosbox.as_deref(), Some("dosbox-staging"));
}

#[test]
fn no_config_field_stores_a_save_slot() {
    // ADR 0002: a slot describes one sitting and is asked every run.
    let mut config = Config {
        last_game: Some("pool-of-radiance".into()),
        ..Default::default()
    };
    config
        .chosen
        .insert("pool-of-radiance".into(), "gog:pool-of-radiance".into());

    let text = config.to_toml().unwrap().to_ascii_lowercase();

    assert!(!text.contains("slot"), "got: {text}");
}

#[test]
fn garbage_gives_an_error_not_a_panic() {
    assert!(Config::from_toml("[[[not toml").is_err());
}

// --- the save folder ----------------------------------------------------------

#[test]
fn the_save_folder_is_root_joined_with_saves() {
    let install = Install {
        game: "pool-of-radiance".into(),
        kind: InstallKind::Steam,
        root: "/steam/POOLRAD".into(),
        saves: "GAME/POOLRAD/SAVE".into(),
    };

    assert_eq!(
        install.save_dir(),
        PathBuf::from("/steam/POOLRAD/GAME/POOLRAD/SAVE")
    );
}

#[test]
fn an_empty_saves_means_the_root_itself() {
    let install = Install {
        game: "pool-of-radiance".into(),
        kind: InstallKind::Manual,
        root: "/hand/POOLRAD".into(),
        saves: String::new(),
    };

    assert_eq!(install.save_dir(), PathBuf::from("/hand/POOLRAD"));
}

// --- absorbing discovery results ----------------------------------------------

#[test]
fn a_discovered_install_is_written_into_the_config() {
    let root = tempdir("absorb");
    let mut config = Config::default();

    let changed = config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    assert!(changed);
    let install = &config.installs["gog:pool-of-radiance"];
    assert_eq!(install.kind, InstallKind::Gog);
    assert_eq!(install.root, root.to_string_lossy());
    assert_eq!(install.saves, "data/POOLRAD");
}

#[test]
fn absorb_replaces_discovered_installs_wholesale() {
    // A rescan is the authority on discovered installs. A stale entry whose
    // root still exists (the old found:/gog: duplicate) must not survive it.
    let root = tempdir("wholesale");
    let mut config = Config::default();
    config.absorb(vec![discovered(&root, None)]);
    assert!(config.installs.contains_key("found:pool-of-radiance"));

    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    assert!(
        !config.installs.contains_key("found:pool-of-radiance"),
        "the stale reading of the same install survived: {:?}",
        config.installs.keys().collect::<Vec<_>>()
    );
    assert!(config.installs.contains_key("gog:pool-of-radiance"));
}

#[test]
fn a_manual_directory_a_discovered_install_also_names_is_dropped() {
    // The user pointed at the same folder discovery later found: that is one
    // install, and the discovered reading of it wins.
    let root = tempdir("manual-dup");
    std::fs::create_dir_all(root.join("data/POOLRAD")).unwrap();
    let mut config = Config::default();
    let dir = root.join("data/POOLRAD");
    config.choose_manual_dir("pool-of-radiance", &dir.to_string_lossy(), "");

    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    let manual_left = config
        .installs
        .values()
        .any(|i| i.kind == InstallKind::Manual);
    assert!(
        !manual_left,
        "{:?}",
        config.installs.keys().collect::<Vec<_>>()
    );
}

#[test]
fn a_manual_directory_elsewhere_survives_absorb() {
    let root = tempdir("manual-stays");
    let elsewhere = tempdir("manual-stays-elsewhere");
    let mut config = Config::default();
    config.choose_manual_dir("pool-of-radiance", &elsewhere.to_string_lossy(), "");

    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    assert!(config
        .installs
        .values()
        .any(|i| i.kind == InstallKind::Manual));
}

#[test]
fn a_dangling_chosen_entry_is_cleared() {
    // The chosen key can vanish in a rescan; the wizard must then ask again
    // rather than index a missing install.
    let root = tempdir("dangling");
    let mut config = Config::default();
    config
        .chosen
        .insert("pool-of-radiance".into(), "steam:pool-of-radiance".into());

    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    assert!(config.chosen.is_empty());
}

// --- when a rescan is needed ---------------------------------------------------

#[test]
fn a_config_with_no_discovered_installs_triggers_rediscovery() {
    // A v1 config migrates to one manual install, which used to sail past
    // both scan triggers: the user never saw their GOG and Steam installs.
    let mut config = Config::default();
    config.choose_manual_dir("pool-of-radiance", "/hand/POOLRAD", "");

    assert!(config.needs_rediscovery());
}

#[test]
fn a_vanished_discovered_root_triggers_rediscovery() {
    let alive = tempdir("alive");
    let mut config = Config::default();
    config.absorb(vec![discovered(&alive, Some(Publisher::Gog))]);
    assert!(!config.needs_rediscovery());

    std::fs::remove_dir_all(&alive).unwrap();

    assert!(config.needs_rediscovery());
}

#[test]
fn a_vanished_manual_root_never_triggers_rediscovery() {
    // Manual means the user named the pieces; a scan cannot find them again,
    // so a vanished manual root is the user's to fix, not a reason to rescan.
    let healthy = tempdir("manual-no-rescan");
    let mut config = Config::default();
    config.absorb(vec![discovered(&healthy, Some(Publisher::Gog))]);
    config.choose_manual_dir("pool-of-radiance", "/gone/away", "");

    assert!(!config.needs_rediscovery());
}

#[test]
fn a_manual_duplicate_of_a_discovered_folder_triggers_rediscovery() {
    // A migrated config can arrive with a manual entry naming the folder a
    // discovered install also names. Only absorb dedups, so it must rescan.
    let root = tempdir("manual-dup-rescan");
    std::fs::create_dir_all(root.join("data/POOLRAD")).unwrap();
    let mut config = Config::default();
    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);
    let dir = root.join("data/POOLRAD");
    config.choose_manual_dir("pool-of-radiance", &dir.to_string_lossy(), "");

    assert!(config.needs_rediscovery());
}

#[test]
fn two_installs_reaching_one_game_folder_trigger_rediscovery() {
    // An existing config can carry the found:/gog: duplicate from before
    // ticket 026. Only a rescan can collapse it, so it must trigger one.
    let base = tempdir("dup-rediscover");
    let inner = base.join("pool-of-radiance");
    std::fs::create_dir_all(inner.join("data/POOLRAD")).unwrap();
    let mut config = Config::default();
    config.absorb(vec![discovered(&inner, Some(Publisher::Gog))]);
    assert!(!config.needs_rediscovery());

    let mut hand = discovered(&base, None);
    hand.saves = PathBuf::from("pool-of-radiance/data/POOLRAD");
    config.absorb(vec![discovered(&inner, Some(Publisher::Gog)), hand]);

    assert!(config.needs_rediscovery());
}

// --- helpers -------------------------------------------------------------------

fn discovered(root: &Path, publisher: Option<Publisher>) -> DiscoveredInstall {
    DiscoveredInstall {
        game_id: "pool-of-radiance".into(),
        publisher,
        root: root.to_path_buf(),
        saves: PathBuf::from("data/POOLRAD"),
    }
}

fn tempdir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-config-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn the_window_size_is_remembered_under_a_key_that_says_what_it_is() {
    let config = Config::from_toml("[hud]\ncolumns = 120\nrows = 40\n").unwrap();
    let hud = config.hud.expect("the size was stored");
    assert_eq!((hud.columns, hud.rows), (120, 40));
    assert!(config.to_toml().unwrap().contains("[hud]"));
}

#[test]
fn a_nonsensical_stored_size_is_ignored_rather_than_fatal() {
    // A terminal that does not know its own size reports zero, and a hand
    // edit can say anything at all. Neither is worth refusing to start over.
    for text in [
        "[hud]\ncolumns = 0\nrows = 40\n",
        "[hud]\ncolumns = 120\nrows = 0\n",
        "[hud]\ncolumns = \"wide\"\nrows = 40\n",
        "[hud]\n",
    ] {
        let config = Config::from_toml(text).unwrap_or_else(|e| panic!("{text:?}: {e}"));
        assert!(config.hud.is_none(), "{text:?} was believed");
    }
}

#[test]
fn a_config_from_before_the_hud_loads_unchanged() {
    let before = "last_game = \"pool-of-radiance\"\n";
    let config = Config::from_toml(before).unwrap();
    assert_eq!(config.last_game.as_deref(), Some("pool-of-radiance"));
    assert!(config.hud.is_none());
    assert!(!config.to_toml().unwrap().contains("hud"));
}
