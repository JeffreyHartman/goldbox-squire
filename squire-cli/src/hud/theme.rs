//! The Gold Box colour story: a warm gold on near-black, with EGA accents for
//! the data. The palette is SSI's sixteen-colour one warmed up a little

use ratatui::style::Color;

/// Metallic gold, the signature of the SSI Gold Box boxes.
pub const GOLD: Color = Color::Rgb(212, 175, 55);
/// A dimmer gold, for frames and rules.
pub const GOLD_DIM: Color = Color::Rgb(120, 100, 32);
/// Warm near-black behind everything.
pub const INK: Color = Color::Rgb(12, 10, 8);
/// Parchment, for body text.
pub const TEXT: Color = Color::Rgb(205, 198, 178);
/// Faint grey, for hints.
pub const HINT: Color = Color::Rgb(96, 92, 82);
/// Hit points at or near full.
pub const HP_FULL: Color = Color::Rgb(0, 170, 0);
/// Hit points wounded.
pub const HP_WOUNDED: Color = Color::Rgb(255, 200, 0);
/// Hit points critical, and any condition that is not `okay`.
pub const HP_CRITICAL: Color = Color::Rgb(255, 60, 60);
/// A condition that is nothing to worry about.
pub const GOOD: Color = Color::Rgb(0, 170, 0);

/// The colour of a character's hit points, by how many are left.
/// Three bands rather than a gradient
pub fn hit_points(current: i16, maximum: u8) -> Color {
    if maximum == 0 || current <= 0 {
        return HP_CRITICAL;
    }
    let left = f64::from(current) / f64::from(maximum);
    if left >= 0.66 {
        HP_FULL
    } else if left >= 0.33 {
        HP_WOUNDED
    } else {
        HP_CRITICAL
    }
}

/// The colour of one condition. Anything that is not `okay` gets color
pub fn condition(word: &str) -> Color {
    if word.eq_ignore_ascii_case("okay") {
        GOOD
    } else {
        HP_CRITICAL
    }
}
