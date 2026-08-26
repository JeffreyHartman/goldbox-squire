//! The HUD's drawing, against a buffer rather than a terminal.
//!
//! Tickets 037 and 038. The drawing code is meant to be thin: it asks the
//! layout plan what to show and draws that. What is worth testing here is
//! everything the plan cannot check for itself, which is whether the cells
//! land inside the area and carry the colours the state calls for.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use squire_cli::hud::{draw, theme};
use squire_cli::layout::{self, Sitting, Size, Toggles};
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

fn party() -> Party {
    Party {
        state: PartyState::Live,
        characters: vec![
            who("THRENDER GRONE", "fighter", 5, 42, 42, 2, "okay"),
            who("BROTHER SEAN", "cleric", 4, 26, 26, 4, "okay"),
            who("AMRYL", "mage", 4, 14, 14, 8, "okay"),
            who("KEIRA", "fighter/thief", 5, 18, 31, 5, "okay"),
            who("DURIN STONEFOOT", "fighter", 5, 4, 44, 1, "poisoned"),
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

fn sitting() -> Sitting {
    Sitting {
        game: "Pool of Radiance".into(),
        slot: Some('J'),
        panel: "party".into(),
        note: None,
    }
}

/// Draws at a size and hands back the buffer.
fn screen(cols: u16, rows: u16, party: &Party, selected: usize) -> Buffer {
    let area = Rect::new(0, 0, cols, rows);
    let mut buf = Buffer::empty(area);
    let plan = layout::plan(Size { cols, rows }, party, &sitting(), Toggles::default());
    draw::draw(area, &mut buf, &plan, party, selected);
    buf
}

fn text(buf: &Buffer) -> String {
    let area = *buf.area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_party_reaches_the_screen() {
    let buf = screen(110, 50, &party(), 0);
    let shown = text(&buf);
    for name in ["THRENDER GRONE", "DURIN STONEFOOT", "ELANNA"] {
        assert!(shown.contains(name), "{name} is missing");
    }
    assert!(shown.contains("Pool of Radiance"));
    assert!(shown.contains("poisoned"));
}

#[test]
fn the_cards_are_framed_and_the_frame_reaches_the_edges() {
    let buf = screen(80, 40, &party(), 0);
    let shown: Vec<String> = text(&buf).lines().map(str::to_string).collect();
    let top = shown
        .iter()
        .find(|l| l.starts_with('┌'))
        .expect("a top edge");
    assert!(top.ends_with('┐'), "{top:?}");
    assert_eq!(top.chars().count(), 80);
}

#[test]
fn nothing_is_drawn_outside_the_area() {
    // Every size, sane and hostile alike. A cell outside the area would have
    // panicked on the way in; what this proves is that none of them do.
    for (cols, rows) in [
        (1, 1),
        (2, 2),
        (5, 3),
        (17, 4),
        (40, 20),
        (60, 40),
        (160, 14),
        (160, 42),
        (300, 120),
    ] {
        let buf = screen(cols, rows, &party(), 0);
        assert_eq!(*buf.area(), Rect::new(0, 0, cols, rows));
        for line in text(&buf).lines() {
            assert_eq!(line.chars().count(), usize::from(cols));
        }
    }
}

#[test]
fn an_empty_party_draws_the_status_line_and_nothing_else() {
    let empty = Party {
        state: PartyState::NotFound,
        characters: Vec::new(),
    };
    let buf = screen(60, 20, &empty, 0);
    let shown = text(&buf);
    assert!(!shown.contains('┌'), "cards were drawn for nobody");
    assert!(shown.contains("party"), "{shown:?}");
}

#[test]
fn resizing_reflows_rather_than_scrolling() {
    // The same party at two widths lands in a different number of columns,
    // which is the whole of what reflowing means here.
    let party = party();
    let narrow = screen(60, 40, &party, 0);
    let wide = screen(160, 42, &party, 0);
    let count = |buf: &Buffer| {
        text(buf)
            .lines()
            .find(|l| l.starts_with('┌'))
            .map(|l| l.matches('┬').count())
            .unwrap_or(0)
    };
    assert_eq!(count(&narrow), 0, "one card across has no inner divider");
    assert_eq!(count(&wide), 2, "three across has two");
}

#[test]
fn the_highlight_sits_on_one_character_and_no_other() {
    let party = party();
    let buf = screen(60, 40, &party, 3);
    let area = *buf.area();
    let mut highlighted: Vec<u16> = Vec::new();
    for y in 0..area.height {
        if (0..area.width).any(|x| buf[(x, y)].style().bg == Some(theme::SELECTED)) {
            highlighted.push(y);
        }
    }
    assert!(!highlighted.is_empty(), "nothing was highlighted");
    // One card's worth of rows, all touching.
    let runs = highlighted
        .windows(2)
        .filter(|pair| pair[1] != pair[0] + 1)
        .count();
    assert_eq!(runs, 0, "the highlight is in pieces: {highlighted:?}");
}

// --- Stale numbers dim, ticket 038 ----------------------------------------

#[test]
fn a_lost_anchor_greys_the_party_and_keeps_the_numbers() {
    let mut lost = party();
    lost.state = PartyState::NotFound;
    let buf = screen(110, 50, &lost, 0);
    let shown = text(&buf);
    assert!(
        shown.contains("THRENDER GRONE"),
        "the last known numbers went with the anchor"
    );
    assert!(shown.contains("lost"), "the status line says nothing");

    // Nothing in the party block keeps its colour coding: a dimmed red would
    // still read as an alarm, and none of it is live. The wordmark below is
    // not the party and keeps its gold.
    let area = *buf.area();
    let plan = layout::plan(
        Size {
            cols: 110,
            rows: 50,
        },
        &lost,
        &sitting(),
        Toggles::default(),
    );
    let block = plan.grid.as_ref().unwrap().rows();
    for y in 1..=block {
        for x in 0..area.width {
            let fg = buf[(x, y)].style().fg;
            assert!(
                fg != Some(theme::HP_CRITICAL) && fg != Some(theme::GOLD),
                "a live colour survived at {x},{y}"
            );
        }
    }
}

#[test]
fn a_lost_anchor_drops_the_highlight_too() {
    // A highlight on a dimmed card would be the one bright thing on screen.
    let mut lost = party();
    lost.state = PartyState::NotFound;
    let buf = screen(110, 50, &lost, 2);
    let area = *buf.area();
    let any = (0..area.height)
        .any(|y| (0..area.width).any(|x| buf[(x, y)].style().bg == Some(theme::SELECTED)));
    assert!(!any);
}

#[test]
fn a_partial_party_stays_bright_and_says_partial() {
    let mut partial = party();
    partial.state = PartyState::Partial;
    partial.characters.truncate(4);
    let buf = screen(110, 50, &partial, 0);
    let shown = text(&buf);
    assert!(shown.contains("partial"), "{shown:?}");
    let area = *buf.area();
    let bright = (1..area.height - 1)
        .any(|y| (0..area.width).any(|x| buf[(x, y)].style().fg == Some(theme::GOLD)));
    assert!(bright, "a partial party's numbers are live and look it");
}

#[test]
fn the_reason_survives_where_only_the_status_line_fits() {
    let mut lost = party();
    lost.state = PartyState::NotFound;
    let buf = screen(30, 2, &lost, 0);
    assert!(text(&buf).contains("lost"), "{:?}", text(&buf));
}

#[test]
fn a_wounded_character_is_coloured_by_how_wounded() {
    let party = party();
    let buf = screen(110, 50, &party, 5);
    let area = *buf.area();
    let mut seen = Vec::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            if cell.symbol() == "█" {
                seen.push(cell.style().fg);
            }
        }
    }
    assert!(
        seen.contains(&Some(theme::HP_FULL)),
        "nobody was drawn healthy"
    );
    assert!(
        seen.contains(&Some(theme::HP_CRITICAL)),
        "DURIN is on four hit points of forty-four"
    );
}

/// Not an assertion. Writes the screens out so they can be looked at beside
/// the mockups. Run with `cargo test -p squire-cli --test hud -- --ignored`.
#[test]
#[ignore]
fn dump_the_screens() {
    for (cols, rows) in [(40, 20), (60, 40), (110, 50), (160, 14), (160, 42)] {
        let buf = screen(cols, rows, &party(), 0);
        println!("--- {cols}x{rows} ---\n{}\n", text(&buf));
    }
}
