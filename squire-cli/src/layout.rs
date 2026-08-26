//! The layout plan: rows, columns and a party in, what is shown out.
//!
//! A pure function. No terminal, no drawing library, no input, no clock. Call
//! it with any size at all, including sizes no real terminal would produce,
//! and it answers instantly. Every decision this effort made about what a HUD
//! shows lives here rather than scattered through drawing code, which is what
//! makes those decisions testable.
//!
//! The rules answer "does this fit". They never name a size, a monitor, or a
//! layout mode. Every number in this file comes from the party, from the game
//! data, or from a judgement about legibility that is argued for where it is
//! written. If a reviewer finds a constant here that came from somebody's
//! screen, the module is wrong.
//!
//! A preference is a different question and arrives as [`Toggles`]. The plan
//! never decides one, or a preference ends up buried inside a fitting rule
//! where nobody can find it.

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// A screen, in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

/// What the user asked for, as opposed to what fits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Toggles {
    /// The ability scores, which are off until a key turns them on.
    pub abilities: bool,
    /// A number of cards across, overriding the rule's answer. Ignored when
    /// it does not fit, because a key that empties the screen is a trap.
    pub across: Option<u16>,
}

/// How much of the party the screen is currently telling the truth about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// Every character was read this poll.
    Live,
    /// Some were. The game is mid-load, or the party changed.
    Partial,
    /// None were, and there are numbers on screen from before. They are the
    /// last known values and they are no longer live.
    Lost,
    /// None were, and none ever have been. Nothing to dim.
    Waiting,
}

/// One line of one card, and what is on it.
///
/// Presence is the plan's business; the characters are the wire it is drawn
/// from. [`line_text`] turns one of these into the text for a given width,
/// and it lives here rather than in the drawing code so that truncation and
/// alignment are decided once and tested without a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardLine {
    /// The name, with the class and level pushed right when they fit.
    Name { class_here: bool },
    /// Hit points, a bar this many cells wide (zero for none), and armour
    /// class when the line still has room.
    HitPoints { bar: u16, armor_here: bool },
    /// One condition, by its position in the character's list.
    Condition(usize),
    /// The class and level, on their own line.
    Class,
    /// Armour class, on its own line.
    Armor,
    /// All six ability scores. Never one of them.
    Abilities,
}

/// One character's card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Top to bottom, already cut to the card's height.
    pub lines: Vec<CardLine>,
}

/// Where the cards go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub across: u16,
    pub down: u16,
    /// The text width inside each column of cards, left to right. The columns
    /// the division left over go to the leftmost cards, so that the frame
    /// reaches the right edge exactly.
    pub widths: Vec<u16>,
    /// Lines inside every card. One number for the whole party: a card laid
    /// out differently from its neighbour reads as a bug.
    pub card_rows: u16,
    /// One per character, in marching order.
    pub cards: Vec<Card>,
}

impl Grid {
    /// The rows the grid occupies, frame included.
    pub fn rows(&self) -> u16 {
        self.down * (self.card_rows + 1) + 1
    }
}

/// The words that say which run is on screen.
///
/// Not a [`crate::wizard::Sitting`], which is the save folder and the slot the
/// wizard resolved. This is what the header and the status line say about the
/// run, and the plan fits both to the width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caption {
    /// The game's display name.
    pub game: String,
    /// The save slot, when one has been picked.
    pub slot: Option<char>,
    /// The panel on screen. Only one exists; the number keys are reserved.
    pub panel: String,
    /// The watch loop's latest word, when it had one.
    pub note: Option<String>,
}

/// What is shown at one size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub size: Size,
    /// One line, already fitted to the width.
    pub header: String,
    /// One line, already fitted to the width. At a size where nothing else
    /// fits, this is the whole screen.
    pub status: String,
    /// `None` when no card fits. The status line still says what is going on.
    pub grid: Option<Grid>,
    /// Squire's name, drawn large, in the room going spare.
    pub wordmark: bool,
    /// The numbers on screen are not live. Drawing dims the party block.
    pub dim: bool,
    pub liveness: Liveness,
}

/// The rows the header and the status line take. Both are always drawn: at a
/// hostile size the status line is the only thing that can be.
const CHROME_ROWS: u16 = 2;

/// The longest a hit point bar gets. Past this it stops telling you anything
/// more and starts eating the line.
const BAR_MAX: u16 = 12;

/// The shortest a bar can be and still mean anything. Below this it is a
/// couple of blocks that barely move, so the numbers go alone instead.
const BAR_MIN: u16 = 4;

/// The fewest lines a card is worth drawing at: the name and the hit points.
const CARD_MIN_ROWS: u16 = 2;

/// The space between the name and the class when they share a line. Less than
/// this and the two words read as one.
const NAME_GAP: u16 = 3;

/// A space each side of the card's text, inside its frame.
const CARD_PAD: u16 = 2;

/// The air a card wants around its widest line, beyond the text itself.
///
/// This is the one number here that is a judgement rather than a measurement.
/// Widening it moves a middling width from two columns of cards to one.
const CARD_AIR: u16 = 4;

/// The keys the HUD answers to, shown on the status line so that they are
/// visible without reading the source.
const KEY_HINTS: &str = "q quit · a abilities · c columns · s slot";

/// The rows Squire's name takes when it is drawn large.
pub const WORDMARK_ROWS: u16 = 5;

/// Rows that must be spare, beyond the wordmark itself, before it is worth
/// drawing one. A wordmark jammed against the cards is not room to spare.
const WORDMARK_AIR: u16 = 2;

/// What is shown at this size, for this party, with these preferences.
pub fn plan(size: Size, party: &Party, caption: &Caption, toggles: Toggles) -> Plan {
    let liveness = liveness(party);
    let grid = choose(size, party, toggles);
    let wordmark = wordmark_fits(size, grid.as_ref());
    let left = status_text(caption, party, liveness, grid.as_ref(), toggles);

    Plan {
        size,
        header: fit(&header_text(caption), size.cols),
        status: two_up(&left, KEY_HINTS, size.cols),
        grid,
        wordmark,
        dim: liveness == Liveness::Lost,
        liveness,
    }
}

/// How much of the party the numbers on screen are telling the truth about.
///
/// A party state of not-found means the anchor is gone. Whether that is worth
/// dimming depends on whether anything was ever found: the caller keeps the
/// last party it saw and hands it back with the current state, so characters
/// with a not-found state are last known values and characters with none are
/// a run that has not started yet.
fn liveness(party: &Party) -> Liveness {
    match party.state {
        PartyState::Live => Liveness::Live,
        PartyState::Partial => Liveness::Partial,
        PartyState::NotFound if party.characters.is_empty() => Liveness::Waiting,
        PartyState::NotFound => Liveness::Lost,
    }
}

// --- The grid -------------------------------------------------------------

/// The widest each line gets across the whole party.
///
/// Held so that every card in one party is laid out the same way, whatever
/// the length of one character's name.
struct Shape {
    name: u16,
    class: u16,
    hit_points: u16,
    armor: u16,
    abilities: u16,
}

fn shape_of(party: &[Character]) -> Shape {
    let widest = |f: fn(&Character) -> String| -> u16 {
        party.iter().map(|c| width(&f(c))).max().unwrap_or(0)
    };
    Shape {
        name: widest(|c| c.name.clone()),
        class: widest(class_text),
        hit_points: widest(hit_points_text),
        armor: widest(armor_text),
        abilities: widest(abilities_text),
    }
}

impl Shape {
    /// The width a card is happiest at: its widest line, the card's own
    /// padding, and the air around it.
    ///
    /// Every part of this comes from the party and from the game's class
    /// names. None of it comes from anybody's monitor.
    fn wants(&self) -> u16 {
        self.name
            .saturating_add(NAME_GAP)
            .saturating_add(self.class)
            .saturating_add(CARD_PAD)
            .saturating_add(CARD_AIR)
    }

    /// The narrowest card worth drawing: one that can hold a whole hit point
    /// line. A name is truncated with an ellipsis and so sets no floor.
    fn floor(&self) -> u16 {
        self.hit_points.max(1)
    }
}

/// The numbers of cards across that a party of `n` divides into evenly.
///
/// A ragged last row would make one card differ from its neighbours, which the
/// party-wide shape rule already forbids. The key that changes the number
/// across steps through this same list, so it can never ask for an
/// arrangement the rule would not have offered.
pub fn arrangements(n: u16) -> Vec<u16> {
    (1..=n).filter(|across| n % across == 0).collect()
}

/// How many cards across, how wide, and how tall.
///
/// The rule picks the number across that gets each card closest to the width
/// it wants. Rows are a hard limit: a screen too short for two rows of cards
/// gets one row of six, whatever the width says.
///
/// There are no named layouts here. A sidecar and a strip are what the ends
/// of this range look like, not states the code can be in.
fn choose(size: Size, party: &Party, toggles: Toggles) -> Option<Grid> {
    let n = u16::try_from(party.characters.len()).ok()?;
    if n == 0 {
        return None;
    }
    let shape = shape_of(&party.characters);
    let body = size.rows.checked_sub(CHROME_ROWS)?;

    let even = arrangements(n);

    let build = |across: u16| -> Option<(u16, Grid)> {
        let down = n / across;
        let text = size
            .cols
            .checked_sub(across + 1)?
            .checked_div(across)?
            .checked_sub(CARD_PAD)?;
        let rows = body.checked_sub(down + 1)?.checked_div(down)?;
        if text < shape.floor() || rows < CARD_MIN_ROWS {
            return None;
        }

        // Every card in a party is shaped from the narrowest column, so the
        // odd spare cell a division leaves over is padding and never a field.
        let spare = size.cols - (across + 1) - across * (text + CARD_PAD);
        let widths: Vec<u16> = (0..across).map(|i| text + u16::from(i < spare)).collect();

        let tallest = party
            .characters
            .iter()
            .map(|c| card_lines(c, text, &shape, toggles, u16::MAX).len())
            .max()
            .unwrap_or(0);
        let card_rows = rows.min(u16::try_from(tallest).unwrap_or(u16::MAX));
        let cards = party
            .characters
            .iter()
            .map(|c| Card {
                lines: card_lines(c, text, &shape, toggles, card_rows),
            })
            .collect();

        Some((
            text.abs_diff(shape.wants()),
            Grid {
                across,
                down,
                widths,
                card_rows,
                cards,
            },
        ))
    };

    // A key that asks for an arrangement that does not fit is not an error.
    // The rule's answer stands instead, because a key that empties the screen
    // is worse than a key that appears to do nothing.
    if let Some(asked) = toggles.across {
        if even.contains(&asked) {
            if let Some((_, grid)) = build(asked) {
                return Some(grid);
            }
        }
    }

    even.into_iter()
        .filter_map(build)
        .min_by_key(|(miss, _)| *miss)
        .map(|(_, grid)| grid)
}

/// One card's lines, in the settled drop order, cut to `budget` lines.
///
/// Last to go first: name, hit points, conditions, class and level, armour
/// class. Ability scores are not in that order at all; they are a toggle, and
/// they sit last so that turning them on never costs a condition. Conditions
/// rank high because a silent `poisoned` is the thing you most want to notice
/// without looking away from the game.
///
/// What differs from a table is that a line which does not fit beside another
/// simply gets its own.
fn card_lines(
    c: &Character,
    width: u16,
    shape: &Shape,
    toggles: Toggles,
    budget: u16,
) -> Vec<CardLine> {
    // Priority, then the line. Lower survives longer.
    let mut lines: Vec<(u8, CardLine)> = Vec::new();

    let class_here = width
        >= shape
            .name
            .saturating_add(NAME_GAP)
            .saturating_add(shape.class);
    lines.push((0, CardLine::Name { class_here }));

    // The bar takes what the hit point line has left after armour class.
    let room = width.saturating_sub(shape.hit_points + 1);
    let armor_here = room >= shape.armor.saturating_add(NAME_GAP);
    let bar = if armor_here {
        room.saturating_sub(shape.armor + CARD_PAD).min(BAR_MAX)
    } else {
        room.min(BAR_MAX)
    };
    let bar = if bar < BAR_MIN { 0 } else { bar };
    lines.push((1, CardLine::HitPoints { bar, armor_here }));

    for i in 0..conditions(c).len() {
        lines.push((2, CardLine::Condition(i)));
    }
    if !class_here {
        lines.push((3, CardLine::Class));
    }
    if !armor_here {
        lines.push((4, CardLine::Armor));
    }
    // All six or none: one score is not worth a line.
    if toggles.abilities && shape.abilities <= width {
        lines.push((5, CardLine::Abilities));
    }

    // Drop from the bottom of the order, then put what survived back into
    // reading order.
    let mut order: Vec<usize> = (0..lines.len()).collect();
    order.sort_by_key(|&i| lines[i].0);
    order.truncate(usize::from(budget));
    order.sort_unstable();
    order.into_iter().map(|i| lines[i].1).collect()
}

// --- The words on a card --------------------------------------------------

/// Everything currently on the character, one item per line.
///
/// Squire reads one status byte today, so this list has one item. The card is
/// shaped for a list because the effects read will fill it later, and when it
/// does the ability scores are what falls off a crowded card rather than the
/// thing that is currently killing you.
pub fn conditions(c: &Character) -> Vec<String> {
    vec![c
        .status
        .clone()
        .unwrap_or_else(|| format!("? {:#04x}", c.status_raw))]
}

fn class_name(c: &Character) -> String {
    c.class
        .clone()
        .unwrap_or_else(|| format!("? {:#04x}", c.class_raw))
}

/// The class and level as they appear beside the name.
fn class_text(c: &Character) -> String {
    format!("{} · lvl {}", class_name(c), c.level)
}

fn hit_points_text(c: &Character) -> String {
    format!("hp {}/{}", c.hit_points_current, c.hit_points_maximum)
}

fn armor_text(c: &Character) -> String {
    format!("ac {}", c.armor_class)
}

/// Six numbers in the order every Gold Box screen prints them.
///
/// No labels: a player who wants these knows the order, and six abbreviations
/// cost more of the card than the numbers do. Percentile strength goes in
/// brackets because it has to survive the slashes.
fn abilities_text(c: &Character) -> String {
    let strength = if c.strength_exceptional > 0 {
        format!("{}({})", c.strength, c.strength_exceptional)
    } else {
        c.strength.to_string()
    };
    format!(
        "{strength}/{}/{}/{}/{}/{}",
        c.intelligence, c.wisdom, c.dexterity, c.constitution, c.charisma
    )
}

/// A hit point bar. Full blocks for what is left, light for what is gone.
fn bar_text(c: &Character, width: u16) -> String {
    let width = usize::from(width);
    if width == 0 || c.hit_points_maximum == 0 {
        return String::new();
    }
    let top = f64::from(c.hit_points_maximum);
    let current = f64::from(c.hit_points_current).max(0.0);
    let mut filled = ((width as f64) * current / top).round() as usize;
    filled = filled.min(width);
    // A living character never draws an empty bar, so that hurt and down
    // never look the same at a glance.
    if c.hit_points_current > 0 {
        filled = filled.max(1);
    }
    "█".repeat(filled) + &"░".repeat(width - filled)
}

/// One planned line as text, exactly `width` cells wide.
pub fn line_text(c: &Character, line: &CardLine, width: u16) -> String {
    match line {
        CardLine::Name { class_here: false } => fit(&c.name, width),
        CardLine::Name { class_here: true } => two_up(&c.name, &class_text(c), width),
        CardLine::HitPoints { bar, armor_here } => {
            let hp = hit_points_text(c);
            let drawn = bar_text(c, *bar);
            let left = if drawn.is_empty() {
                hp
            } else {
                format!("{hp} {drawn}")
            };
            if *armor_here {
                two_up(&left, &armor_text(c), width)
            } else {
                fit(&left, width)
            }
        }
        CardLine::Condition(i) => fit(conditions(c).get(*i).map_or("", |s| s), width),
        // The own-line form drops the separator: a card narrow enough to need
        // this line is narrow enough to want the four cells back.
        CardLine::Class => fit(&format!("{} {}", class_name(c), c.level), width),
        CardLine::Armor => fit(&armor_text(c), width),
        CardLine::Abilities => fit(&abilities_text(c), width),
    }
}

// --- The header and the status line ---------------------------------------

fn header_text(caption: &Caption) -> String {
    match caption.slot {
        Some(letter) => format!("gbs — {} · slot {letter}", caption.game),
        None => format!("gbs — {}", caption.game),
    }
}

/// What the status line says about the party, before the keys are added.
///
/// The panel's number goes first because the number keys are what selects it,
/// and a panel that shows its own key needs no menu built for it.
///
/// The arrangement is named here rather than left to be counted off the screen.
/// There are no named layouts in this program, so the honest name for one is
/// how many cards are across and whether the rule chose it or a key did. A key
/// that cycles with no read-out is a key you press until something looks right.
fn status_text(
    caption: &Caption,
    party: &Party,
    liveness: Liveness,
    grid: Option<&Grid>,
    toggles: Toggles,
) -> String {
    let state = match liveness {
        Liveness::Live => format!("live · {}", party.characters.len()),
        Liveness::Partial => format!("partial · {} shown", party.characters.len()),
        Liveness::Lost => "anchor lost, rescanning".to_string(),
        Liveness::Waiting => "no party yet".to_string(),
    };
    let mut text = format!("1 {} · {state}", caption.panel);
    if let Some(grid) = grid {
        // `chosen` means a key asked for this and `c` will move off it.
        // `auto` means the rule picked it and a resize may change it.
        let how = if toggles.across == Some(grid.across) {
            "chosen"
        } else {
            "auto"
        };
        text.push_str(&format!(" · {} across, {how}", grid.across));
    }
    if let Some(note) = &caption.note {
        text.push_str(" · ");
        text.push_str(note);
    }
    text
}

// --- The wordmark ---------------------------------------------------------

/// Squire's name in block letters, one string per row.
pub fn wordmark() -> Vec<String> {
    const FONT: [(char, [&str; WORDMARK_ROWS as usize]); 6] = [
        ('S', ["█████", "█    ", "█████", "    █", "█████"]),
        ('Q', ["█████", "█   █", "█ █ █", "█  ██", "█████"]),
        ('U', ["█   █", "█   █", "█   █", "█   █", "█████"]),
        ('I', ["█████", "  █  ", "  █  ", "  █  ", "█████"]),
        ('R', ["█████", "█   █", "█████", "█  █ ", "█   █"]),
        ('E', ["█████", "█    ", "█████", "█    ", "█████"]),
    ];
    (0..WORDMARK_ROWS as usize)
        .map(|row| {
            FONT.iter()
                .map(|(_, glyph)| glyph[row])
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}

/// Whether there is room to spare after everything the party needs.
///
/// Roomy is a question, not a measurement. What is roomy for a party panel
/// alone is cramped for the same panel beside a map, and when a map exists
/// this is where that changes.
fn wordmark_fits(size: Size, grid: Option<&Grid>) -> bool {
    let Some(grid) = grid else {
        return false;
    };
    let block = wordmark();
    let widest = block.iter().map(|l| width(l)).max().unwrap_or(0);
    let spare = size
        .rows
        .saturating_sub(CHROME_ROWS)
        .saturating_sub(grid.rows());
    widest <= size.cols && spare >= WORDMARK_ROWS + WORDMARK_AIR
}

// --- Fitting text ---------------------------------------------------------

fn width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// `text`, padded or cut to exactly `width` cells.
///
/// Cut text ends in an ellipsis so that a truncated name is never mistaken
/// for a short one. Below two cells there is no room to say that, so the text
/// is simply cut.
fn fit(text: &str, width: u16) -> String {
    let width = usize::from(width);
    let len = text.chars().count();
    if len <= width {
        let mut out = text.to_string();
        out.push_str(&" ".repeat(width - len));
        return out;
    }
    if width < 2 {
        return text.chars().take(width).collect();
    }
    let mut out: String = text.chars().take(width - 1).collect();
    out.push('…');
    out
}

/// `left` and `right` on one line, pushed to opposite ends.
///
/// When they will not both fit, the right one goes: it is always the less
/// important of the two, and half of each is worse than all of one.
fn two_up(left: &str, right: &str, width: u16) -> String {
    let (l, r) = (self::width(left), self::width(right));
    if l.saturating_add(r).saturating_add(1) > width {
        return fit(left, width);
    }
    let mut out = left.to_string();
    out.push_str(&" ".repeat(usize::from(width - l - r)));
    out.push_str(right);
    out
}
