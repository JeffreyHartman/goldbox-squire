//! The layout plan: rows, columns and a party in, what is shown out.
//!
//! Ticket 036. Every decision this effort made about what a HUD shows is
//! asserted here, at sizes no real terminal would produce, with no terminal
//! anywhere in the test. The drop order stops being something an agent
//! remembers and becomes something the build checks.
//!
//! The sizes named below come from ticket 034's mockups. They are the sizes
//! Jeff looked at and answered for, so the answers are pinned here. No
//! constant in `layout.rs` may name any of them.

use squire_cli::layout::{self, Axis, Caption, CardLine, Liveness, Plan, Size, Toggles};
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// The party from the mockups, so that the recorded answers can be checked
/// against the rule that is supposed to produce them.
fn party() -> Party {
    Party {
        state: PartyState::Live,
        characters: vec![
            who("THRENDER GRONE", "fighter", 5, 42, 42, 2, "okay"),
            who("BROTHER SEAN", "cleric", 4, 26, 26, 4, "okay"),
            who("AMRYL", "mage", 4, 14, 14, 8, "okay"),
            who("KEIRA", "fighter/thief", 5, 18, 31, 5, "okay"),
            who("DURIN STONEFOOT", "fighter", 5, 38, 44, 1, "poisoned"),
            who("ELANNA", "cleric/mage", 3, 20, 22, 6, "okay"),
        ],
    }
}

fn who(
    name: &str,
    class: &str,
    level: u8,
    hp: i16,
    hp_max: u8,
    ac: i16,
    status: &str,
) -> Character {
    Character {
        name: name.to_string(),
        race: Some("human".into()),
        race_raw: 0,
        class: Some(class.to_string()),
        class_raw: 0,
        gender: Some("male".into()),
        alignment: Some("lawful good".into()),
        status: Some(status.to_string()),
        status_raw: 0,
        level,
        hit_points_current: hp,
        hit_points_maximum: hp_max,
        armor_class: ac,
        thac0: 16,
        experience: 12_000,
        age: 30,
        strength: 18,
        strength_exceptional: 72,
        intelligence: 9,
        wisdom: 11,
        dexterity: 16,
        constitution: 17,
        charisma: 10,
    }
}

fn caption() -> Caption {
    Caption {
        game: "Pool of Radiance".into(),
        slot: Some('J'),
        panel: "party".into(),
        note: None,
    }
}

fn at(cols: u16, rows: u16) -> Plan {
    layout::plan(
        Size { cols, rows },
        &party(),
        &caption(),
        Toggles::default(),
    )
}

/// Every line the plan would put on screen, already fitted to the width.
/// Nothing here draws; it renders what the plan already decided.
fn card_text(plan: &Plan, party: &Party, card: usize) -> Vec<String> {
    let grid = plan
        .grid
        .as_ref()
        .expect("a grid was expected at this size");
    let width = grid.widths[card % grid.across as usize];
    grid.cards[card]
        .lines
        .iter()
        .map(|line| layout::line_text(&party.characters[card], line, width))
        .collect()
}

fn has_abilities(card: &layout::Card) -> bool {
    card.lines.iter().any(|l| matches!(l, CardLine::Abilities))
}

// --- The number of cards across -------------------------------------------

#[test]
fn horizontal_never_narrows_as_the_terminal_widens() {
    // Horizontal is the default. A wider terminal never seats fewer cards
    // across, and the widest terminal tried reaches one row of the whole
    // party.
    let party = party();
    let mut last = 0u16;
    for cols in (20..=200).step_by(10) {
        let plan = layout::plan(
            Size { cols, rows: 40 },
            &party,
            &caption(),
            Toggles {
                axis: Axis::Horizontal,
                ..Toggles::default()
            },
        );
        if let Some(grid) = plan.grid {
            assert!(
                grid.across >= last,
                "{cols} across dropped to {}",
                grid.across
            );
            last = grid.across;
        }
    }
    assert_eq!(
        last, 6,
        "the widest terminal tried did not reach one row of six"
    );
}

#[test]
fn the_key_flips_between_horizontal_and_vertical() {
    // Six characters at 60x40: horizontal packs a wide, short room; vertical
    // packs a single tall column instead.
    let size = Size { cols: 60, rows: 40 };
    let horizontal = layout::plan(
        size,
        &party(),
        &caption(),
        Toggles {
            axis: Axis::Horizontal,
            ..Toggles::default()
        },
    );
    let vertical = layout::plan(
        size,
        &party(),
        &caption(),
        Toggles {
            axis: Axis::Vertical,
            ..Toggles::default()
        },
    );
    let h = horizontal.grid.expect("horizontal fits at 60x40");
    let v = vertical.grid.expect("vertical fits at 60x40");
    assert!(h.across >= h.down, "{h:?} is not the wider arrangement");
    assert!(v.down >= v.across, "{v:?} is not the taller arrangement");
}

#[test]
fn the_cards_never_reach_past_the_right_edge() {
    for cols in [16u16, 24, 40, 61, 80, 137, 160, 400] {
        let plan = at(cols, 40);
        let Some(grid) = plan.grid.as_ref() else {
            continue;
        };
        // One cell of frame per card, plus a closing one, plus two of padding
        // inside each card.
        let used: u16 = grid.widths.iter().map(|w| w + 3).sum::<u16>() + 1;
        assert_eq!(used, cols, "{cols} columns drew {used}");
    }
}

// --- The drop order -------------------------------------------------------

#[test]
fn a_wide_card_holds_the_whole_character() {
    // Vertical, so the card gets the whole width rather than being split
    // into as many columns as fit.
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &party(),
        &caption(),
        Toggles {
            axis: Axis::Vertical,
            ..Toggles::default()
        },
    );
    let card = &plan.grid.as_ref().unwrap().cards[4];
    assert!(matches!(card.lines[0], CardLine::Name { class_here: true }));
    assert!(matches!(
        card.lines[1],
        CardLine::HitPoints {
            armor_here: true,
            bar
        } if bar > 0
    ));
    assert!(matches!(card.lines[2], CardLine::Condition(0)));
    assert_eq!(card.lines.len(), 3, "the toggle is off, so no ability line");
}

#[test]
fn fields_leave_in_the_settled_order_as_the_card_narrows() {
    // Last to go first: name, hit points, conditions, class, armour class.
    //
    // Vertical pins the party to one column so that the only thing changing
    // is how many lines a card may have. Letting a row re-choose the number
    // across would widen the cards as the screen got shorter, which is real
    // behaviour and the wrong thing to measure here.
    let mut seen: Vec<Vec<&'static str>> = Vec::new();
    for rows in [39u16, 33, 27, 21] {
        let plan = layout::plan(
            Size { cols: 19, rows },
            &party(),
            &caption(),
            Toggles {
                axis: Axis::Vertical,
                ..Toggles::default()
            },
        );
        let grid = plan
            .grid
            .as_ref()
            .unwrap_or_else(|| panic!("19x{rows} drew nothing"));
        seen.push(grid.cards[4].lines.iter().map(kind).collect());
    }

    assert_eq!(
        seen,
        vec![
            vec!["name", "hp", "condition", "class", "armor"],
            vec!["name", "hp", "condition", "class"],
            vec!["name", "hp", "condition"],
            vec!["name", "hp"],
        ]
    );
}

fn kind(line: &CardLine) -> &'static str {
    match line {
        CardLine::Name { .. } => "name",
        CardLine::HitPoints { .. } => "hp",
        CardLine::Condition(_) => "condition",
        CardLine::Class => "class",
        CardLine::Armor => "armor",
        CardLine::Abilities => "abilities",
    }
}

#[test]
fn armour_class_leaves_before_the_condition_does() {
    // Armour class is first out of the drop order and a condition is not,
    // because a silent `poisoned` is the thing you most want to notice
    // without looking away from the game.
    let plan = layout::plan(
        Size { cols: 19, rows: 27 },
        &party(),
        &caption(),
        Toggles {
            axis: Axis::Vertical,
            ..Toggles::default()
        },
    );
    let lines: Vec<&str> = plan.grid.as_ref().unwrap().cards[4]
        .lines
        .iter()
        .map(kind)
        .collect();
    assert!(lines.contains(&"condition"), "{lines:?}");
    assert!(!lines.contains(&"armor"), "{lines:?}");
}

#[test]
fn every_card_in_one_party_is_shaped_the_same() {
    // A card laid out differently because its owner has a short name reads
    // as a bug. The shape comes from the longest name and class in the party.
    let plan = at(80, 40);
    let grid = plan.grid.as_ref().unwrap();
    let shape = |card: &layout::Card| -> Vec<std::mem::Discriminant<CardLine>> {
        card.lines.iter().map(std::mem::discriminant).collect()
    };
    let first = shape(&grid.cards[0]);
    for (i, card) in grid.cards.iter().enumerate() {
        assert_eq!(shape(card), first, "card {i} is shaped differently");
    }
}

// --- Ability scores are a toggle, not a width rule -------------------------

#[test]
fn ability_scores_are_off_unless_the_user_asks() {
    let plan = at(160, 42);
    for card in &plan.grid.as_ref().unwrap().cards {
        assert!(!has_abilities(card));
    }
}

#[test]
fn the_toggle_adds_the_ability_line_and_moves_nothing_else() {
    let size = Size {
        cols: 110,
        rows: 50,
    };
    let vertical = Toggles {
        axis: Axis::Vertical,
        ..Toggles::default()
    };
    let off = layout::plan(size, &party(), &caption(), vertical);
    let on = layout::plan(
        size,
        &party(),
        &caption(),
        Toggles {
            abilities: true,
            ..vertical
        },
    );
    let (off_grid, on_grid) = (off.grid.unwrap(), on.grid.unwrap());
    assert_eq!(off_grid.across, on_grid.across);
    assert_eq!(off_grid.widths, on_grid.widths);
    for (a, b) in off_grid.cards.iter().zip(&on_grid.cards) {
        assert!(!has_abilities(a));
        assert!(has_abilities(b));
        assert_eq!(b.lines.len(), a.lines.len() + 1);
    }
}

#[test]
fn no_card_ever_shows_one_ability_score() {
    // There is no half measure: all six or none. A card too narrow for the
    // line simply does not get it.
    let party = party();
    for cols in [16u16, 20, 30, 44, 60, 110] {
        let plan = layout::plan(
            Size { cols, rows: 40 },
            &party,
            &caption(),
            Toggles {
                abilities: true,
                ..Toggles::default()
            },
        );
        let Some(grid) = plan.grid.as_ref() else {
            continue;
        };
        for (i, card) in grid.cards.iter().enumerate() {
            if !has_abilities(card) {
                continue;
            }
            let width = grid.widths[i % grid.across as usize];
            let text = layout::line_text(&party.characters[i], &CardLine::Abilities, width);
            assert_eq!(
                text.trim_end().matches('/').count(),
                5,
                "{cols} columns cut the ability line: {text:?}"
            );
        }
    }
}

// --- The wordmark ---------------------------------------------------------

#[test]
fn the_wordmark_appears_only_when_roomy() {
    // Roomy is room to spare after the party, not a size. A tall sidecar has
    // spare rows and gets one; a short wide strip does not.
    for (cols, rows) in [(160, 42), (50, 40), (110, 50)] {
        assert!(at(cols, rows).wordmark, "{cols}x{rows} lost its wordmark");
    }
    for (cols, rows) in [(40, 20), (160, 14), (39, 60)] {
        assert!(
            !at(cols, rows).wordmark,
            "{cols}x{rows} has no room and drew a wordmark"
        );
    }
}

#[test]
fn a_wordmark_never_reaches_past_the_edge() {
    let block = layout::wordmark();
    assert_eq!(block.len(), layout::WORDMARK_ROWS as usize);
    let widest = block.iter().map(|l| l.chars().count()).max().unwrap();
    for cols in [40u16, 60, 160, 400] {
        let plan = at(cols, 60);
        if plan.wordmark {
            assert!(widest <= cols as usize, "{cols} columns cannot hold it");
        }
    }
}

// --- Stale numbers, for ticket 038 ----------------------------------------

#[test]
fn a_lost_anchor_dims_the_party_and_says_why() {
    let mut party = party();
    party.state = PartyState::NotFound;
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &party,
        &caption(),
        Toggles::default(),
    );
    assert!(plan.dim, "a lost anchor must dim");
    assert_eq!(plan.liveness, Liveness::Lost);
    assert!(
        plan.status.contains("lost"),
        "the status line must say why: {:?}",
        plan.status
    );
    assert!(
        plan.grid.is_some(),
        "the last known numbers stay on screen while dimmed"
    );
}

#[test]
fn a_partial_party_is_not_a_lost_one() {
    let mut party = party();
    party.state = PartyState::Partial;
    party.characters.truncate(4);
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &party,
        &caption(),
        Toggles::default(),
    );
    assert_eq!(plan.liveness, Liveness::Partial);
    assert!(!plan.dim, "a partial party's numbers are live");
    assert!(plan.status.contains("partial"), "{:?}", plan.status);
    assert_eq!(plan.grid.unwrap().cards.len(), 4);
}

#[test]
fn a_live_party_is_neither_dim_nor_apologetic() {
    let plan = at(110, 50);
    assert!(!plan.dim);
    assert_eq!(plan.liveness, Liveness::Live);
    assert!(plan.status.contains("live"), "{:?}", plan.status);
}

#[test]
fn the_reason_survives_at_a_hostile_size() {
    // Where the status line is the only thing that fits, it still says it.
    let mut party = party();
    party.state = PartyState::NotFound;
    let plan = layout::plan(
        Size { cols: 24, rows: 3 },
        &party,
        &caption(),
        Toggles::default(),
    );
    assert!(plan.dim);
    assert!(plan.status.contains("lost"), "{:?}", plan.status);
}

// --- Sizes nobody sensible would produce ----------------------------------

#[test]
fn a_hostile_size_draws_something_and_stays_inside_it() {
    let party = party();
    let plan = at(40, 20);
    let grid = plan.grid.as_ref().expect("40x20 must still draw cards");
    assert!(grid.card_rows >= 2);
    for i in 0..grid.cards.len() {
        for line in card_text(&plan, &party, i) {
            let width = grid.widths[i % grid.across as usize] as usize;
            assert_eq!(line.chars().count(), width, "{line:?} is not {width} wide");
        }
    }
    assert!(plan.header.chars().count() <= 40);
    assert!(plan.status.chars().count() <= 40);
}

#[test]
fn an_enormous_size_does_not_overflow_anything() {
    let plan = at(1000, 400);
    let grid = plan.grid.as_ref().expect("everything fits at 1000x400");
    let used: u16 = grid.widths.iter().map(|w| w + 3).sum::<u16>() + 1;
    assert_eq!(used, 1000);
    // The header, the cards, their frames, the spare room and the status
    // line must all fit inside the rows.
    let card_rows = grid.down * (grid.card_rows + 1) + 1;
    assert!(card_rows + 2 <= 400, "{card_rows} card rows in 400");
}

#[test]
fn a_size_with_no_room_for_a_card_still_says_what_is_happening() {
    for (cols, rows) in [(1u16, 1u16), (4, 2), (10, 3), (80, 2)] {
        let plan = at(cols, rows);
        assert!(
            plan.grid.is_none(),
            "{cols}x{rows} drew cards it cannot hold"
        );
        assert!(plan.status.chars().count() <= cols as usize);
        assert!(plan.header.chars().count() <= cols as usize);
    }
}

#[test]
fn an_empty_party_is_not_a_panic() {
    let empty = Party {
        state: PartyState::NotFound,
        characters: Vec::new(),
    };
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &empty,
        &caption(),
        Toggles::default(),
    );
    assert!(plan.grid.is_none());
    assert!(!plan.status.is_empty());
}

#[test]
fn a_party_of_one_gets_a_card() {
    let mut one = party();
    one.characters.truncate(1);
    let plan = layout::plan(
        Size { cols: 60, rows: 40 },
        &one,
        &caption(),
        Toggles::default(),
    );
    let grid = plan.grid.expect("one character still draws");
    assert_eq!((grid.across, grid.down), (1, 1));
    assert_eq!(grid.cards.len(), 1);
}

#[test]
fn an_uneven_party_leaves_a_short_last_row_in_horizontal() {
    // Seven characters, forced to five across: a full row of five, then a
    // short row of two. A ragged last row is not an error to route around.
    let mut seven = party();
    seven
        .characters
        .push(who("SISTER MORA", "cleric", 2, 12, 16, 7, "okay"));
    let plan = layout::plan(
        Size { cols: 60, rows: 42 },
        &seven,
        &caption(),
        Toggles::default(),
    );
    let grid = plan.grid.expect("seven characters lay out somehow");
    assert_eq!(grid.cards.len(), 7);
    assert!(
        grid.across * grid.down > 7,
        "expected a ragged grid, got {} across, {} down",
        grid.across,
        grid.down
    );
}

// --- The header and the status line ---------------------------------------

#[test]
fn the_header_names_the_game_and_the_slot() {
    let plan = at(110, 50);
    assert!(
        plan.header.contains("Pool of Radiance"),
        "{:?}",
        plan.header
    );
    assert!(plan.header.contains('J'), "{:?}", plan.header);
}

#[test]
fn the_status_line_lists_the_keys_when_there_is_room() {
    let plan = at(160, 42);
    assert!(plan.status.contains('q'), "{:?}", plan.status);
    assert!(
        plan.status.contains("party"),
        "the panel and its number: {:?}",
        plan.status
    );
}

#[test]
fn the_watch_loops_own_words_reach_the_status_line() {
    let mut caption = caption();
    caption.note = Some("waiting for the party of save slot J to load...".into());
    let plan = layout::plan(
        Size {
            cols: 160,
            rows: 42,
        },
        &party(),
        &caption,
        Toggles::default(),
    );
    assert!(plan.status.contains("waiting"), "{:?}", plan.status);
}

#[test]
fn a_run_with_no_slot_yet_does_not_invent_one() {
    let mut caption = caption();
    caption.slot = None;
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &party(),
        &caption,
        Toggles::default(),
    );
    assert!(!plan.header.contains("slot"), "{:?}", plan.header);
}

#[test]
fn nothing_found_yet_is_not_a_lost_anchor() {
    // Before the first party is read there is nothing to dim and nothing to
    // apologise for. The loop's own words carry the screen.
    let empty = Party {
        state: PartyState::NotFound,
        characters: Vec::new(),
    };
    let mut caption = caption();
    caption.note = Some("waiting for the party of save slot J to load...".into());
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &empty,
        &caption,
        Toggles::default(),
    );
    assert_eq!(plan.liveness, Liveness::Waiting);
    assert!(!plan.dim);
    assert!(!plan.status.contains("lost"), "{:?}", plan.status);
    assert!(plan.status.contains("waiting"), "{:?}", plan.status);
}
