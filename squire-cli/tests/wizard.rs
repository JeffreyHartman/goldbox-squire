//! The wizard: game, directory, slot — answerable in advance, back is 0.
//!
//! No terminal is involved: input is plain lines in, prompts are lines out.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use squire_cli::config::{Config, InstallKind};
use squire_cli::wizard;
use squire_core::games;

fn por() -> games::Game {
    games::find("pool-of-radiance").expect("Pool of Radiance is compiled in")
}

/// What choose() hands back: the install key and the optional sitting.
type Chosen = Result<(String, Option<wizard::Sitting>), String>;

fn choose(
    input: &str,
    config: &mut Config,
    game: Option<&str>,
    game_dir: Option<&str>,
    design: Option<&str>,
    slot: Option<char>,
) -> (Chosen, String) {
    let mut output = Vec::new();
    let result = wizard::choose(
        &mut Cursor::new(input.as_bytes()),
        &mut output,
        config,
        game,
        game_dir,
        design,
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
    config.chosen.insert("pool-of-radiance".into(), key.into());
    config.last_game = Some("pool-of-radiance".into());
}

// --- the flow ------------------------------------------------------------------

#[test]
fn a_first_run_asks_game_directory_and_slot() {
    let (mut config, gog, _) = two_dirs("first");

    let (result, output) = choose("1\n1\nA\n", &mut config, None, None, None, None);

    let (key, sitting) = result.unwrap();
    let sitting = sitting.expect("a saved game exists");
    assert_eq!(key, "gog:pool-of-radiance");
    assert_eq!(sitting.save_dir, gog);
    assert_eq!(sitting.slot, 'A');
    assert!(output.contains("Which game?"), "got: {output}");
    assert!(output.contains("Where is"), "got: {output}");
    assert!(output.contains("Which save slot?"), "got: {output}");
    // The pick is remembered.
    assert_eq!(
        config.chosen.get("pool-of-radiance").map(String::as_str),
        Some("gog:pool-of-radiance")
    );
}

#[test]
fn a_returning_user_is_two_enters_from_a_running_game() {
    let (mut config, _, _) = two_dirs("return");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\n\n", &mut config, None, None, None, None);

    let (key, sitting) = result.unwrap();
    assert_eq!(key, "gog:pool-of-radiance");
    assert_eq!(sitting.unwrap().slot, 'A');
    assert!(
        !output.contains("Where is"),
        "a chosen directory skips the question: {output}"
    );
}

#[test]
fn the_game_menu_lists_every_compiled_in_game_even_without_a_directory() {
    let mut config = Config::default();

    // The run fails later (no directory typed), but the menu must show.
    let (_, output) = choose("1\n0\n0\n", &mut config, None, None, None, None);

    assert!(output.contains("Pool of Radiance"), "got: {output}");
}

#[test]
fn a_typed_path_becomes_the_chosen_manual_directory() {
    let dir = saves_dir("typed", &["CHRDATA1.SAV"]);
    let mut config = Config::default();

    let input = format!("1\n1\n{}\n\n", dir.display());
    let (result, output) = choose(&input, &mut config, None, None, None, None);

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
    let (result, output) = choose(&input, &mut config, None, None, None, None);

    assert!(result.is_ok(), "got: {result:?}");
    assert!(
        output.contains("CHRDAT"),
        "explains what was missing: {output}"
    );
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
        None,
    );

    let (key, sitting) = result.unwrap();
    assert_eq!(key, "manual:pool-of-radiance");
    assert_eq!(sitting.unwrap().slot, 'J');
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

    let (result, output) = choose(
        "\n",
        &mut config,
        Some("pool-of-radiance"),
        None,
        None,
        None,
    );

    assert!(result.is_ok());
    assert!(!output.contains("Which game?"), "got: {output}");
}

#[test]
fn naming_the_slot_skips_the_slot_question() {
    let (mut config, _, _) = two_dirs("slot-arg");
    chosen(&mut config, "steam:pool-of-radiance");

    let (result, output) = choose("\n", &mut config, None, None, None, Some('B'));

    assert_eq!(result.unwrap().1.unwrap().slot, 'B');
    assert!(!output.contains("Which save slot?"), "got: {output}");
}

#[test]
fn naming_everything_asks_nothing() {
    let (mut config, _, _) = two_dirs("all-args");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose(
        "",
        &mut config,
        Some("pool-of-radiance"),
        None,
        None,
        Some('A'),
    );

    assert!(result.is_ok());
    assert_eq!(output, "");
}

#[test]
fn a_named_slot_that_is_empty_errors_with_the_populated_list() {
    let (mut config, _, _) = two_dirs("empty-slot");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, _) = choose("\n", &mut config, None, None, None, Some('J'));

    let err = result.unwrap_err();
    assert!(err.contains('J') && err.contains('A'), "got: {err}");
}

#[test]
fn an_unknown_game_argument_names_the_compiled_in_games() {
    let mut config = Config::default();

    let (result, _) = choose("", &mut config, Some("wizardry"), None, None, None);

    let err = result.unwrap_err();
    assert!(err.contains("pool-of-radiance"), "got: {err}");
}

// --- the keys ---------------------------------------------------------------------

#[test]
fn slot_b_is_picked_by_typing_b() {
    // `b` used to mean back, which made save slot B unpickable by letter.
    let (mut config, _, _) = two_dirs("slot-b");
    chosen(&mut config, "steam:pool-of-radiance");

    let (result, _) = choose("\nb\n", &mut config, None, None, None, None);

    assert_eq!(result.unwrap().1.unwrap().slot, 'B');
}

#[test]
fn zero_on_the_slot_question_goes_back_to_the_game_question() {
    let (mut config, _, _) = two_dirs("zero-back");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\n0\n\n\n", &mut config, None, None, None, None);

    assert!(result.is_ok());
    let asked = output.matches("Which game?").count();
    assert_eq!(asked, 2, "got: {output}");
}

#[test]
fn a_lowercase_slot_letter_works() {
    let (mut config, _, _) = two_dirs("lower");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, _) = choose("\na\n", &mut config, None, None, None, None);

    assert_eq!(result.unwrap().1.unwrap().slot, 'A');
}

#[test]
fn nonsense_input_re_asks_the_question() {
    let (mut config, _, _) = two_dirs("nonsense");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, output) = choose("\nQ9\nA\n", &mut config, None, None, None, None);

    assert_eq!(result.unwrap().1.unwrap().slot, 'A');
    assert!(output.contains("Pick one of"), "got: {output}");
}

// --- Unlimited Adventures: the design question ---------------------------------------

/// A FRUA game folder with two designs holding saved parties, chosen.
fn frua_config(tag: &str) -> (Config, PathBuf) {
    let root = saves_dir(&format!("{tag}-frua"), &[]);
    for design in ["BASILISK", "TUTORIAL"] {
        let save = root.join(format!("{design}.DSN/SAVE"));
        std::fs::create_dir_all(&save).unwrap();
        let mut bytes = vec![0u8; 1039];
        bytes[1037] = 1;
        bytes.extend_from_slice(&frua_record("HERO"));
        std::fs::write(save.join("SAVGAMA.CSV"), bytes).unwrap();
    }
    let mut config = Config::default();
    config.installs.insert(
        "manual:unlimited-adventures".into(),
        squire_cli::config::Install {
            game: "unlimited-adventures".into(),
            kind: InstallKind::Manual,
            root: root.to_string_lossy().into_owned(),
            saves: String::new(),
        },
    );
    config.chosen.insert(
        "unlimited-adventures".into(),
        "manual:unlimited-adventures".into(),
    );
    (config, root)
}

/// One record that passes validation against the Unlimited Adventures table.
fn frua_record(name: &str) -> Vec<u8> {
    let mut r = vec![0u8; 398];
    r[0x60..0x60 + name.len()].copy_from_slice(name.as_bytes());
    for offset in [0x71, 0x73, 0x75, 0x77, 0x79, 0x7B] {
        r[offset] = 12;
    }
    r[0x58] = 0x05; // human
    r[0x59] = 0x02; // fighter
    r[0x9F] = 1; // level_fighter
    r[0x81] = 9; // hit_points_maximum
    r[0x18B] = 9; // hit_points_current
    r
}

#[test]
fn a_designs_game_asks_which_adventure() {
    let (mut config, root) = frua_config("ask");

    let (result, output) = choose(
        "1\nA\n",
        &mut config,
        Some("unlimited-adventures"),
        None,
        None,
        None,
    );

    let (_, sitting) = result.unwrap();
    let sitting = sitting.expect("a saved game exists");
    assert!(output.contains("Which adventure?"), "got: {output}");
    assert_eq!(sitting.save_dir, root.join("BASILISK.DSN/SAVE"));
    assert_eq!(sitting.slot, 'A');
}

#[test]
fn naming_the_design_skips_the_adventure_question() {
    let (mut config, root) = frua_config("named");

    let (result, output) = choose(
        "",
        &mut config,
        Some("unlimited-adventures"),
        None,
        Some("tutorial"),
        Some('A'),
    );

    let (_, sitting) = result.unwrap();
    assert!(!output.contains("Which adventure?"), "got: {output}");
    assert_eq!(sitting.unwrap().save_dir, root.join("TUTORIAL.DSN/SAVE"));
}

#[test]
fn an_unknown_design_errors_naming_the_ones_with_saves() {
    let (mut config, _) = frua_config("unknown");

    let (result, _) = choose(
        "",
        &mut config,
        Some("unlimited-adventures"),
        None,
        Some("NOPE"),
        Some('A'),
    );

    let err = result.unwrap_err();
    assert!(
        err.contains("BASILISK") && err.contains("TUTORIAL"),
        "got: {err}"
    );
}

#[test]
fn design_on_a_game_without_designs_is_an_error() {
    let (mut config, _, _) = two_dirs("no-designs");
    chosen(&mut config, "gog:pool-of-radiance");

    let (result, _) = choose(
        "",
        &mut config,
        Some("pool-of-radiance"),
        None,
        Some("BASILISK"),
        Some('A'),
    );

    let err = result.unwrap_err();
    assert!(err.contains("design"), "got: {err}");
}

// --- the fresh-install escape --------------------------------------------------------

/// Overwrites both designs' save files with the zero-filled file a fresh
/// install ships: designs exist, no saved game does.
fn blank_the_designs(root: &Path) {
    for design in ["BASILISK", "TUTORIAL"] {
        std::fs::write(
            root.join(format!("{design}.DSN/SAVE/SAVGAMA.CSV")),
            vec![0u8; 10285],
        )
        .unwrap();
    }
}

#[test]
fn a_fresh_frua_install_offers_to_launch_anyway() {
    // A fresh install ships designs with zero-filled save files. gbs is the
    // launcher, so refusing to run would make the first save impossible.
    let (mut config, root) = frua_config("fresh");
    blank_the_designs(&root);

    let (result, output) = choose(
        "\n",
        &mut config,
        Some("unlimited-adventures"),
        None,
        None,
        None,
    );

    let (_, sitting) = result.unwrap();
    assert!(sitting.is_none(), "no saved game exists yet");
    assert!(output.contains("Start the game anyway?"), "got: {output}");
}

#[test]
fn zero_at_the_launch_anyway_question_goes_back() {
    let (mut config, root) = frua_config("fresh-back");
    blank_the_designs(&root);

    // Pick the game (12), answer 0 at launch-anyway; the input then ends,
    // which is an error, proving the wizard asked again rather than launching.
    let (result, output) = choose("12\n0\n", &mut config, None, None, None, None);

    assert!(result.is_err());
    assert!(output.contains("Start the game anyway?"), "got: {output}");
    assert_eq!(output.matches("Which game?").count(), 2, "got: {output}");
}

#[test]
fn a_fresh_chrdat_install_offers_to_launch_anyway_too() {
    let dir = saves_dir("fresh-chrdat", &[]);
    let mut config = Config::default();
    config.installs.insert(
        "manual:pool-of-radiance".into(),
        squire_cli::config::Install {
            game: "pool-of-radiance".into(),
            kind: InstallKind::Manual,
            root: dir.to_string_lossy().into_owned(),
            saves: String::new(),
        },
    );
    chosen(&mut config, "manual:pool-of-radiance");

    let (result, output) = choose(
        "\n",
        &mut config,
        Some("pool-of-radiance"),
        None,
        None,
        None,
    );

    let (_, sitting) = result.unwrap();
    assert!(sitting.is_none());
    assert!(output.contains("Start the game anyway?"), "got: {output}");
}

#[test]
fn an_explicit_slot_on_a_fresh_install_stays_a_hard_error() {
    // A script that named a slot expects that slot; launching without it
    // would silently do something else.
    let dir = saves_dir("fresh-slot-arg", &[]);
    let mut config = Config::default();
    config.installs.insert(
        "manual:pool-of-radiance".into(),
        squire_cli::config::Install {
            game: "pool-of-radiance".into(),
            kind: InstallKind::Manual,
            root: dir.to_string_lossy().into_owned(),
            saves: String::new(),
        },
    );
    chosen(&mut config, "manual:pool-of-radiance");

    let (result, _) = choose(
        "",
        &mut config,
        Some("pool-of-radiance"),
        None,
        None,
        Some('A'),
    );

    assert!(result.is_err());
}

#[test]
fn a_typed_path_to_a_fresh_game_folder_is_accepted_and_offers_to_launch() {
    // The folder holds the game (START.EXE) but no save yet. Refusing it
    // would make the first save impossible, since gbs is the launcher.
    let dir = saves_dir("typed-fresh", &[]);
    std::fs::write(dir.join("START.EXE"), b"MZ").unwrap();
    let mut config = Config::default();

    let (result, output) = choose(
        "\n",
        &mut config,
        Some("pool-of-radiance"),
        Some(dir.to_str().unwrap()),
        None,
        None,
    );

    let (key, sitting) = result.unwrap();
    assert_eq!(key, "manual:pool-of-radiance");
    assert!(sitting.is_none());
    assert!(output.contains("Start the game anyway?"), "got: {output}");
}

#[test]
fn a_repick_finds_the_first_save_written_into_a_child_folder() {
    // Launched fresh, the game wrote its first save into a SAVE child; the
    // mid-watch Enter must find it without a restart.
    let dir = saves_dir("repick-child", &[]);
    std::fs::create_dir_all(dir.join("SAVE")).unwrap();
    write_save(&dir.join("SAVE"), "CHRDATA1.SAV", "NEWBORN");
    let mut output = Vec::new();

    let picked = wizard::repick(
        &mut Cursor::new(b"A\n".as_slice()),
        &mut output,
        &por(),
        &dir,
    )
    .unwrap();

    let (slot, names) = picked.expect("the save was found");
    assert_eq!(slot, 'A');
    assert_eq!(names, vec!["NEWBORN"]);
}

// --- repicking mid-watch -----------------------------------------------------------

#[test]
fn repicking_a_designs_game_asks_the_design_then_the_slot() {
    let (_, root) = frua_config("repick-frua");
    let mut output = Vec::new();

    let picked = wizard::repick(
        &mut Cursor::new(b"2\nA\n".as_slice()),
        &mut output,
        &games::find("unlimited-adventures").unwrap(),
        &root,
    )
    .unwrap();

    let (slot, names) = picked.expect("a slot was picked");
    assert_eq!(slot, 'A');
    assert_eq!(names, vec!["HERO"]);
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Which adventure?"), "got: {text}");
}

#[test]
fn a_repick_with_no_saved_game_keeps_watching_instead_of_dying() {
    // Mid-watch the game is running; killing the watch over a failed repick
    // would take the session down with it.
    let dir = saves_dir("repick-none", &[]);
    let mut output = Vec::new();

    let picked =
        wizard::repick(&mut Cursor::new(b"".as_slice()), &mut output, &por(), &dir).unwrap();

    assert!(picked.is_none());
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("CHRDAT"), "the reason is printed: {text}");
}

#[test]
fn repicking_returns_the_new_slot_and_its_names() {
    let dir = saves_dir("repick", &["CHRDATA1.SAV", "CHRDATJ1.SAV"]);
    let mut output = Vec::new();

    let picked = wizard::repick(
        &mut Cursor::new(b"J\n".as_slice()),
        &mut output,
        &por(),
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

    let picked = wizard::repick(
        &mut Cursor::new(b"0\n".as_slice()),
        &mut output,
        &por(),
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
