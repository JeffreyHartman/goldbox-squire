//! Turns a layout plan into cells.
//!
//! This file makes no decision about what is shown, and it is not given the
//! party, so it cannot. The plan arrives holding every line's text and its
//! [`crate::layout::Tint`], and this file turns those into cells.
//!
//! It writes into a `Buffer` rather than taking a `Frame`, so that the tests
//! draw at any size at all and read the cells back with no terminal anywhere.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::hud::theme;
use crate::layout::{self, Plan};

/// The whole screen: the header, the cards, the logo, the status line.
///
/// No character is picked out. A highlight that always sits on somebody makes
/// that character look like the party leader when it means nothing, and there
/// is nothing yet to select a character *for*. When there is, the highlight
/// comes back with the action it belongs to.
pub fn draw(area: Rect, buf: &mut Buffer, plan: &Plan) {
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
    cards(area, buf, plan, grid);

    // The plan was made for the size the terminal last reported, and a resize
    // can land between that question and this draw. So the room is measured
    // again here rather than trusted, and a logo that no longer fits is
    // simply not drawn.
    if plan.show_logo {
        // The room going spare, and nowhere else. What is left below is where
        // a map or a journal will live, if they ever do.
        let spare = area
            .height
            .saturating_sub(layout::EDGE_ROWS)
            .saturating_sub(grid.rows());
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
    Style::default().fg(theme::color(plan.liveness.tint()))
}

fn cards(area: Rect, buf: &mut Buffer, plan: &Plan, grid: &layout::Grid) {
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
                // An uneven party leaves the last card short, not a middle
                // one, whichever way the grid flows.
                let who = grid.who_at(row, column);
                if let Some(card) = grid.cards.get(who) {
                    if let Some(planned) = card.lines.get(usize::from(line)) {
                        put(
                            area,
                            buf,
                            x + 2,
                            y,
                            &planned.text,
                            line_style(plan, planned),
                        );
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

/// The tint the plan gave the line, in this palette.
///
/// A lost anchor arrives as [`layout::Tint::Faint`] on every line, so the
/// terminal's DIM attribute is the only thing left to add here.
fn line_style(plan: &Plan, line: &layout::Line) -> Style {
    let style = Style::default().fg(theme::color(line.tint));
    if plan.dim {
        style.add_modifier(Modifier::DIM)
    } else {
        style
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
