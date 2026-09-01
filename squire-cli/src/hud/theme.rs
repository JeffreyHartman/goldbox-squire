//! The Gold Box colour story: a warm gold on near-black, with EGA accents for
//! the data. The palette is SSI's sixteen-colour one warmed up a little
//!
//! What a line means is [`crate::layout::Tint`]'s answer, decided where a view
//! with no terminal in it can reach it.

use ratatui::style::Color;

use crate::layout::Tint;

/// Metallic gold, the signature of the SSI Gold Box boxes.
pub const GOLD: Color = Color::Rgb(212, 175, 55);
/// A dimmer gold, for frames and rules.
pub const GOLD_DIM: Color = Color::Rgb(120, 100, 32);
/// Warm near-black behind everything.
pub const INK: Color = Color::Rgb(12, 10, 8);
/// Parchment, for body text.
pub const TEXT: Color = Color::Rgb(205, 198, 178);
/// Faint grey.
pub const HINT: Color = Color::Rgb(96, 92, 82);
/// EGA green.
pub const GOOD: Color = Color::Rgb(0, 170, 0);
/// EGA yellow, warmed.
pub const WOUNDED: Color = Color::Rgb(255, 200, 0);
/// EGA red, warmed.
pub const CRITICAL: Color = Color::Rgb(255, 60, 60);

/// The colour a tint is drawn in.
pub fn color(tint: Tint) -> Color {
    match tint {
        Tint::Heading => GOLD,
        Tint::Body => TEXT,
        Tint::Good => GOOD,
        Tint::Wounded => WOUNDED,
        Tint::Critical => CRITICAL,
        Tint::Faint => HINT,
    }
}
