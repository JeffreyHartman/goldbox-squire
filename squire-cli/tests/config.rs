//! The config file: a map of installs plus the last choice.

use squire_cli::config::{Config, InstallKind};

#[test]
fn a_v2_config_parses_with_ordered_confs_and_the_last_choice() {
    let text = r#"
        last_install = "steam:pool-of-radiance"

        [installs."steam:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "steam"
        root = "/games/steam/Pool of Radiance"
        saves = "GAME/POOLRAD"
        confs = ["base.conf", "graphics.conf", "game.conf"]
        emulator = "dosbox"
    "#;

    let config = Config::from_toml(text).unwrap();

    assert_eq!(config.last_install.as_deref(), Some("steam:pool-of-radiance"));
    let install = &config.installs["steam:pool-of-radiance"];
    assert_eq!(install.game, "pool-of-radiance");
    assert_eq!(install.kind, InstallKind::Steam);
    assert_eq!(
        install.confs,
        vec!["base.conf", "graphics.conf", "game.conf"],
        "conf order is part of the install"
    );
    assert_eq!(install.emulator.as_deref(), Some("dosbox"));
}

#[test]
fn an_old_flat_config_migrates_to_one_manual_install() {
    // What the current released version writes.
    let text = r#"
        game_dir = "/home/jeff/goldbox/POOLRAD"
        dosbox = "dosbox-staging"
        conf = "/home/jeff/goldbox/por.conf"
    "#;

    let config = Config::from_toml(text).unwrap();

    assert_eq!(config.installs.len(), 1);
    let (key, install) = config.installs.iter().next().unwrap();
    assert_eq!(install.kind, InstallKind::Manual);
    assert_eq!(install.root, "/home/jeff/goldbox/POOLRAD");
    assert_eq!(install.saves, "", "the old game_dir was the save folder");
    assert_eq!(install.confs, vec!["/home/jeff/goldbox/por.conf"]);
    assert_eq!(install.emulator.as_deref(), Some("dosbox-staging"));
    assert_eq!(
        config.last_install.as_deref(),
        Some(key.as_str()),
        "the migrated install is the default"
    );
}

#[test]
fn a_migrated_config_round_trips() {
    let old = r#"
        game_dir = "/saves/POOLRAD"
        conf = "/saves/por.conf"
    "#;

    let migrated = Config::from_toml(old).unwrap();
    let again = Config::from_toml(&migrated.to_toml().unwrap()).unwrap();

    assert_eq!(migrated, again);
}

#[test]
fn a_v2_config_round_trips() {
    let text = r#"
        last_install = "gog:pool-of-radiance"
        extra_roots = ["/mnt/games"]

        [installs."gog:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "gog"
        root = "/home/jeff/GOG Games/por"
        saves = "data/POOLRAD"
        confs = ["dosbox_por.conf", "dosbox_por_single.conf"]
    "#;

    let config = Config::from_toml(text).unwrap();
    let again = Config::from_toml(&config.to_toml().unwrap()).unwrap();

    assert_eq!(config, again);
    assert_eq!(config.extra_roots, vec!["/mnt/games"]);
}

#[test]
fn the_save_folder_is_root_joined_with_saves() {
    let text = r#"
        [installs."gog:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "gog"
        root = "/games/por"
        saves = "data/POOLRAD"
        confs = ["a.conf"]
    "#;

    let config = Config::from_toml(text).unwrap();
    let install = &config.installs["gog:pool-of-radiance"];

    assert_eq!(
        install.save_dir().to_string_lossy(),
        "/games/por/data/POOLRAD"
    );
}

#[test]
fn an_empty_saves_means_the_root_itself() {
    // A manual install's game_dir is the save folder.
    let text = r#"
        [installs."manual:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "manual"
        root = "/saves/POOLRAD"
        saves = ""
        confs = []
    "#;

    let config = Config::from_toml(text).unwrap();
    let install = &config.installs["manual:pool-of-radiance"];

    assert_eq!(install.save_dir().to_string_lossy(), "/saves/POOLRAD");
}

#[test]
fn no_config_field_stores_a_save_slot() {
    // A slot describes one sitting. Remembering it would pin the user to a
    // save they stopped playing, which is the bug this rework removes.
    let text = r#"
        last_install = "gog:pool-of-radiance"

        [installs."gog:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "gog"
        root = "/games/por"
        saves = "data/POOLRAD"
        confs = ["a.conf"]
    "#;

    let written = Config::from_toml(text).unwrap().to_toml().unwrap();

    assert!(
        !written.to_lowercase().contains("slot"),
        "the config must never mention a slot: {written}"
    );
}

#[test]
fn garbage_gives_an_error_not_a_panic() {
    assert!(Config::from_toml("this is not toml [[[").is_err());
}

// --- discovered installs are cached in the config ---------------------------

use squire_core::discover::{DiscoveredInstall, Publisher};
use std::path::PathBuf;

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

fn discovered(root: &std::path::Path, publisher: Option<Publisher>) -> DiscoveredInstall {
    DiscoveredInstall {
        game_id: "pool-of-radiance".into(),
        publisher,
        root: root.to_path_buf(),
        saves: PathBuf::from("data/POOLRAD"),
        confs: vec!["a.conf".into(), "b.conf".into()],
        emulator: None,
    }
}

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
    assert_eq!(install.confs, vec!["a.conf", "b.conf"]);
}

#[test]
fn absorbing_the_same_results_again_changes_nothing() {
    let root = tempdir("absorb-twice");
    let mut config = Config::default();
    config.absorb(vec![discovered(&root, Some(Publisher::Steam))]);

    let changed = config.absorb(vec![discovered(&root, Some(Publisher::Steam))]);

    assert!(!changed, "a normal run must not rewrite the file");
}

#[test]
fn a_vanished_root_asks_for_rediscovery() {
    let root = tempdir("vanish");
    let mut config = Config::default();
    config.absorb(vec![discovered(&root, Some(Publisher::Gog))]);

    assert!(!config.needs_rediscovery());
    std::fs::remove_dir_all(&root).unwrap();
    assert!(config.needs_rediscovery());
}

#[test]
fn a_manual_install_never_triggers_rediscovery() {
    // Manual means the user named the pieces; a scan cannot find them again,
    // so a vanished manual root is the user's to fix, not a reason to rescan.
    let text = r#"
        [installs."manual:pool-of-radiance"]
        game = "pool-of-radiance"
        kind = "manual"
        root = "/gone/away"
        saves = ""
        confs = ["por.conf"]
    "#;
    let config = Config::from_toml(text).unwrap();

    assert!(!config.needs_rediscovery());
}

#[test]
fn rediscovery_drops_a_cached_install_whose_root_vanished() {
    let stays = tempdir("stays");
    let goes = tempdir("goes");
    let mut config = Config::default();
    config.absorb(vec![discovered(&stays, Some(Publisher::Gog))]);
    let mut second = discovered(&goes, Some(Publisher::Steam));
    second.game_id = "pool-of-radiance".into();
    config.absorb(vec![second]);
    std::fs::remove_dir_all(&goes).unwrap();

    config.absorb(vec![discovered(&stays, Some(Publisher::Gog))]);

    assert!(config.installs.contains_key("gog:pool-of-radiance"));
    assert!(
        !config.installs.contains_key("steam:pool-of-radiance"),
        "a cached result whose root vanished is stale, not remembered"
    );
}

#[test]
fn two_installs_of_the_same_kind_and_game_get_distinct_keys() {
    let one = tempdir("dup-one");
    let two = tempdir("dup-two");
    let mut config = Config::default();

    config.absorb(vec![
        discovered(&one, Some(Publisher::Gog)),
        discovered(&two, Some(Publisher::Gog)),
    ]);

    assert_eq!(config.installs.len(), 2, "{:?}", config.installs.keys());
}
