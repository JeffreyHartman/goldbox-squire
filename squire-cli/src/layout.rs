//! Turns a party and a terminal [`Size`] into a [`Plan`]. This module does no
//! terminal I/O.
//!
//! [`plan()`] is the entry point. It decides three things:
//!
//! - How many cards fit, and how they are arranged ([`fit_grid`])
//! - What goes on each card, and what leaves first when space runs short
//!   ([`card_fields`] and [`cut`])
//! - The text of the header and the status line
//!
//! [`crate::hud::draw`] draws the answer. It works out cell positions and
//! turns each line's [`Tint`] into a color, and it is never given the party.
//! [`Toggles`] carry what the user asked for. A toggle is never a fitting
//! rule.

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

/// A screen, in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

/// Which way the cards flow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Axis {
    /// Fill a row, then start the next row.
    #[default]
    Horizontal,
    /// Fill a column, then start the next column.
    Vertical,
}

/// What the user asked for. What fits is decided elsewhere.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Toggles {
    /// Show the ability scores. A key turns them on, and they start off.
    pub abilities: bool,
    /// Which way the cards flow. A key flips it.
    pub axis: Axis,
}

/// How much of the party on screen is still true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// Every character was read this poll.
    Live,
    /// Some were read. The game is mid-load, or the party changed.
    Partial,
    /// None were read. The numbers on screen are the last ones found, and
    /// they are no longer live.
    Lost,
    /// None were read, and none ever were. There is nothing to dim.
    Waiting,
}

impl Liveness {
    /// What the status line's words mean.
    pub fn tint(self) -> Tint {
        match self {
            Liveness::Live => Tint::Body,
            Liveness::Partial => Tint::Wounded,
            Liveness::Lost => Tint::Critical,
            Liveness::Waiting => Tint::Faint,
        }
    }
}

/// What a line means, for a view to color as it sees fit.
///
/// A tint is never a color. `layout` decides that a character on four hit
/// points of forty-four is critical; a view decides what critical looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    /// The character's name.
    Heading,
    /// Text that carries no alarm of its own.
    Body,
    /// Nothing to worry about.
    Good,
    /// Worth a look.
    Wounded,
    /// Worth looking away from the game for.
    Critical,
    /// Not live. Nothing here is worth reading as an alarm.
    Faint,
}

/// What one line of a card is about.
///
/// A variant names a field, not its text. [`DROP_ORDER`] ranks the same list
/// by what survives a narrowing card, and [`CardShape`] says which fields
/// share a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// The character's name, with the class beside it when there is room.
    Name,
    /// The hit points, the bar, and the armor class when there is room.
    HitPoints,
    /// One condition, by its index into [`conditions`].
    Condition(usize),
    /// The class and the level, on a line of their own.
    Class,
    /// The armor class, on a line of its own.
    Armor,
    /// All six ability scores. A card never shows fewer than six.
    Abilities,
}

/// One finished line of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Exactly as wide as the card's column, padded or cut to it.
    pub text: String,
    /// What the line means. A view turns this into a color.
    pub tint: Tint,
    /// Which field the text came from, so that a caller can name a line
    /// without matching on its wording.
    pub field: Field,
}

/// One character's card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Top to bottom, already cut to the card's height.
    pub lines: Vec<Line>,
}

/// Where the cards go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    /// Cards in a row.
    pub across: u16,
    /// Cards in a column.
    pub down: u16,
    /// The text width inside each column of cards, left to right. One entry
    /// per column. Integer division leaves a cell or two spare. The leftmost
    /// columns take them, so the frame reaches the right edge exactly.
    pub widths: Vec<u16>,
    /// Lines inside one card. One number for the whole party, because every
    /// card gets the same layout.
    pub card_rows: u16,
    /// One per character, in marching order.
    pub cards: Vec<Card>,
    /// Which fields share a line, and how wide the bar is.
    pub shape: CardShape,
    /// Which way an uneven last card lands: a short last row, or a short last
    /// column.
    pub axis: Axis,
}

impl Grid {
    /// The rows the grid occupies, frame included.
    pub fn rows(&self) -> u16 {
        self.down * (self.card_rows + 1) + 1
    }

    /// The character whose card sits at `row`, `column`.
    ///
    /// The inverse of [`column_of`], which is why the two live together.
    pub fn who_at(&self, row: u16, column: u16) -> usize {
        match self.axis {
            Axis::Horizontal => usize::from(row * self.across + column),
            Axis::Vertical => usize::from(column * self.down + row),
        }
    }
}

/// Which column the card for `who` lands in. `Horizontal` fills a row before
/// starting the next, so the column cycles; `Vertical` fills a column before
/// starting the next, so it steps once every `down` cards.
fn column_of(who: usize, across: u16, down: u16, axis: Axis) -> usize {
    match axis {
        Axis::Horizontal => who % usize::from(across),
        Axis::Vertical => who / usize::from(down),
    }
}

/// The words that say which run is on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caption {
    /// The game's display name.
    pub game: String,
    /// The save slot, when the wizard picked one.
    pub slot: Option<char>,
    /// The panel on screen.
    pub panel: String,
    /// The watch loop's latest word. It goes at the end of the status line.
    pub note: Option<String>,
}

/// Everything that goes on screen: every line's text, already fitted, and
/// what each one means.
///
/// [`crate::hud::draw`] puts these strings into cells in the colors they ask
/// for. It is not given the party, so it cannot decide anything about
/// content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The top line, cut to the width.
    pub header: String,
    /// The bottom line, cut to the width.
    pub status: String,
    /// `None` when not one card fits. The status line still says what is
    /// going on.
    pub grid: Option<Grid>,
    /// True when there is room for Squire's name in block letters.
    pub show_logo: bool,
    /// True when `liveness` is [`Liveness::Lost`]. The party block goes gray.
    pub dim: bool,
    /// Sets the color of the status line.
    pub liveness: Liveness,
}

/// The rows the header and the status line take. Always drawn.
pub(crate) const EDGE_ROWS: u16 = 2;

/// The longest a hit point bar gets.
const BAR_MAX: u16 = 12;

/// The shortest a bar can be.
const BAR_MIN: u16 = 4;

/// The fewest lines a card is worth drawing at: the name and the hit points.
const CARD_MIN_ROWS: u16 = 2;

/// The space between the name and the class when they share a line.
const NAME_GAP: u16 = 3;

/// A space each side of the card's text, inside its frame.
const CARD_PAD: u16 = 2;

/// The keys the HUD answers to. The status line shows them, so a user never
/// reads the source to find one.
const KEY_HINTS: &str = "q quit · a abilities · c layout · s slot";

/// The rows Squire's name takes when it is drawn large.
pub const LOGO_ROWS: u16 = 5;

/// Spare rows the logo needs beyond its own height. A logo pressed against
/// the cards is not room to spare.
const LOGO_AIR: u16 = 2;

/// Works out what to draw. It takes the size of the terminal, the party from
/// the last read, and the toggles, and it returns a [`Plan`].
pub fn plan(size: Size, party: &Party, caption: &Caption, toggles: Toggles) -> Plan {
    let liveness = liveness(party);
    let dim = liveness == Liveness::Lost;
    let grid =
        fit_grid(size, party, toggles).map(|fit| fit.grid(&party.characters, toggles.axis, dim));
    let show_logo = logo_fits(size, grid.as_ref());
    let left = status_text(caption, party, liveness, grid.as_ref());

    Plan {
        header: fit(&header_text(caption), size.cols),
        status: two_up(&left, KEY_HINTS, size.cols),
        grid,
        show_logo,
        dim,
        liveness,
    }
}

/// Splits the session's not-found state in two. A run that never found a
/// party waits. A run that found one and lost it is lost.
fn liveness(party: &Party) -> Liveness {
    match party.state {
        PartyState::Live => Liveness::Live,
        PartyState::Partial => Liveness::Partial,
        PartyState::NotFound if party.characters.is_empty() => Liveness::Waiting,
        PartyState::NotFound => Liveness::Lost,
    }
}

// --- The grid -------------------------------------------------------------

/// The widest each field gets across the whole party.
///
/// Every card uses these widths. A short name never gives its owner a
/// different layout from the rest of the party.
struct MaxFieldWidths {
    name: u16,
    class: u16,
    hit_points: u16,
    armor: u16,
    abilities: u16,
}

fn max_field_widths(party: &[Character]) -> MaxFieldWidths {
    let widest = |f: fn(&Character) -> String| -> u16 {
        party.iter().map(|c| width(&f(c))).max().unwrap_or(0)
    };
    MaxFieldWidths {
        name: widest(|c| c.name.clone()),
        class: widest(class_text),
        hit_points: widest(hit_points_text),
        armor: widest(armor_text),
        abilities: widest(abilities_text),
    }
}

impl MaxFieldWidths {
    /// The narrowest card worth drawing. It holds a whole hit point line.
    /// A name ends in an ellipsis when it is cut, so a name sets no floor.
    fn floor(&self) -> u16 {
        self.hit_points.max(1)
    }
}

/// Which fields share a line at a given card width, and how wide the hit
/// point bar is.
///
/// Every value comes from the column width and the party-wide
/// [`MaxFieldWidths`]. No character changes them, so one [`CardShape`] covers
/// the whole grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardShape {
    /// The class and level fit beside the name.
    pub class_inline: bool,
    /// The armor class fits beside the hit points.
    pub armor_inline: bool,
    /// Cells for the hit point bar. Zero when there is no room for a bar a
    /// reader can judge at a glance.
    pub bar: u16,
}

impl CardShape {
    fn new(width: u16, max_widths: &MaxFieldWidths) -> CardShape {
        let class_inline = width
            >= max_widths
                .name
                .saturating_add(NAME_GAP)
                .saturating_add(max_widths.class);
        // The bar takes what the hit point line has left after the armor class.
        let room = width.saturating_sub(max_widths.hit_points + 1);
        let armor_inline = room >= max_widths.armor.saturating_add(NAME_GAP);
        let bar = if armor_inline {
            room.saturating_sub(max_widths.armor + CARD_PAD)
                .min(BAR_MAX)
        } else {
            room.min(BAR_MAX)
        };
        let bar = if bar < BAR_MIN { 0 } else { bar };
        CardShape {
            class_inline,
            armor_inline,
            bar,
        }
    }
}

/// A grid that fits, before its cards carry any text.
///
/// The text waits until one of these wins, because [`fit_grid`] tries several
/// values of `across` and throws most of them away.
struct Fit {
    across: u16,
    down: u16,
    widths: Vec<u16>,
    card_rows: u16,
    shape: CardShape,
    /// One per character, in marching order.
    fields: Vec<Vec<Field>>,
}

impl Fit {
    /// Fits every line's text to the width of the column its card lands in,
    /// and gives each one its meaning.
    fn grid(self, party: &[Character], axis: Axis, dim: bool) -> Grid {
        let cards = self
            .fields
            .iter()
            .zip(party)
            .enumerate()
            .map(|(who, (fields, c))| {
                // The leftmost columns are the ones that took the spare cell.
                let width = self.widths[column_of(who, self.across, self.down, axis)];
                Card {
                    lines: fields
                        .iter()
                        .map(|&field| Line {
                            text: line_text(c, field, self.shape, width),
                            tint: if dim { Tint::Faint } else { tint(c, field) },
                            field,
                        })
                        .collect(),
                }
            })
            .collect();
        Grid {
            across: self.across,
            down: self.down,
            widths: self.widths,
            card_rows: self.card_rows,
            cards,
            shape: self.shape,
            axis,
        }
    }
}

/// Decides how many cards fit, and how they are arranged.
///
/// `Horizontal` fills a row first and packs in as many cards across as fit.
/// `Vertical` fills a column first and packs in as many down as fit.
///
/// Rows are a hard limit either way. A screen too short for its cards falls
/// back to fewer down and more across.
///
/// A party that does not divide evenly leaves its last row or column short.
fn fit_grid(size: Size, party: &Party, toggles: Toggles) -> Option<Fit> {
    let n = u16::try_from(party.characters.len()).ok()?;
    if n == 0 {
        return None;
    }
    let max_field_widths = max_field_widths(&party.characters);
    let body = size.rows.checked_sub(EDGE_ROWS)?;

    let build = |across: u16| -> Option<Fit> {
        let down = n.div_ceil(across);
        let text = size
            .cols
            .checked_sub(across + 1)?
            .checked_div(across)?
            .checked_sub(CARD_PAD)?;
        let rows = body.checked_sub(down + 1)?.checked_div(down)?;
        if text < max_field_widths.floor() || rows < CARD_MIN_ROWS {
            return None;
        }

        // Every card is shaped from the narrowest column, so the odd spare
        // cell a division leaves over is padding and never a field.
        let spare = size.cols - (across + 1) - across * (text + CARD_PAD);
        let widths: Vec<u16> = (0..across).map(|i| text + u16::from(i < spare)).collect();
        let shape = CardShape::new(text, &max_field_widths);

        let wanted: Vec<Vec<Field>> = party
            .characters
            .iter()
            .map(|c| card_fields(c, text, &max_field_widths, shape, toggles))
            .collect();
        let tallest = wanted.iter().map(Vec::len).max().unwrap_or(0);
        let card_rows = rows.min(u16::try_from(tallest).unwrap_or(u16::MAX));
        let fields = wanted.into_iter().map(|f| cut(f, card_rows)).collect();

        Some(Fit {
            across,
            down,
            widths,
            card_rows,
            shape,
            fields,
        })
    };

    match toggles.axis {
        // Widest first: the largest `across` that fits.
        Axis::Horizontal => (1..=n).rev().find_map(build),
        // Narrowest first: the smallest `across`, which leaves the most rows.
        Axis::Vertical => (1..=n).find_map(build),
    }
}

/// The drop order, first to last. The first entry is the last field to leave
/// a card, and the last entry is the first to go.
///
/// Conditions outrank the class and the armor class, because a silent
/// `poisoned` is the one thing a player must not miss. The abilities sit at
/// the end because a key turns them on, so they never cost a condition.
const DROP_ORDER: [Field; 6] = [
    Field::Name,
    Field::HitPoints,
    Field::Condition(0),
    Field::Class,
    Field::Armor,
    Field::Abilities,
];

impl Field {
    /// The position of this field in [`DROP_ORDER`]. A lower number survives
    /// longer.
    ///
    /// Compared by discriminant, so that every `Condition` shares the one
    /// rank the array holds whatever index it carries.
    fn rank(self) -> usize {
        let want = std::mem::discriminant(&self);
        DROP_ORDER
            .iter()
            .position(|f| std::mem::discriminant(f) == want)
            .expect("every Field is in DROP_ORDER")
    }
}

/// The fields one card wants, in reading order, before any budget applies.
fn card_fields(
    c: &Character,
    width: u16,
    max_widths: &MaxFieldWidths,
    shape: CardShape,
    toggles: Toggles,
) -> Vec<Field> {
    let mut fields = vec![Field::Name, Field::HitPoints];

    let CardShape {
        class_inline,
        armor_inline,
        ..
    } = shape;

    for i in 0..conditions(c).len() {
        fields.push(Field::Condition(i));
    }
    if !class_inline {
        fields.push(Field::Class);
    }
    if !armor_inline {
        fields.push(Field::Armor);
    }
    // All six or none. One score is not worth a line.
    if toggles.abilities && max_widths.abilities <= width {
        fields.push(Field::Abilities);
    }
    fields
}

/// `fields` cut to `budget` lines, in reading order.
///
/// [`DROP_ORDER`] decides what leaves. The sort must stay stable: every
/// condition shares one rank, and they keep the order they were pushed in.
fn cut(fields: Vec<Field>, budget: u16) -> Vec<Field> {
    let mut order: Vec<usize> = (0..fields.len()).collect();
    order.sort_by_key(|&i| fields[i].rank());
    order.truncate(usize::from(budget));
    order.sort_unstable();
    order.into_iter().map(|i| fields[i]).collect()
}

// --- The words on a card --------------------------------------------------

/// What one field of a character means: the hit point bands in thirds rather
/// than a gradient, and any condition other than `okay` as critical.
fn tint(c: &Character, field: Field) -> Tint {
    match field {
        Field::Name => Tint::Heading,
        Field::HitPoints => {
            if c.hit_points_maximum == 0 || c.hit_points_current <= 0 {
                return Tint::Critical;
            }
            let left = f64::from(c.hit_points_current) / f64::from(c.hit_points_maximum);
            if left >= 0.66 {
                Tint::Good
            } else if left >= 0.33 {
                Tint::Wounded
            } else {
                Tint::Critical
            }
        }
        Field::Condition(i) => match conditions(c).get(i) {
            Some(word) if word.eq_ignore_ascii_case("okay") => Tint::Good,
            Some(_) => Tint::Critical,
            None => Tint::Body,
        },
        Field::Class | Field::Armor | Field::Abilities => Tint::Body,
    }
}

/// Every condition on the character, one item per line.
fn conditions(c: &Character) -> Vec<String> {
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

/// The six ability scores in the standard D&D order. The exceptional
/// strength score goes in parentheses when the character has one.
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

/// A hit point bar. Full blocks for the hit points that are left, and light
/// blocks for the ones that are gone.
fn bar_text(c: &Character, width: u16) -> String {
    let width = usize::from(width);
    if width == 0 || c.hit_points_maximum == 0 {
        return String::new();
    }
    let top = f64::from(c.hit_points_maximum);
    let current = f64::from(c.hit_points_current).max(0.0);
    let mut filled = ((width as f64) * current / top).round() as usize;
    filled = filled.min(width);
    // A living character always keeps one block, so that hurt and down never
    // look the same.
    if c.hit_points_current > 0 {
        filled = filled.max(1);
    }
    "█".repeat(filled) + &"░".repeat(width - filled)
}

/// One planned line as text, exactly `width` cells wide.
///
/// `shape` is the grid's, not the character's. It says which fields share a
/// line at this width.
fn line_text(c: &Character, line: Field, shape: CardShape, width: u16) -> String {
    match line {
        Field::Name => {
            if shape.class_inline {
                two_up(&c.name, &class_text(c), width)
            } else {
                fit(&c.name, width)
            }
        }
        Field::HitPoints => {
            let hp = hit_points_text(c);
            let drawn = bar_text(c, shape.bar);
            let left = if drawn.is_empty() {
                hp
            } else {
                format!("{hp} {drawn}")
            };
            if shape.armor_inline {
                two_up(&left, &armor_text(c), width)
            } else {
                fit(&left, width)
            }
        }
        Field::Condition(i) => fit(conditions(c).get(i).map_or("", |s| s), width),
        // The own-line form has no separator. A card narrow enough to need
        // this line is narrow enough to want the four cells back.
        Field::Class => fit(&format!("{} {}", class_name(c), c.level), width),
        Field::Armor => fit(&armor_text(c), width),
        Field::Abilities => fit(&abilities_text(c), width),
    }
}

// --- The header and the status line ---------------------------------------

/// The top line: the program, the game, and the save slot when one is picked.
fn header_text(caption: &Caption) -> String {
    match caption.slot {
        Some(letter) => format!("gbs — {} · slot {letter}", caption.game),
        None => format!("gbs — {}", caption.game),
    }
}

/// What the status line says about the party, before the keys are added.
fn status_text(
    caption: &Caption,
    party: &Party,
    liveness: Liveness,
    grid: Option<&Grid>,
) -> String {
    let state = match liveness {
        Liveness::Live => format!("live · {}", party.characters.len()),
        Liveness::Partial => format!("partial · {} shown", party.characters.len()),
        Liveness::Lost => "anchor lost, rescanning".to_string(),
        Liveness::Waiting => "no party yet".to_string(),
    };
    let mut text = format!("1 {} · {state}", caption.panel);
    if let Some(grid) = grid {
        let axis = match grid.axis {
            Axis::Horizontal => "horizontal",
            Axis::Vertical => "vertical",
        };
        text.push_str(&format!(" · {axis}"));
    }
    if let Some(note) = &caption.note {
        text.push_str(" · ");
        text.push_str(note);
    }
    text
}

// --- The logo -------------------------------------------------------------

/// Squire's name in block letters, one string per row.
pub fn logo() -> Vec<String> {
    const FONT: [(char, [&str; LOGO_ROWS as usize]); 6] = [
        ('S', ["█████", "█    ", "█████", "    █", "█████"]),
        ('Q', ["█████", "█   █", "█ █ █", "█  ██", "█████"]),
        ('U', ["█   █", "█   █", "█   █", "█   █", "█████"]),
        ('I', ["█████", "  █  ", "  █  ", "  █  ", "█████"]),
        ('R', ["█████", "█   █", "█████", "█  █ ", "█   █"]),
        ('E', ["█████", "█    ", "█████", "█    ", "█████"]),
    ];
    (0..LOGO_ROWS as usize)
        .map(|row| {
            FONT.iter()
                .map(|(_, glyph)| glyph[row])
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}

/// Whether there is room to spare for the logo.
fn logo_fits(size: Size, grid: Option<&Grid>) -> bool {
    let Some(grid) = grid else {
        return false;
    };
    let block = logo();
    let widest = block.iter().map(|l| width(l)).max().unwrap_or(0);
    let spare = size
        .rows
        .saturating_sub(EDGE_ROWS)
        .saturating_sub(grid.rows());
    widest <= size.cols && spare >= LOGO_ROWS + LOGO_AIR
}

// --- Fitting text ---------------------------------------------------------

/// The cells `text` takes, one per character.
fn width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// `text`, padded or cut to exactly `width` cells.
///
/// Cut text ends in an ellipsis, so a reader never mistakes a cut name for a
/// short one. Under two cells there is no room for the ellipsis, and the text
/// is cut without one.
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
/// When both do not fit, the right one goes. It is always the less important
/// of the two, and half of each is worse than all of one.
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
