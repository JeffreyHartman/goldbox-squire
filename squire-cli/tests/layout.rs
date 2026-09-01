//! Tests for `layout::plan`: given a terminal size and a party, does it pick
//! the right rows, columns, and dropped fields? Ticket 036.
//!
//! These tests run at sizes no real terminal produces, and none of them
//! opens a terminal.
//!
//! The sizes come from ticket 034's mockups.

use squire_cli::layout::{self, Axis, Caption, Field, Liveness, Plan, Size, Tint, Toggles};
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

fn has_abilities(card: &layout::Card) -> bool {
    card.lines.iter().any(|l| l.field == Field::Abilities)
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
    let grid = plan.grid.as_ref().unwrap();
    let card = &grid.cards[4];
    // At this width the class sits beside the name, and the armor class sits
    // beside the hit points. The shape is the grid's, so this holds for every
    // card in the party.
    assert!(grid.shape.class_inline);
    assert!(grid.shape.armor_inline);
    assert!(grid.shape.bar > 0);
    assert_eq!(card.lines[0].field, Field::Name);
    assert_eq!(card.lines[1].field, Field::HitPoints);
    assert_eq!(card.lines[2].field, Field::Condition(0));
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

fn kind(line: &layout::Line) -> &'static str {
    match line.field {
        Field::Name => "name",
        Field::HitPoints => "hp",
        Field::Condition(_) => "condition",
        Field::Class => "class",
        Field::Armor => "armor",
        Field::Abilities => "abilities",
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
    let shape = |card: &layout::Card| -> Vec<std::mem::Discriminant<Field>> {
        card.lines
            .iter()
            .map(|l| std::mem::discriminant(&l.field))
            .collect()
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
        for card in &grid.cards {
            let Some(line) = card.lines.iter().find(|l| l.field == Field::Abilities) else {
                continue;
            };
            assert_eq!(
                line.text.trim_end().matches('/').count(),
                5,
                "{cols} columns cut the ability line: {:?}",
                line.text
            );
        }
    }
}

// --- The logo -------------------------------------------------------------

#[test]
fn the_logo_appears_only_when_roomy() {
    // Roomy is room to spare after the party, not a size. A tall sidecar has
    // spare rows and gets one; a short wide strip does not.
    for (cols, rows) in [(160, 42), (50, 40), (110, 50)] {
        assert!(at(cols, rows).show_logo, "{cols}x{rows} lost its logo");
    }
    for (cols, rows) in [(40, 20), (160, 14), (39, 60)] {
        assert!(
            !at(cols, rows).show_logo,
            "{cols}x{rows} has no room and drew a logo"
        );
    }
}

#[test]
fn a_logo_never_reaches_past_the_edge() {
    let block = layout::logo();
    assert_eq!(block.len(), layout::LOGO_ROWS as usize);
    let widest = block.iter().map(|l| l.chars().count()).max().unwrap();
    for cols in [40u16, 60, 160, 400] {
        let plan = at(cols, 60);
        if plan.show_logo {
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
    let plan = at(40, 20);
    let grid = plan.grid.as_ref().expect("40x20 must still draw cards");
    assert!(grid.card_rows >= 2);
    for column in 0..grid.across {
        let card = &grid.cards[grid.who_at(0, column)];
        let width = usize::from(grid.widths[usize::from(column)]);
        for line in &card.lines {
            assert_eq!(
                line.text.chars().count(),
                width,
                "{:?} is not {width} wide",
                line.text
            );
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

// --- What a line means ------------------------------------------------

/// Every line of every card, with the cards run together.
fn lines_at(party: &Party, cols: u16, rows: u16) -> Vec<layout::Line> {
    layout::plan(Size { cols, rows }, party, &caption(), Toggles::default())
        .grid
        .expect("a grid was expected at this size")
        .cards
        .into_iter()
        .flat_map(|card| card.lines)
        .collect()
}

/// The tint of one character's hit point line.
fn hit_point_tint(hp: i16, hp_max: u8) -> Tint {
    let mut party = party();
    party.characters.truncate(1);
    party.characters[0].hit_points_current = hp;
    party.characters[0].hit_points_maximum = hp_max;
    lines_at(&party, 110, 50)
        .into_iter()
        .find(|l| l.field == Field::HitPoints)
        .expect("every card has a hit point line")
        .tint
}

#[test]
fn the_hit_point_bands_are_thirds() {
    assert_eq!(hit_point_tint(44, 44), Tint::Good);
    assert_eq!(hit_point_tint(30, 44), Tint::Good);
    assert_eq!(hit_point_tint(22, 44), Tint::Wounded);
    assert_eq!(hit_point_tint(15, 44), Tint::Wounded);
    assert_eq!(hit_point_tint(4, 44), Tint::Critical);
    assert_eq!(hit_point_tint(0, 44), Tint::Critical);
    assert_eq!(hit_point_tint(-3, 44), Tint::Critical);
}

#[test]
fn a_character_with_no_maximum_is_not_a_division_by_zero() {
    assert_eq!(hit_point_tint(0, 0), Tint::Critical);
}

#[test]
fn anything_that_is_not_okay_is_critical() {
    // A silent `poisoned` is the thing you most want to notice without
    // looking away from the game.
    let party = party();
    let tints: Vec<(String, Tint)> = lines_at(&party, 110, 50)
        .into_iter()
        .filter(|l| matches!(l.field, Field::Condition(_)))
        .map(|l| (l.text.trim_end().to_string(), l.tint))
        .collect();
    assert!(
        tints.contains(&("okay".to_string(), Tint::Good)),
        "{tints:?}"
    );
    assert!(
        tints.contains(&("poisoned".to_string(), Tint::Critical)),
        "{tints:?}"
    );
}

#[test]
fn a_name_is_a_heading_and_the_rest_is_body() {
    for line in lines_at(&party(), 110, 50) {
        let want = match line.field {
            Field::Name => Tint::Heading,
            Field::Class | Field::Armor | Field::Abilities => Tint::Body,
            _ => continue,
        };
        assert_eq!(line.tint, want, "{:?} of {:?}", line.field, line.text);
    }
}

#[test]
fn a_lost_anchor_faints_every_line_it_leaves_on_screen() {
    let mut lost = party();
    lost.state = PartyState::NotFound;
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &lost,
        &caption(),
        Toggles::default(),
    );
    assert!(plan.dim, "a lost anchor asks for the DIM attribute too");
    let lines: Vec<&layout::Line> = plan
        .grid
        .as_ref()
        .expect("the last known numbers stay on screen")
        .cards
        .iter()
        .flat_map(|card| &card.lines)
        .collect();
    assert!(!lines.is_empty());
    for line in lines {
        assert_eq!(line.tint, Tint::Faint, "{:?} kept a live colour", line.text);
    }
}

#[test]
fn the_status_lines_meaning_travels_with_the_plan() {
    assert_eq!(Liveness::Live.tint(), Tint::Body);
    assert_eq!(Liveness::Partial.tint(), Tint::Wounded);
    assert_eq!(Liveness::Lost.tint(), Tint::Critical);
    assert_eq!(Liveness::Waiting.tint(), Tint::Faint);
}

// --- The cards, word for word ---------------------------------------------

/// Sizes from ticket 034's mockups: hostile, the width the sidecar opens at,
/// tall, short and wide, and roomy.
const MOCKUP_SIZES: [(u16, u16); 5] = [(40, 20), (60, 40), (110, 50), (160, 14), (160, 42)];

#[test]
fn every_card_is_pinned_word_for_word() {
    let mut seen: Vec<String> = Vec::new();
    for (cols, rows) in MOCKUP_SIZES {
        for card in &at(cols, rows).grid.unwrap().cards {
            seen.extend(card.lines.iter().map(|l| l.text.clone()));
        }
    }
    let want = [
        // 40x20
        "THRENDER …",
        "hp 42/42  ",
        "okay      ",
        "fighter 5 ",
        "ac 2      ",
        "BROTHER S…",
        "hp 26/26  ",
        "okay      ",
        "cleric 4  ",
        "ac 4      ",
        "AMRYL     ",
        "hp 14/14  ",
        "okay      ",
        "mage 4    ",
        "ac 8      ",
        "KEIRA     ",
        "hp 18/31  ",
        "okay      ",
        "fighter/t…",
        "ac 5      ",
        "DURIN STO…",
        "hp 38/44  ",
        "poisoned  ",
        "fighter 5 ",
        "ac 1      ",
        "ELANNA    ",
        "hp 20/22  ",
        "okay      ",
        "cleric/ma…",
        "ac 6      ",
        // 60x40
        "THRENDER…",
        "hp 42/42 ",
        "okay     ",
        "fighter 5",
        "ac 2     ",
        "BROTHER …",
        "hp 26/26 ",
        "okay     ",
        "cleric 4 ",
        "ac 4     ",
        "AMRYL    ",
        "hp 14/14 ",
        "okay     ",
        "mage 4   ",
        "ac 8     ",
        "KEIRA    ",
        "hp 18/31 ",
        "okay     ",
        "fighter/…",
        "ac 5     ",
        "DURIN S…",
        "hp 38/44",
        "poisoned",
        "fighter…",
        "ac 1    ",
        "ELANNA   ",
        "hp 20/22 ",
        "okay     ",
        "cleric/m…",
        "ac 6     ",
        // 110x50
        "THRENDER GRONE  ",
        "hp 42/42 ██████ ",
        "okay            ",
        "fighter 5       ",
        "ac 2            ",
        "BROTHER SEAN   ",
        "hp 26/26 ██████",
        "okay           ",
        "cleric 4       ",
        "ac 4           ",
        "AMRYL          ",
        "hp 14/14 ██████",
        "okay           ",
        "mage 4         ",
        "ac 8           ",
        "KEIRA          ",
        "hp 18/31 ███░░░",
        "okay           ",
        "fighter/thief 5",
        "ac 5           ",
        "DURIN STONEFOOT",
        "hp 38/44 █████░",
        "poisoned       ",
        "fighter 5      ",
        "ac 1           ",
        "ELANNA         ",
        "hp 20/22 █████░",
        "okay           ",
        "cleric/mage 3  ",
        "ac 6           ",
        // 160x14
        "THRENDER GRONE          ",
        "hp 42/42 ████████   ac 2",
        "okay                    ",
        "fighter 5               ",
        "BROTHER SEAN            ",
        "hp 26/26 ████████   ac 4",
        "okay                    ",
        "cleric 4                ",
        "AMRYL                   ",
        "hp 14/14 ████████   ac 8",
        "okay                    ",
        "mage 4                  ",
        "KEIRA                  ",
        "hp 18/31 █████░░░  ac 5",
        "okay                   ",
        "fighter/thief 5        ",
        "DURIN STONEFOOT        ",
        "hp 38/44 ███████░  ac 1",
        "poisoned               ",
        "fighter 5              ",
        "ELANNA                 ",
        "hp 20/22 ███████░  ac 6",
        "okay                   ",
        "cleric/mage 3          ",
        // 160x42
        "THRENDER GRONE          ",
        "hp 42/42 ████████   ac 2",
        "okay                    ",
        "fighter 5               ",
        "BROTHER SEAN            ",
        "hp 26/26 ████████   ac 4",
        "okay                    ",
        "cleric 4                ",
        "AMRYL                   ",
        "hp 14/14 ████████   ac 8",
        "okay                    ",
        "mage 4                  ",
        "KEIRA                  ",
        "hp 18/31 █████░░░  ac 5",
        "okay                   ",
        "fighter/thief 5        ",
        "DURIN STONEFOOT        ",
        "hp 38/44 ███████░  ac 1",
        "poisoned               ",
        "fighter 5              ",
        "ELANNA                 ",
        "hp 20/22 ███████░  ac 6",
        "okay                   ",
        "cleric/mage 3          ",
    ];
    assert_eq!(seen, want);
}

#[test]
fn a_card_is_padded_to_the_column_it_lands_in() {
    // 62 columns divide with a cell left over, so the leftmost column is one
    // wider than the rest and a card padded to the wrong one shows up.
    //
    // The walk is the drawing code's: every frame column, and the card that
    // lands in it. A card whose text does not fill that column exactly would
    // leave a gap or run into the frame.
    for axis in [Axis::Horizontal, Axis::Vertical] {
        let plan = layout::plan(
            Size { cols: 62, rows: 20 },
            &party(),
            &caption(),
            Toggles {
                axis,
                ..Toggles::default()
            },
        );
        let grid = plan.grid.expect("62x20 fits either way");
        assert!(
            grid.across > 1 && grid.widths[0] != grid.widths[1],
            "{axis:?} lost the uneven columns this test needs: {grid:?}"
        );
        for row in 0..grid.down {
            for column in 0..grid.across {
                let Some(card) = grid.cards.get(grid.who_at(row, column)) else {
                    continue;
                };
                let width = usize::from(grid.widths[usize::from(column)]);
                for line in &card.lines {
                    assert_eq!(
                        line.text.chars().count(),
                        width,
                        "{axis:?} row {row} column {column}: {:?} is not {width} wide",
                        line.text
                    );
                }
            }
        }
    }
}
