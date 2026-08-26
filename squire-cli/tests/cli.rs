//! The command line contract: what the arguments mean, and what is printed.

use squire_cli::args::Args;
use squire_cli::output;
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

fn parse(argv: &[&str]) -> Result<Args, String> {
    Args::parse(argv.iter().map(|s| s.to_string())).map_err(|e| e.to_string())
}

// --- arguments -------------------------------------------------------------

#[test]
fn design_names_the_adventure_in_advance() {
    let a = parse(&["--design", "BASILISK"]).unwrap();
    assert_eq!(a.design.as_deref(), Some("BASILISK"));
}

#[test]
fn game_dir_sets_the_game_folder() {
    let a = parse(&["--game-dir", "/games/poolrad"]).unwrap();

    assert_eq!(a.game_dir.as_deref(), Some("/games/poolrad"));
}

#[test]
fn json_selects_machine_readable_output() {
    assert!(!parse(&[]).unwrap().json);
    assert!(parse(&["--json"]).unwrap().json);
}

#[test]
fn watch_is_not_a_flag_because_watching_is_what_the_tool_does() {
    let err = parse(&["--watch"]).unwrap_err();

    assert!(err.contains("--watch"), "got: {err}");
    assert!(
        err.contains("--help"),
        "the error points at the usage: {err}"
    );
}

#[test]
fn the_game_can_be_named_in_advance() {
    let a = parse(&["--game", "pool-of-radiance"]).unwrap();

    assert_eq!(a.game.as_deref(), Some("pool-of-radiance"));
}

#[test]
fn the_save_slot_can_be_named_in_advance() {
    let a = parse(&["--slot", "J"]).unwrap();

    assert_eq!(a.slot, Some('J'));
}

#[test]
fn a_lowercase_slot_letter_is_accepted() {
    assert_eq!(parse(&["--slot", "j"]).unwrap().slot, Some('J'));
}

#[test]
fn a_slot_that_is_not_a_letter_a_through_j_is_rejected() {
    let err = parse(&["--slot", "K"]).unwrap_err();
    assert!(err.contains('K'), "got: {err}");

    let err = parse(&["--slot", "AB"]).unwrap_err();
    assert!(err.contains("AB"), "got: {err}");
}

#[test]
fn the_emulator_command_can_be_set() {
    let a = parse(&["--dosbox", "dosbox-staging"]).unwrap();

    assert_eq!(a.dosbox.as_deref(), Some("dosbox-staging"));
}

#[test]
fn attach_reads_an_emulator_this_tool_did_not_start() {
    let a = parse(&["--pid", "1234"]).unwrap();

    assert_eq!(a.pid, Some(1234));
}

#[test]
fn an_unknown_flag_is_rejected_and_named() {
    let err = parse(&["--frobnicate"]).unwrap_err();

    assert!(err.contains("--frobnicate"), "got: {err}");
}

#[test]
fn a_flag_that_needs_a_value_and_has_none_is_rejected() {
    let err = parse(&["--game-dir"]).unwrap_err();

    assert!(err.contains("--game-dir"), "got: {err}");
}

#[test]
fn a_pid_that_is_not_a_number_is_rejected() {
    let err = parse(&["--pid", "banana"]).unwrap_err();

    assert!(err.contains("banana"), "got: {err}");
}

#[test]
fn help_is_asked_for_and_not_an_error() {
    assert!(parse(&["--help"]).unwrap().help);
    assert!(parse(&["-h"]).unwrap().help);
    assert!(!parse(&[]).unwrap().help);
}

// --- output ----------------------------------------------------------------

fn character(name: &str, cur: i16, max: u8) -> Character {
    Character {
        name: name.to_string(),
        race: Some("dwarf".into()),
        race_raw: 1,
        class: Some("fighter".into()),
        class_raw: 2,
        gender: Some("male".into()),
        alignment: Some("lawful good".into()),
        status: Some("okay".into()),
        status_raw: 0,
        level: 1,
        hit_points_current: cur,
        hit_points_maximum: max,
        armor_class: 4,
        thac0: 20,
        experience: 32,
        age: 52,
        strength: 17,
        strength_exceptional: 0,
        intelligence: 12,
        wisdom: 12,
        dexterity: 17,
        constitution: 16,
        charisma: 15,
    }
}

fn party(state: PartyState, characters: Vec<Character>) -> Party {
    Party { state, characters }
}

#[test]
fn the_table_shows_a_row_for_each_character() {
    let p = party(
        PartyState::Live,
        vec![
            character("THRENDER GRONE", 11, 11),
            character("BAKSHI", 3, 7),
        ],
    );

    let text = output::table(&p);

    assert!(text.contains("THRENDER GRONE"));
    assert!(text.contains("BAKSHI"));
    assert!(text.contains("11/11"));
    assert!(text.contains("3/7"));
}

#[test]
fn the_table_marks_a_character_who_is_not_okay() {
    let mut dying = character("PHINEAS", -3, 6);
    dying.status = Some("dying".into());
    let p = party(PartyState::Live, vec![dying]);

    let text = output::table(&p);

    assert!(text.contains("dying"), "the status is shown: {text}");
    assert!(
        text.contains("-3/6"),
        "negative hit points are shown as such"
    );
}

#[test]
fn the_table_says_so_when_no_party_is_in_memory() {
    let p = party(PartyState::NotFound, vec![]);

    let text = output::table(&p);

    assert!(text.to_lowercase().contains("no party"), "got: {text}");
}

#[test]
fn the_table_says_so_when_only_part_of_the_party_is_visible() {
    let p = party(PartyState::Partial, vec![character("BAKSHI", 7, 7)]);

    let text = output::table(&p);

    assert!(text.to_lowercase().contains("partial"), "got: {text}");
}

#[test]
fn the_json_holds_the_fields_a_front_end_needs() {
    let p = party(PartyState::Live, vec![character("BAKSHI", 3, 7)]);

    let json: serde_json::Value = serde_json::from_str(&output::json(&p)).unwrap();

    assert_eq!(json["state"], "live");
    assert_eq!(json["characters"][0]["name"], "BAKSHI");
    assert_eq!(json["characters"][0]["hit_points"]["current"], 3);
    assert_eq!(json["characters"][0]["hit_points"]["maximum"], 7);
    assert_eq!(json["characters"][0]["class"], "fighter");
    assert_eq!(json["characters"][0]["status"], "okay");
}

#[test]
fn the_json_reports_an_unknown_value_as_null_rather_than_inventing_one() {
    let mut c = character("MYSTERY", 5, 5);
    c.class = None;
    c.class_raw = 0x7F;
    let p = party(PartyState::Live, vec![c]);

    let json: serde_json::Value = serde_json::from_str(&output::json(&p)).unwrap();

    assert!(json["characters"][0]["class"].is_null());
    assert_eq!(json["characters"][0]["class_raw"], 0x7F);
}

#[test]
fn the_json_is_valid_when_there_is_no_party() {
    let p = party(PartyState::NotFound, vec![]);

    let json: serde_json::Value = serde_json::from_str(&output::json(&p)).unwrap();

    assert_eq!(json["state"], "not_found");
    assert_eq!(json["characters"].as_array().unwrap().len(), 0);
}

#[test]
fn a_long_name_does_not_break_the_table_alignment() {
    let p = party(
        PartyState::Live,
        vec![character("A", 1, 1), character("THRENDER GRONE!", 11, 11)],
    );

    let text = output::table(&p);
    let rows: Vec<&str> = text.lines().filter(|l| l.contains('|')).collect();
    let widths: Vec<usize> = rows.iter().map(|r| r.chars().count()).collect();

    assert!(
        widths.windows(2).all(|w| w[0] == w[1]),
        "every row is the same width: {widths:?}"
    );
}

// The printed screen: the loop hands it a party, it decides how that looks.

#[test]
fn the_printed_screen_clears_before_it_redraws_the_table() {
    let p = party(PartyState::Live, vec![character("BAKSHI", 3, 7)]);
    let mut out = Vec::new();
    let mut err = Vec::new();
    {
        let mut screen = output::Plain::new(&mut out, &mut err, false);
        squire_cli::watch::Screen::party(&mut screen, &p);
    }

    let text = String::from_utf8(out).unwrap();
    assert!(text.starts_with("\x1b[2J\x1b[H"), "got: {text:?}");
    assert!(text.contains("BAKSHI"));
    assert!(err.is_empty(), "the table goes to standard output alone");
}

#[test]
fn the_json_screen_does_not_clear_the_screen_under_a_pipe() {
    let p = party(PartyState::Live, vec![character("BAKSHI", 3, 7)]);
    let mut out = Vec::new();
    let mut err = Vec::new();
    {
        let mut screen = output::Plain::new(&mut out, &mut err, true);
        squire_cli::watch::Screen::party(&mut screen, &p);
    }

    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.contains("\x1b"),
        "an escape sequence would corrupt the JSON"
    );
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["characters"][0]["name"], "BAKSHI");
}

#[test]
fn a_notice_is_named_and_kept_off_the_party_stream() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    {
        let mut screen = output::Plain::new(&mut out, &mut err, false);
        squire_cli::watch::Screen::notice(&mut screen, "the emulator ended. Until next time.");
    }

    assert_eq!(
        String::from_utf8(err).unwrap(),
        "gbs: the emulator ended. Until next time.\n"
    );
    assert!(out.is_empty());
}

/// A writer that has gone away, the way a closed pipe does.
struct ClosedPipe;

impl std::io::Write for ClosedPipe {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
#[should_panic(expected = "failed printing the party")]
fn a_closed_pipe_ends_the_run_rather_than_watching_forever_in_silence() {
    let p = party(PartyState::Live, vec![character("BAKSHI", 3, 7)]);
    let mut err = Vec::new();
    let mut screen = output::Plain::new(ClosedPipe, &mut err, false);

    squire_cli::watch::Screen::party(&mut screen, &p);
}

#[test]
fn plain_is_the_escape_from_the_hud_and_the_hud_needs_no_flag() {
    // There is no --tui. An argument required to make the program work is
    // not an argument, which is the same reasoning that killed --watch.
    assert!(!parse(&[]).unwrap().plain);
    assert!(parse(&["--plain"]).unwrap().plain);
    assert!(!parse(&["--plain"]).unwrap().json);
}

#[test]
fn the_usage_says_where_the_window_size_is_remembered() {
    // A remembered thing the user cannot find is a hidden behaviour.
    assert!(squire_cli::args::USAGE.contains("[hud]"));
    assert!(squire_cli::args::USAGE.contains("--plain"));
    assert!(squire_cli::args::USAGE.contains("q quits"));
}
