//! A view: the process that draws in a window of its own.
//!
//! What is tested here is everything below the terminal. A view is a socket
//! turned into draw calls, and the turning is what can be wrong.

use std::path::{Path, PathBuf};

use squire_cli::args::Args;
use squire_cli::terminals::ViewKind;
use squire_cli::view::{self, Incoming};
use squire_cli::wire::{self, Hello};
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

fn parse(argv: &[&str]) -> Result<Args, String> {
    Args::parse(argv.iter().map(|s| s.to_string())).map_err(|e| e.to_string())
}

fn character(name: &str) -> Character {
    Character {
        name: name.to_string(),
        race: Some("elf".into()),
        race_raw: 3,
        class: Some("mage".into()),
        class_raw: 5,
        gender: Some("female".into()),
        alignment: Some("neutral good".into()),
        status: Some("okay".into()),
        status_raw: 0,
        level: 4,
        hit_points_current: 11,
        hit_points_maximum: 14,
        armor_class: 8,
        thac0: 19,
        experience: 9000,
        age: 84,
        strength: 9,
        strength_exceptional: 0,
        intelligence: 17,
        wisdom: 11,
        dexterity: 16,
        constitution: 10,
        charisma: 13,
    }
}

// --- how a view is asked for ----------------------------------------------

#[test]
fn a_view_is_asked_for_by_kind_and_socket() {
    let a = parse(&["--view", "hud", "--socket", "/run/user/1000/x.sock"]).unwrap();

    assert_eq!(a.view, Some(ViewKind::Hud));
    assert_eq!(a.socket.as_deref(), Some("/run/user/1000/x.sock"));
}

#[test]
fn a_view_kind_squire_does_not_have_is_a_named_error() {
    let err = parse(&["--view", "journal", "--socket", "/x.sock"]).unwrap_err();

    assert!(err.contains("journal"), "got: {err}");
    assert!(err.contains("hud"), "the error lists what there is: {err}");
}

#[test]
fn a_view_without_a_socket_is_an_error_rather_than_a_window_showing_nothing() {
    let err = parse(&["--view", "hud"]).unwrap_err();

    assert!(err.contains("--socket"), "got: {err}");
}

#[test]
fn a_socket_without_a_view_is_an_error_too() {
    let err = parse(&["--socket", "/x.sock"]).unwrap_err();

    assert!(err.contains("--view"), "got: {err}");
}

#[test]
fn a_bare_gbs_is_not_a_view() {
    let a = parse(&[]).unwrap();

    assert_eq!(a.view, None);
    assert_eq!(a.socket, None);
}

// --- what a view makes of what it is sent ---------------------------------

#[test]
fn the_hello_tells_the_view_which_run_it_is_drawing() {
    let sent = Hello {
        game_id: "curse-of-the-azure-bonds".into(),
        game_name: "Curse of the Azure Bonds".into(),
        slot: Some('B'),
        save_dir: Some(PathBuf::from("/games/curse")),
    };

    let Incoming::Hello(got) = view::decode(&sent.line()).unwrap() else {
        panic!("a hello line did not decode as a hello");
    };

    assert_eq!(got, sent);
}

#[test]
fn a_run_with_no_slot_yet_still_has_a_hello() {
    // A fresh install has no save, so the host has no slot to name until the
    // user saves in game and repicks.
    let sent = Hello {
        game_id: "pool-of-radiance".into(),
        game_name: "Pool of Radiance".into(),
        slot: None,
        save_dir: None,
    };

    let Incoming::Hello(got) = view::decode(&sent.line()).unwrap() else {
        panic!("a hello line did not decode as a hello");
    };

    assert_eq!(got.slot, None);
    assert_eq!(got.save_dir, None);
}

#[test]
fn a_party_line_decodes_to_the_party_the_host_read() {
    let party = Party {
        state: PartyState::Live,
        characters: vec![character("Ilyana")],
    };
    let line =
        serde_json::json!({ "kind": "party", "party": wire::party_value(&party) }).to_string();

    let Incoming::Party(got) = view::decode(&line).unwrap() else {
        panic!("a party line did not decode as a party");
    };

    assert_eq!(got, party);
}

#[test]
fn a_notice_line_decodes_to_the_words_the_watch_said() {
    let line = serde_json::json!({ "kind": "notice", "message": "the anchor moved" }).to_string();

    let Incoming::Notice(message) = view::decode(&line).unwrap() else {
        panic!("a notice line did not decode as a notice");
    };

    assert_eq!(message, "the anchor moved");
}

#[test]
fn a_kind_this_build_has_never_heard_of_is_skipped_and_not_fatal() {
    // A newer host talking to an older view. Ignoring what it cannot draw is
    // what lets the wire grow a message without every view being rebuilt.
    let line = serde_json::json!({ "kind": "weather", "raining": true }).to_string();

    assert!(matches!(view::decode(&line), Ok(Incoming::Unknown)));
}

#[test]
fn a_blank_line_is_skipped() {
    assert!(matches!(view::decode("   "), Ok(Incoming::Unknown)));
}

#[test]
fn a_party_line_that_is_malformed_says_what_was_wrong() {
    let line = serde_json::json!({ "kind": "party", "party": { "state": "molten" } }).to_string();

    let err = view::decode(&line).unwrap_err();

    assert!(err.contains("molten"), "got: {err}");
}

// --- connecting ------------------------------------------------------------

#[test]
fn a_socket_that_is_not_there_says_so_and_names_the_path() {
    let missing = Path::new("/nonexistent/goldbox-squire/999999.sock");

    let err = view::connect(missing).unwrap_err();

    assert!(err.contains("999999.sock"), "got: {err}");
}
