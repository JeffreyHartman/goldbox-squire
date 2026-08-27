//! The host: one process reads the emulator and hands the party out.
//!
//! Every test here runs with no terminal and no emulator. The host is a
//! `Screen` and a `Keys`, so a test drives it exactly as the watch loop does
//! and reads the other end of the socket by hand.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use squire_cli::host::{self, Hello, Host};
use squire_cli::watch::{Interrupt, Keys, Screen};
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// A socket path of this test's own, in a directory the test owns.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gbs-host-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("making the test directory");
    dir.join("run.sock")
}

fn hello() -> Hello {
    Hello {
        game_id: "pool-of-radiance".into(),
        game_name: "Pool of Radiance".into(),
        slot: Some('A'),
        save_dir: Some(PathBuf::from("/games/poolrad")),
    }
}

fn started(path: &Path) -> Host {
    Host::start(path, hello(), Box::new(Vec::new())).expect("the host starts")
}

fn character(name: &str) -> Character {
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
        level: 3,
        hit_points_current: 18,
        hit_points_maximum: 22,
        armor_class: 4,
        thac0: 18,
        experience: 3200,
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

/// Reads one line, giving up rather than hanging if the host says nothing.
fn line(reader: &mut BufReader<UnixStream>) -> String {
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("setting a read timeout");
    let mut buf = String::new();
    reader.read_line(&mut buf).expect("reading from the host");
    assert!(!buf.is_empty(), "the host closed instead of answering");
    buf
}

/// Connects, and lets the host notice, which it does inside its own pause.
fn connect(host: &mut Host, path: &Path) -> BufReader<UnixStream> {
    let stream = UnixStream::connect(path).expect("connecting to the host");
    let mut keys = host.keys();
    keys.wait(Duration::from_millis(50)).expect("the pause");
    BufReader::new(stream)
}

#[test]
fn the_socket_is_there_while_the_run_is_and_gone_after_it() {
    let path = scratch("lifetime");

    let host = started(&path);
    assert!(
        path.exists(),
        "the host is listening but there is no socket"
    );

    drop(host);
    assert!(!path.exists(), "the socket outlived the run");
}

#[test]
fn a_view_is_told_what_it_needs_before_any_party_arrives() {
    // A view has to caption itself and to run a repick, and neither can wait
    // for the next poll: a game sitting at the title screen polls every two
    // seconds and may never have found a party at all.
    let path = scratch("hello");
    let mut host = started(&path);

    let mut view = connect(&mut host, &path);
    let first = line(&mut view);

    let value: serde_json::Value = serde_json::from_str(&first).expect("valid JSON");
    assert_eq!(value["kind"], "hello");
    assert_eq!(value["game_id"], "pool-of-radiance");
    assert_eq!(value["game_name"], "Pool of Radiance");
    assert_eq!(value["slot"], "A");
    assert_eq!(value["save_dir"], "/games/poolrad");
}

#[test]
fn a_view_receives_the_party_as_it_changes() {
    let path = scratch("party");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);
    line(&mut view);

    let mut screen = host.screen();
    screen.party(&Party {
        state: PartyState::Live,
        characters: vec![character("Ilyana")],
    });

    let value: serde_json::Value = serde_json::from_str(&line(&mut view)).expect("valid JSON");
    assert_eq!(value["kind"], "party");
    assert_eq!(value["party"]["state"], "live");
    assert_eq!(value["party"]["characters"][0]["name"], "Ilyana");
}

#[test]
fn a_view_receives_the_watchs_notices() {
    let path = scratch("notice");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);
    line(&mut view);

    host.screen().notice("the emulator ended. Until next time.");

    let value: serde_json::Value = serde_json::from_str(&line(&mut view)).expect("valid JSON");
    assert_eq!(value["kind"], "notice");
    assert_eq!(value["message"], "the emulator ended. Until next time.");
}

#[test]
fn a_view_that_arrives_late_is_caught_up_at_once() {
    // Opening the map an hour into a sitting must not show an empty window
    // until the next poll, and the poll it would wait for is half a second at
    // best and two seconds while nothing has been found.
    let path = scratch("late");
    let mut host = started(&path);

    host.screen().party(&Party {
        state: PartyState::Live,
        characters: vec![character("Ilyana")],
    });
    host.screen()
        .notice("waiting for the party of save slot A to load...");

    let mut view = connect(&mut host, &path);

    let kinds: Vec<String> = (0..3)
        .map(|_| {
            let v: serde_json::Value = serde_json::from_str(&line(&mut view)).expect("valid JSON");
            v["kind"].as_str().expect("a kind").to_string()
        })
        .collect();
    assert_eq!(kinds, ["hello", "party", "notice"]);
}

#[test]
fn a_view_that_comes_and_goes_does_not_disturb_the_run() {
    // Views are throwaway. Closing the HUD to reopen it at a different size is
    // a thing a person does, and the game plays on in between.
    let path = scratch("reconnect");
    let mut host = started(&path);

    let view = connect(&mut host, &path);
    drop(view);

    let mut keys = host.keys();
    assert_eq!(
        keys.wait(Duration::from_millis(50)).expect("the pause"),
        Interrupt::None,
        "a view closing is not an interrupt"
    );
    drop(keys);

    let mut second = connect(&mut host, &path);
    assert!(line(&mut second).contains("hello"));

    host.screen().party(&Party {
        state: PartyState::Live,
        characters: vec![character("Ilyana")],
    });
    assert!(line(&mut second).contains("Ilyana"));
}

#[test]
fn the_party_still_goes_out_when_nothing_is_connected() {
    // Nobody is listening between the run starting and the window opening, and
    // that is not a failure.
    let path = scratch("nobody");
    let host = started(&path);

    host.screen().party(&Party {
        state: PartyState::NotFound,
        characters: Vec::new(),
    });
    host.screen().notice("still here");
}

#[test]
fn a_view_asking_to_quit_reaches_the_loop() {
    let path = scratch("quit");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);
    line(&mut view);

    view.get_mut()
        .write_all(b"{\"kind\":\"quit\"}\n")
        .expect("sending quit");

    let mut keys = host.keys();
    assert_eq!(
        keys.wait(Duration::from_millis(500)).expect("the pause"),
        Interrupt::Quit
    );
}

#[test]
fn a_view_repicking_reaches_the_loop_with_the_slot_and_the_names() {
    // The view asks the slot question, because the user is looking at the
    // view's window. What crosses the socket is the resolved answer.
    let path = scratch("repick");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);
    line(&mut view);

    view.get_mut()
        .write_all(b"{\"kind\":\"repick\",\"slot\":\"C\",\"names\":[\"Ilyana\",\"Brom\"]}\n")
        .expect("sending a repick");

    let mut keys = host.keys();
    assert_eq!(
        keys.wait(Duration::from_millis(500)).expect("the pause"),
        Interrupt::Repick {
            slot: 'C',
            names: vec!["Ilyana".to_string(), "Brom".to_string()],
        }
    );
}

#[test]
fn a_pause_with_nobody_typing_spends_the_whole_interval() {
    // The pause is also the cadence. Returning at once would turn a 500ms
    // poll into a spin through the emulator's whole memory.
    let path = scratch("cadence");
    let host = started(&path);
    let mut keys = host.keys();

    let started_at = std::time::Instant::now();
    assert_eq!(
        keys.wait(Duration::from_millis(200)).expect("the pause"),
        Interrupt::None
    );

    assert!(
        started_at.elapsed() >= Duration::from_millis(180),
        "the pause returned after {:?}",
        started_at.elapsed()
    );
}

#[test]
fn a_line_a_view_should_never_send_is_ignored_rather_than_fatal() {
    // A view is another program. Killing a sitting because one sent nonsense
    // would be the tool taking the game down with it.
    let path = scratch("nonsense");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);
    line(&mut view);

    view.get_mut()
        .write_all(b"not json at all\n{\"kind\":\"unheard-of\"}\n")
        .expect("sending nonsense");

    let mut keys = host.keys();
    assert_eq!(
        keys.wait(Duration::from_millis(200)).expect("the pause"),
        Interrupt::None
    );
}

#[test]
fn the_socket_is_one_per_run_under_the_runtime_directory() {
    // Per run, not per user, so that two games at once is not a special case.
    // The runtime directory is cleared at logout, so a killed host leaves
    // nothing behind for the next one to trip over.
    let path = host::socket_path(Some(Path::new("/run/user/1000")), 4321);

    assert_eq!(
        path,
        Path::new("/run/user/1000/goldbox-squire/4321.sock"),
        "got {}",
        path.display()
    );
}

#[test]
fn a_machine_with_no_runtime_directory_still_gets_a_socket() {
    let path = host::socket_path(None, 4321);

    assert!(
        path.ends_with("goldbox-squire/4321.sock"),
        "got {}",
        path.display()
    );
    assert!(path.is_absolute(), "got {}", path.display());
}

#[test]
fn a_party_survives_the_round_trip_through_the_wire() {
    // A view redraws from this and nothing else, so a field lost in transit
    // is a field the HUD would draw as zero without saying so.
    let before = Party {
        state: PartyState::Partial,
        characters: vec![Character {
            name: "Brom".into(),
            race: None,
            race_raw: 9,
            class: None,
            class_raw: 200,
            gender: None,
            alignment: None,
            status: None,
            status_raw: 7,
            // A dying character holds a negative value, and a good armour
            // class is negative too, so both must cross unmangled.
            hit_points_current: -7,
            armor_class: -3,
            thac0: -1,
            experience: 4_000_000,
            ..character("Brom")
        }],
    };

    let after = squire_cli::wire::party_from_value(&squire_cli::wire::party_value(&before))
        .expect("a party Squire wrote is a party Squire can read");

    assert_eq!(after, before);
}

#[test]
fn a_party_that_is_not_one_says_what_is_wrong_rather_than_panicking() {
    let value = serde_json::json!({ "state": "sideways", "characters": [] });

    let err = squire_cli::wire::party_from_value(&value).unwrap_err();

    assert!(err.contains("sideways"), "got: {err}");
}

#[test]
fn a_view_that_has_stopped_reading_cannot_stall_the_run() {
    // A view stops reading whenever it steps aside for the slot question, and
    // the game is still being played while it does. A host that blocked on
    // one window would be a tool that freezes the party display because
    // somebody opened a menu in another window.
    let path = scratch("deaf");
    let mut host = started(&path);
    let _view = connect(&mut host, &path);

    let started_at = std::time::Instant::now();
    for _ in 0..2000 {
        host.screen().party(&Party {
            state: PartyState::Live,
            characters: vec![character("Ilyana")],
        });
    }

    assert!(
        started_at.elapsed() < Duration::from_secs(5),
        "2000 polls took {:?} with nobody reading",
        started_at.elapsed()
    );
}

#[test]
fn a_view_that_falls_behind_and_comes_back_gets_what_it_missed() {
    // What the slot question costs is a pause, not a gap in the numbers.
    let path = scratch("behind");
    let mut host = started(&path);
    let mut view = connect(&mut host, &path);

    host.screen().notice("first");
    host.screen().notice("second");

    let lines: Vec<String> = (0..3).map(|_| line(&mut view)).collect();
    assert!(lines[0].contains("hello"), "{lines:?}");
    assert!(lines[1].contains("first"), "{lines:?}");
    assert!(lines[2].contains("second"), "{lines:?}");
}
