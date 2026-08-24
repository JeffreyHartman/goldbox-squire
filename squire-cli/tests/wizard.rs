//! The wizard: game, directory, slot — answerable in advance, back is 0.
//!
//! No terminal is involved: input is plain lines in, prompts are lines out.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use squire_cli::config::{Config, InstallKind};
use squire_cli::wizard;

fn choose(
    input: &str,
    config: &mut Config,
    game: Option<&str>,
    game_dir: Option<&str>,
    slot: Option<char>,
) -> (Result<(String, char), String>, String) {
    let mut output = Vec::new();
    let result = wizard::choose(
        &mut Cursor::new(input.as_bytes()),
        &mut output,
        config,
        game,
        game_dir,
        slot,
    );
    (result, String::from_utf8(output).unwrap())
}

/// A config with two discovered directories of Pool of Radiance, unchosen.
fn two_dirs(tag: &str) -> (Config, PathBuf, PathBuf) {
    let gog = saves_dir(&format!("{tag}-gog"), &["CHRDATA1.SAV"]);
    let steam = saves_dir(&format!("{tag}-steam"), &["CHRDATA1.SAV", "CHRDATB1.SAV"]);
    let mut config = Config::default();
    for (key, kind, dir) in [
        ("gog:pool-of-radiance", InstallKind::Gog, &gog),
        ("steam:pool-of-radiance", InstallKind::Steam, &steam),
    ] {
        config.installs.insert(
            key.into(),
            squire_cli::config::Install {
                game: "pool-of-radiance".into(),
                kind,
                root: dir.to_string_lossy().into_owned(),
                saves: String::new(),
            },
        );
    }
    (config, gog, steam)
}

fn chosen(config: &mut Config, key: &str) {
    config
        .chosen
        .insert("pool-of-radiance".into(), key.into());
    config.last_game = Some("pool-of-radiance".into());
}

// --- the flow ------------------------------------------------------------------

#[test]
fn a_first_run_asks_game_directory_and_slot() {
    let (mut config, gog, _) = two_dirs("first");

    let (result, output) = choose("1\n1\nA\n", &mut config, None, None, None);

    let (key, slot) = result.unwrap();
    assert_eq!(key, "gog:pool-of-radiance");
    assert_eq!(slot, 'A');
    assert!(output.contains("Which game?"), "got: {output}");
    assert!(output.contains("Where is"), "got: {output}");
    assert!(output.contains("Which save slot?"), "got: {output}");
    // The pick is remembered.
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("gog:pool-of-radiance")
    );
    assert_eq!(gog, gog); // silence the unused binding lint plainly
}

#[test]
fn a_returning_user_is_two_enters_from_a_running_game() {
    let (mut config, _, _) = two_dirs("return");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\n\n", &mut config, None, None, None);

    let (key, slot) = result.unwrap();
    assert_eq!(key, "gog:pool-of-radiance");
    assert_eq!(slot, 'A');
    assert!(
        !output.contains("Where is"),
        "a chosen directory skips the question: {output}"
    );
}

#[test]
fn the_game_menu_lists_every_compiled_in_game_even_without_a_directory() {
    let mut config = Config::default();

    // The run fails later (no directory typed), but the menu must show.
    let (_, output) = choose("1\n0\n0\n", &mut config, None, None, None);

    assert!(output.contains("Pool of Radiance"), "got: {output}");
}

#[test]
fn a_typed_path_becomes_the_chosen_manual_directory() {
    let dir = saves_dir("typed", &["CHRDATA1.SAV"]);
    let mut config = Config::default();

    let input = format!("1\n1\n{}\n\n", dir.display());
    let (result, output) = choose(&input, &mut config, None, None, None);

    let (key, _) = result.unwrap();
    assert_eq!(key, "manual:pool-of-radiance");
    assert_eq!(config.installs[&key].kind, InstallKind::Manual);
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("manual:pool-of-radiance")
    );
    assert!(output.contains("type a path"), "got: {output}");
}

#[test]
fn a_bad_typed_path_is_explained_and_asked_again() {
    let empty = saves_dir("typed-bad-empty", &[]);
    let good = saves_dir("typed-bad-good", &["CHRDATA1.SAV"]);
    let mut config = Config::default();

    let input = format!("1\n1\n{}\n{}\n\n", empty.display(), good.display());
    let (result, output) = choose(&input, &mut config, None, None, None);

    assert!(result.is_ok(), "got: {result:?}");
    assert!(output.contains("CHRDAT"), "explains what was missing: {output}");
}

#[test]
fn game_dir_re_points_the_game_even_when_one_was_chosen() {
    let (mut config, _, _) = two_dirs("repoint");
    chosen(&mut config, "gog:pool-of-radiance");
    let new_dir = saves_dir("repoint-new", &["CHRDATJ1.SAV"]);

    let (result, _) = choose(
        "\n\n",
        &mut config,
        None,
        Some(new_dir.to_str().unwrap()),
        None,
    );

    let (key, slot) = result.unwrap();
    assert_eq!(key, "manual:pool-of-radiance");
    assert_eq!(slot, 'J');
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("manual:pool-of-radiance")
    );
}

// --- arguments answer questions --------------------------------------------------

#[test]
fn naming_the_game_skips_the_game_question() {
    let (mut config, _, _) = two_dirs("game-arg");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\n", &mut config, Some("pool-of-radiance"), None, None);

    assert!(result.is_ok());
    assert!(!output.contains("Which game?"), "got: {output}");
}

#[test]
fn naming_the_slot_skips_the_slot_question() {
    let (mut config, _, _) = two_dirs("slot-arg");
    chosen(&mut config, "steam:pool-of-radiance");

    let (result, output) = choose("\n", &mut config, None, None, Some('B'));

    assert_eq!(result.unwrap().1, 'B');
    assert!(!output.contains("Which save slot?"), "got: {output}");
}

#[test]
fn naming_everything_asks_nothing() {
    let (mut config, _, _) = two_dirs("all-args");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("", &mut config, Some("pool-of-radiance"), None, Some('A'));

    assert!(result.is_ok());
    assert_eq!(output, "");
}

#[test]
fn a_named_slot_that_is_empty_errors_with_the_populated_list() {
    let (mut config, _, _) = two_dirs("empty-slot");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, _) = choose("\n", &mut config, None, None, Some('J'));

    let err = result.unwrap_err();
    assert!(err.contains('J') && err.contains('A'), "got: {err}");
}

#[test]
fn an_unknown_game_argument_names_the_compiled_in_games() {
    let mut config = Config::default();

    let (result, _) = choose("", &mut config, Some("wizardry"), None, None);

    let err = result.unwrap_err();
    assert!(err.contains("pool-of-radiance"), "got: {err}");
}

// --- the keys ---------------------------------------------------------------------

#[test]
fn slot_b_is_picked_by_typing_b() {
    // `b` used to mean back, which made save slot B unpickable by letter.
    let (mut config, _, _) = two_dirs("slot-b");
    chosen(&mut config, "steam:pool-of-radiance");

    let (result, _) = choose("\nb\n", &mut config, None, None, None);

    assert_eq!(result.unwrap().1, 'B');
}

#[test]
fn zero_on_the_slot_question_goes_back_to_the_game_question() {
    let (mut config, _, _) = two_dirs("zero-back");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\n0\n\n\n", &mut config, None, None, None);

    assert!(result.is_ok());
    let asked = output.matches("Which game?").count();
    assert_eq!(asked, 2, "got: {output}");
}

#[test]
fn a_lowercase_slot_letter_works() {
    let (mut config, _, _) = two_dirs("lower");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, _) = choose("\na\n", &mut config, None, None, None);

    assert_eq!(result.unwrap().1, 'A');
}

#[test]
fn nonsense_input_re_asks_the_question() {
    let (mut config, _, _) = two_dirs("nonsense");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\nQ9\nA\n", &mut config, None, None, None);

    assert_eq!(result.unwrap().1, 'A');
    assert!(output.contains("Pick one of"), "got: {output}");
}

// --- repicking mid-watch -----------------------------------------------------------

#[test]
fn repicking_returns_the_new_slot_and_its_names() {
    let dir = saves_dir("repick", &["CHRDATA1.SAV", "CHRDATJ1.SAV"]);
    let mut output = Vec::new();

    let picked = wizard::repick_slot(
        &mut Cursor::new(b"J\n".as_slice()),
        &mut output,
        &dir,
    )
    .unwrap();

    let (slot, names) = picked.expect("a slot was picked");
    assert_eq!(slot, 'J');
    assert_eq!(names, vec!["JULIET"]);
}

#[test]
fn backing_out_of_a_repick_keeps_the_current_slot() {
    let dir = saves_dir("repick-back", &["CHRDATA1.SAV"]);
    let mut output = Vec::new();

    let picked = wizard::repick_slot(
        &mut Cursor::new(b"0\n".as_slice()),
        &mut output,
        &dir,
    )
    .unwrap();

    assert!(picked.is_none());
}

// --- helpers ------------------------------------------------------------------------

fn saves_dir(tag: &str, files: &[&str]) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gbs-wizard-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    for file in files {
        let name = if file.contains('J') {
            "JULIET"
        } else if file.contains('B') {
            "BRAVO"
        } else {
            "ALPHA"
        };
        write_save(&base, file, name);
    }
    base
}

fn write_save(dir: &Path, file: &str, name: &str) {
    let mut bytes = vec![0u8; 285];
    bytes[0] = name.len() as u8;
    bytes[1..1 + name.len()].copy_from_slice(name.as_bytes());
    std::fs::write(dir.join(file), bytes).unwrap();
}
