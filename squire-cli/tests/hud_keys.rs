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

// --- The highlight --------------------------------------------------------

#[test]
fn the_highlight_moves_and_stops_at_both_ends() {
    let mut view = watching();
    assert_eq!(view.selected(), 0);
    for _ in 0..20 {
        press(&mut view, KeyCode::Down);
    }
    assert_eq!(view.selected(), 5, "it ran off the bottom");
    for _ in 0..20 {
        press(&mut view, KeyCode::Up);
    }
    assert_eq!(view.selected(), 0, "it ran off the top");
}

#[test]
fn the_vi_keys_do_the_same_thing_as_the_arrows() {
    let mut arrows = watching();
    let mut letters = watching();
    press(&mut arrows, KeyCode::Down);
    press(&mut letters, KeyCode::Char('j'));
    assert_eq!(arrows.selected(), letters.selected());
    press(&mut arrows, KeyCode::Up);
    press(&mut letters, KeyCode::Char('k'));
    assert_eq!(arrows.selected(), letters.selected());
}

#[test]
fn a_party_that_shrank_leaves_the_highlight_somewhere_real() {
    let mut view = watching();
    for _ in 0..5 {
        press(&mut view, KeyCode::Down);
    }
    let mut fewer = party();
    fewer.characters.truncate(2);
    fewer.state = PartyState::Partial;
    view.saw(&fewer);
    assert_eq!(view.selected(), 1);
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
fn c_steps_through_the_arrangements_and_back_to_the_rule() {
    // Six characters divide evenly into one, two, three and six across, and
    // the key walks exactly that list before handing the choice back. What
    // the rule itself picks depends on the party's own names, so it is read
    // rather than assumed.
    let mut view = watching();
    let rule = roomy(&view).grid.unwrap().across;

    let mut seen = Vec::new();
    for _ in 0..5 {
        press(&mut view, KeyCode::Char('c'));
        seen.push(roomy(&view).grid.unwrap().across);
    }
    assert_eq!(seen, vec![1, 2, 3, 6, rule], "{seen:?}");
}

#[test]
fn the_key_never_asks_for_an_arrangement_the_rule_would_not_offer() {
    // Five characters have no three-across arrangement without a ragged row.
    let mut five = party();
    five.characters.truncate(5);
    let mut view = View::new(caption());
    view.saw(&five);

    let mut seen = Vec::new();
    for _ in 0..3 {
        press(&mut view, KeyCode::Char('c'));
        seen.push(roomy(&view).grid.unwrap().across);
    }
    assert_eq!(&seen[..2], &[1, 5], "{seen:?}");
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
fn enter_asks_for_a_slot_rather_than_deciding_one() {
    // The wizard owns the question. All the view does is say it was asked.
    let mut view = watching();
    assert_eq!(press(&mut view, KeyCode::Enter), Press::AskForSlot);
    assert_eq!(view.party().characters.len(), 6, "nothing changed yet");
}

#[test]
fn retargeting_clears_the_party_the_old_slot_left_behind() {
    let mut view = watching();
    for _ in 0..4 {
        press(&mut view, KeyCode::Down);
    }
    view.retargeted('B');

    assert!(view.party().characters.is_empty(), "the old party survived");
    assert_eq!(view.selected(), 0);
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
