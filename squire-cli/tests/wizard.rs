//! The wizard: two questions, answerable in advance, defaulting to Enter.
//!
//! No terminal is involved: input is plain lines in, prompts are lines out.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use squire_cli::config::{Config, Install, InstallKind};
use squire_cli::wizard;

/// Runs the wizard flow against scripted input, returning its pick and what
/// it printed.
fn choose(
    config: &Config,
    game_arg: Option<&str>,
    slot_arg: Option<char>,
    input: &str,
) -> (Result<(String, char), String>, String) {
    let mut output = Vec::new();
    let result = wizard::choose(
        &mut Cursor::new(input.as_bytes().to_vec()),
        &mut output,
        config,
        game_arg,
        slot_arg,
    );
    (result, String::from_utf8(output).unwrap())
}

/// A config with two installs of Pool of Radiance and saves on disk.
///
/// The GOG install's folder holds slots A and J; the manual install's holds
/// only slot C.
fn two_installs() -> Config {
    let gog_saves = tempdir("gog");
    write_save(&gog_saves, "CHRDATA1.SAV", "GOG HERO");
    write_save(&gog_saves, "CHRDATJ1.SAV", "JULIET ONE");
    write_save(&gog_saves, "CHRDATJ2.SAV", "JULIET TWO");
    let manual_saves = tempdir("manual");
    write_save(&manual_saves, "CHRDATC1.SAV", "HAND ROLLED");

    let mut config = Config::default();
    config.installs.insert(
        "gog:pool-of-radiance".into(),
        install(InstallKind::Gog, &gog_saves),
    );
    config.installs.insert(
        "manual:pool-of-radiance".into(),
        install(InstallKind::Manual, &manual_saves),
    );
    config
}

#[test]
fn first_run_picks_an_install_by_number_and_a_slot_by_letter() {
    let config = two_installs();

    let (result, printed) = choose(&config, None, None, "1\nJ\n");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
    assert!(printed.contains("Pool of Radiance"), "{printed}");
    assert!(printed.contains("GOG"), "the install kind is shown: {printed}");
}

#[test]
fn a_returning_user_is_two_enters_from_a_running_game() {
    let mut config = two_installs();
    config.last_install = Some("manual:pool-of-radiance".into());

    let (result, _) = choose(&config, None, None, "\n\n");

    // Enter takes the remembered install, then the first populated slot.
    assert_eq!(result.unwrap(), ("manual:pool-of-radiance".into(), 'C'));
}

#[test]
fn the_slot_list_shows_the_party_names() {
    let config = two_installs();

    let (_, printed) = choose(&config, None, None, "1\nJ\n");

    assert!(
        printed.contains("JULIET ONE") && printed.contains("JULIET TWO"),
        "picking by recognising your party beats remembering a letter: {printed}"
    );
}

#[test]
fn naming_the_game_skips_the_install_question() {
    let mut config = two_installs();
    config.last_install = Some("gog:pool-of-radiance".into());

    let (result, printed) = choose(&config, Some("pool-of-radiance"), None, "J\n");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
    assert!(
        !printed.contains("Which game"),
        "the answered question is not asked: {printed}"
    );
}

#[test]
fn naming_the_slot_skips_the_slot_question() {
    let mut config = two_installs();
    config.last_install = Some("gog:pool-of-radiance".into());

    let (result, printed) = choose(&config, None, Some('J'), "\n");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
    assert!(!printed.contains("Which save slot"), "{printed}");
}

#[test]
fn naming_both_asks_nothing() {
    let mut config = two_installs();
    config.last_install = Some("gog:pool-of-radiance".into());

    let (result, printed) = choose(&config, Some("pool-of-radiance"), Some('J'), "");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
    assert_eq!(printed, "", "no question was printed");
}

#[test]
fn a_named_slot_that_is_empty_errors_with_the_populated_list() {
    let mut config = two_installs();
    config.last_install = Some("gog:pool-of-radiance".into());

    // One Enter answers the install question; the slot was named.
    let (result, _) = choose(&config, None, Some('B'), "\n");

    let err = result.unwrap_err();
    assert!(err.contains('B'), "got: {err}");
    assert!(err.contains('A') && err.contains('J'), "got: {err}");
}

#[test]
fn b_on_the_slot_question_goes_back_to_the_install_question() {
    let config = two_installs();

    // Pick install 2 (slot C only), back out, pick install 1, slot J.
    let (result, printed) = choose(&config, None, None, "2\nb\n1\nJ\n");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
    let games_asked = printed.matches("Which game").count();
    assert_eq!(games_asked, 2, "the install question was asked again: {printed}");
}

#[test]
fn nonsense_input_re_asks_the_question() {
    let config = two_installs();

    let (result, _) = choose(&config, None, None, "banana\n1\nQ\nJ\n");

    assert_eq!(result.unwrap(), ("gog:pool-of-radiance".into(), 'J'));
}

#[test]
fn a_lowercase_slot_letter_works() {
    let config = two_installs();

    let (result, _) = choose(&config, None, None, "1\nj\n");

    assert_eq!(result.unwrap().1, 'J');
}

#[test]
fn an_unknown_game_argument_names_the_compiled_in_games() {
    let config = two_installs();

    let (result, _) = choose(&config, Some("wizardry"), None, "");

    let err = result.unwrap_err();
    assert!(err.contains("wizardry"), "got: {err}");
    assert!(err.contains("pool-of-radiance"), "got: {err}");
}

// --- helpers -----------------------------------------------------------------

fn install(kind: InstallKind, saves_dir: &Path) -> Install {
    Install {
        game: "pool-of-radiance".into(),
        kind,
        root: saves_dir.to_string_lossy().into_owned(),
        saves: String::new(),
        confs: vec!["por.conf".into()],
        emulator: None,
        introduced: false,
    }
}

fn tempdir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-wizard-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

fn write_save(dir: &Path, file: &str, name: &str) {
    let mut bytes = vec![0u8; 285];
    bytes[0] = name.len() as u8;
    bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
    std::fs::write(dir.join(file), bytes).unwrap();
}

// --- repicking the slot mid-watch (ticket 020) -------------------------------

#[test]
fn repicking_returns_the_new_slot_and_its_names() {
    let dir = tempdir("repick");
    write_save(&dir, "CHRDATA1.SAV", "ALPHA");
    write_save(&dir, "CHRDATJ1.SAV", "JULIET");

    let mut output = Vec::new();
    let picked = wizard::repick_slot(
        &mut Cursor::new(b"J\n".to_vec()),
        &mut output,
        &dir,
    )
    .unwrap();

    assert_eq!(picked, Some(('J', vec!["JULIET".to_string()])));
}

#[test]
fn backing_out_of_a_repick_keeps_the_current_slot() {
    let dir = tempdir("repick-back");
    write_save(&dir, "CHRDATA1.SAV", "ALPHA");

    let mut output = Vec::new();
    let picked = wizard::repick_slot(
        &mut Cursor::new(b"b\n".to_vec()),
        &mut output,
        &dir,
    )
    .unwrap();

    assert_eq!(picked, None, "b means keep watching the slot already chosen");
}
