//! Turns a layout plan into cells.
//!
//! This file makes no decision about what is shown. It asks [`crate::layout`]
//! and draws the answer. If a field's presence is ever decided here, the HUD
//! is wrong: the whole point of the plan is that the rules are somewhere a
//! test can reach without a terminal.
//!
//! It writes into a `Buffer` rather than taking a `Frame`, so that the tests
//! draw at any size at all and read the cells back with no terminal anywhere.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use squire_core::session::Party;

use crate::hud::theme;
use crate::layout::{self, Axis, CardLine, Plan};

/// The whole screen: the header, the cards, the logo, the status line.
///
/// No character is picked out. A highlight that always sits on somebody makes
/// that character look like the party leader when it means nothing, and there
/// is nothing yet to select a character *for*. When there is, the highlight
/// comes back with the action it belongs to.
pub fn draw(area: Rect, buf: &mut Buffer, plan: &Plan, party: &Party) {
    buf.set_style(area, Style::default().bg(theme::INK).fg(theme::TEXT));

    put(
        area,
        buf,
        0,
        0,
        &plan.header,
        Style::default().fg(theme::GOLD),
    );

    if area.height >= 1 {
        let last = area.height - 1;
        put(area, buf, 0, last, &plan.status, status_style(plan));
    }

    let Some(grid) = plan.grid.as_ref() else {
        return;
    };
    cards(area, buf, plan, grid, party);

    // The plan was made for the size the terminal last reported, and a resize
    // can land between that question and this draw. So the room is measured
    // again here rather than trusted, and a logo that no longer fits is
    // simply not drawn.
    if plan.show_logo {
        // The room going spare, and nowhere else. What is left below is where
        // a map or a journal will live, if they ever do.
        let spare = area.height.saturating_sub(layout::EDGE_ROWS).saturating_sub(grid.rows());
        let top = 1 + grid.rows() + spare.saturating_sub(layout::LOGO_ROWS) / 2;
        for (i, line) in layout::logo().iter().enumerate() {
            let width = u16::try_from(line.chars().count()).unwrap_or(u16::MAX);
            let x = area.width.saturating_sub(width) / 2;
            let y = top + u16::try_from(i).unwrap_or(0);
            put(area, buf, x, y, line, Style::default().fg(theme::GOLD));
        }
    }
}

/// The status line's colour says the same thing its words do, because a HUD is
/// read from the corner of the eye and the colour arrives first.
fn status_style(plan: &Plan) -> Style {
    let colour = match plan.liveness {
        layout::Liveness::Live => theme::TEXT,
        layout::Liveness::Partial => theme::HP_WOUNDED,
        layout::Liveness::Lost => theme::HP_CRITICAL,
        layout::Liveness::Waiting => theme::HINT,
    };
    Style::default().fg(colour)
}

fn cards(area: Rect, buf: &mut Buffer, plan: &Plan, grid: &layout::Grid, party: &Party) {
    // Stale numbers are still worth something, so they stay on screen. They
    // go grey and lose their colour coding, which is unmissable in peripheral
    // vision, which is where a HUD is read from.
    let frame = Style::default().fg(if plan.dim {
        theme::HINT
    } else {
        theme::GOLD_DIM
    });

    let mut y = 1;
    for row in 0..grid.down {
        rule(
            area,
            buf,
            grid,
            y,
            if row == 0 { Edge::Top } else { Edge::Mid },
            frame,
        );
        y += 1;
        for line in 0..grid.card_rows {
            let mut x = 0;
            for column in 0..grid.across {
                let width = grid.widths[usize::from(column)];
                set(area, buf, x, y, "│", frame);
                // Horizontal fills a row before starting the next; vertical
                // fills a column before starting the next. Either way, an
                // uneven party leaves the last one short, not a middle one.
                let who = match grid.axis {
                    Axis::Horizontal => usize::from(row * grid.across + column),
                    Axis::Vertical => usize::from(column * grid.down + row),
                };
                if let (Some(card), Some(c)) = (grid.cards.get(who), party.characters.get(who)) {
                    if let Some(planned) = card.lines.get(usize::from(line)) {
                        let text = layout::line_text(c, planned, width);
                        put(area, buf, x + 2, y, &text, line_style(plan, planned, c));
                    }
                }
                x += width + 3;
            }
            set(area, buf, x, y, "│", frame);
            y += 1;
        }
    }
    rule(area, buf, grid, y, Edge::Bottom, frame);
}

/// What one planned line looks like.
///
/// Colour is the only thing decided here, and it is decided from the numbers
/// rather than from the size. Nothing about presence is touched.
fn line_style(plan: &Plan, line: &CardLine, c: &squire_core::record::Character) -> Style {
    if plan.dim {
        // One grey for the whole block. A dimmed red would still read as an
        // alarm, and the point of dimming is that none of it is live.
        Style::default().fg(theme::HINT).add_modifier(Modifier::DIM)
    } else {
        let colour: Color = match line {
            CardLine::Name { .. } => theme::GOLD,
            CardLine::HitPoints { .. } => {
                theme::hit_points(c.hit_points_current, c.hit_points_maximum)
            }
            CardLine::Condition(i) => layout::conditions(c)
                .get(*i)
                .map_or(theme::TEXT, |w| theme::condition(w)),
            CardLine::Class | CardLine::Armor | CardLine::Abilities => theme::TEXT,
        };
        Style::default().fg(colour)
    }
}

enum Edge {
    Top,
    Mid,
    Bottom,
}

/// One horizontal rule of the grid's frame.
fn rule(area: Rect, buf: &mut Buffer, grid: &layout::Grid, y: u16, edge: Edge, style: Style) {
    let (left, mid, right) = match edge {
        Edge::Top => ("┌", "┬", "┐"),
        Edge::Mid => ("├", "┼", "┤"),
        Edge::Bottom => ("└", "┴", "┘"),
    };
    let mut x = 0;
    for (i, width) in grid.widths.iter().enumerate() {
        set(area, buf, x, y, if i == 0 { left } else { mid }, style);
        for cell in 1..=(width + 2) {
            set(area, buf, x + cell, y, "─", style);
        }
        x += width + 3;
    }
    set(area, buf, x, y, right, style);
}

/// Writes `text` at `x`, `y` inside `area`, clipped to it.
///
/// Everything the plan produces already fits, so clipping never triggers in
/// practice. It is here because a HUD that panics when a terminal reports a
/// size it does not have is worse than one that draws a short line.
fn put(area: Rect, buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    if y >= area.height {
        return;
    }
    for (at, ch) in (x..).zip(text.chars()) {
        if at >= area.width {
            return;
        }
        set(area, buf, at, y, &ch.to_string(), style);
    }
}

fn set(area: Rect, buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if x >= area.width || y >= area.height {
        return;
    }
    buf[(area.x + x, area.y + y)]
        .set_symbol(symbol)
        .set_style(style);
}
