//! The keyboard contract, with no terminal anywhere.
//!
//! Ticket 040. Every key is a change to what the HUD knows, and what the HUD
//! knows is a `View`. Testing it here rather than through a pseudo-terminal is
//! why the state was split out of the terminal in the first place.

use crossterm::event::{KeyCode, KeyModifiers};

use squire_cli::hud::{Press, View};
use squire_cli::layout::{Caption, Size};
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

fn caption() -> Caption {
    Caption {
        game: "Pool of Radiance".into(),
        slot: Some('J'),
        panel: "party".into(),
        note: None,
    }
}

fn party() -> Party {
    Party {
        state: PartyState::Live,
        characters: (0..6).map(|i| who(&format!("NUMBER {i}"))).collect(),
    }
}

fn who(name: &str) -> Character {
    Character {
        name: name.to_string(),
        race: Some("human".into()),
        race_raw: 0,
        class: Some("fighter".into()),
        class_raw: 0,
        gender: Some("male".into()),
        alignment: Some("lawful good".into()),
        status: Some("okay".into()),
        status_raw: 0,
        level: 5,
        hit_points_current: 20,
        hit_points_maximum: 30,
        armor_class: 4,
        thac0: 16,
        experience: 12_000,
        age: 30,
        strength: 18,
        strength_exceptional: 0,
        intelligence: 9,
        wisdom: 11,
        dexterity: 16,
        constitution: 17,
        charisma: 10,
    }
}

/// A view with the party already found.
fn watching() -> View {
    let mut view = View::new(caption());
    view.saw(&party());
    view
}

fn press(view: &mut View, code: KeyCode) -> Press {
    view.press(code, KeyModifiers::NONE)
}

fn roomy(view: &View) -> squire_cli::layout::Plan {
    view.plan(Size {
        cols: 160,
        rows: 42,
    })
}

// --- Quitting -------------------------------------------------------------

#[test]
fn three_documented_keys_quit() {
    for code in [KeyCode::Char('q'), KeyCode::Esc] {
        assert_eq!(press(&mut watching(), code), Press::Quit, "{code:?}");
    }
    assert_eq!(
        watching().press(KeyCode::Char('c'), KeyModifiers::CONTROL),
        Press::Quit
    );
}

#[test]
fn a_key_nobody_bound_does_nothing() {
    let before = watching();
    let mut after = watching();
    assert_eq!(press(&mut after, KeyCode::Char('z')), Press::Handled);
    assert_eq!(before, after);
}

// --- The toggles ----------------------------------------------------------

#[test]
fn a_turns_the_ability_scores_on_and_off_again() {
    use squire_cli::layout::CardLine;
    let has = |view: &View| {
        roomy(view).grid.unwrap().cards[0]
            .lines
            .iter()
            .any(|l| matches!(l, CardLine::Abilities))
    };
    let mut view = watching();
    assert!(!has(&view), "they start off");
    press(&mut view, KeyCode::Char('a'));
    assert!(has(&view));
    press(&mut view, KeyCode::Char('a'));
    assert!(!has(&view));
}

#[test]
fn c_flips_between_horizontal_and_vertical_and_back() {
    let mut view = watching();
    let horizontal = roomy(&view).grid.unwrap().across;

    press(&mut view, KeyCode::Char('c'));
    let vertical = roomy(&view).grid.unwrap().across;
    assert_ne!(vertical, horizontal, "vertical drew the same grid");

    press(&mut view, KeyCode::Char('c'));
    assert_eq!(roomy(&view).grid.unwrap().across, horizontal);
}

// --- The panels -----------------------------------------------------------

#[test]
fn the_number_keys_are_reserved_and_one_has_a_screen() {
    let mut view = watching();
    assert!(roomy(&view).status.contains("1 party"));
    // Nothing behind the other numbers yet, and pressing one is not an error.
    for digit in ['2', '5', '9'] {
        assert_eq!(press(&mut view, KeyCode::Char(digit)), Press::Handled);
        assert!(roomy(&view).status.contains("1 party"), "{digit} moved it");
    }
    assert_eq!(press(&mut view, KeyCode::Char('1')), Press::Handled);
    assert!(roomy(&view).status.contains("1 party"));
}

// --- The slot repick ------------------------------------------------------

#[test]
fn s_asks_for_a_slot_and_enter_does_nothing() {
    // Enter is the key you press to find out what a key does. Going back to
    // the wizard is not a thing to discover by accident.
    let mut view = watching();
    assert_eq!(press(&mut view, KeyCode::Enter), Press::Handled);
    assert_eq!(press(&mut view, KeyCode::Char('s')), Press::AskForSlot);
    assert_eq!(view.party().characters.len(), 6, "nothing changed yet");
}

#[test]
fn the_arrow_keys_are_unbound() {
    // The highlight went with the selector. When there is something to select
    // a character for, both come back together.
    let before = watching();
    let mut after = watching();
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Char('j'),
        KeyCode::Char('k'),
    ] {
        assert_eq!(press(&mut after, code), Press::Handled, "{code:?}");
    }
    assert_eq!(before, after);
}

#[test]
fn retargeting_clears_the_party_the_old_slot_left_behind() {
    let mut view = watching();
    view.retargeted('B');

    assert!(view.party().characters.is_empty(), "the old party survived");
    let plan = roomy(&view);
    assert!(plan.header.contains('B'), "{:?}", plan.header);
    assert!(!plan.dim, "an empty new slot is waiting, not lost");
    assert!(plan.grid.is_none());
}

// --- What the loop says ---------------------------------------------------

#[test]
fn a_found_party_takes_the_loops_words_off_the_status_line() {
    let mut view = View::new(caption());
    view.note("waiting for the party of save slot J to load...");
    assert!(roomy(&view).status.contains("waiting for"));
    view.saw(&party());
    assert!(!roomy(&view).status.contains("waiting for"));
}

#[test]
fn losing_and_recovering_the_anchor_dims_and_undims_the_same_party() {
    // Ticket 038's "restores full brightness with no flicker": the recovered
    // frame is the frame from before it was lost, character for character.
    let mut view = watching();
    let live = roomy(&view);
    assert!(!live.dim);

    view.saw(&Party {
        state: PartyState::NotFound,
        characters: Vec::new(),
    });
    let lost = roomy(&view);
    assert!(lost.dim);
    assert_eq!(lost.grid, live.grid, "the numbers moved while dimmed");

    view.saw(&party());
    let back = roomy(&view);
    assert!(!back.dim);
    assert_eq!(
        back, live,
        "the recovered frame is not the frame from before"
    );
}

#[test]
fn the_status_line_says_which_axis_is_active() {
    let mut view = watching();
    assert!(
        roomy(&view).status.contains("horizontal"),
        "{:?}",
        roomy(&view).status
    );

    press(&mut view, KeyCode::Char('c'));
    assert!(
        roomy(&view).status.contains("vertical"),
        "{:?}",
        roomy(&view).status
    );

    press(&mut view, KeyCode::Char('c'));
    assert!(
        roomy(&view).status.contains("horizontal"),
        "{:?}",
        roomy(&view).status
    );
}
