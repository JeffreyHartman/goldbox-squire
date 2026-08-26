# 034 — Mockups: the party at four sizes

Type: `wayfinder:prototype`
Status: resolved
Triage: `ready-for-agent`
Blocked by: 033

## What to build

A mockup Jeff can open beside a real terminal and compare, showing the party
panel at four sizes. What it delivers is two reviewed decisions: the drop order
and the roomy threshold. Ticket 036 encodes both, so they should be looked at
before they become tests.

The medium is a character grid, not a web page pretending to be one. One
monospace block per mockup, sized in real terminal cells, drawn with the box
characters ratatui would emit. A mockup that can render half a cell or pick its
own line height is a mockup that lies about the constraint.

Starting sizes, roughly 110x50, 160x42, 160x14 and 40x20. These are scratch
values, chosen to span tall, roomy, short-and-wide, and deliberately hostile.
They are not requirements and no code should ever contain them. The hostile one
matters most: a layout that survives it is a layout with no assumptions in it.

Party only. No map, no journal, no combat: nothing is mocked up for data Squire
cannot yet read, which is the mistake that produced the spike's left menu.

This ticket is allowed to contradict the map's drop order. That is the point of
building it before 036 rather than after.

## Acceptance criteria

- [x] Four mockups, each a fixed character grid at its stated size
- [x] Party data only, with plausible six-character parties including a wounded
      one and one with a status
- [x] Each mockup shows which fields survived at that size, and the roomy one
      shows the wordmark
- [x] Jeff has looked at them and either confirmed the drop order or changed it
- [x] Whatever he decides is recorded on this ticket, because 036 reads it

## First pass: tables. Superseded.

The first pass drew one table at four widths. Jeff's answer was that a table is
the wrong unit: the thing on screen is a **card** per character, and the layout
is how those cards are arranged, not how one table is squeezed. GBC does the
same, and his own mockups make it plain. The tables are kept under `table-*.txt`
as a fallback and nothing more.

He also asked for sidecar widths the first pass did not draw, and pointed out
that a character can carry several conditions at once, which a single status
column cannot hold.

## Built, and waiting on Jeff

`.scratch/squire-hud/mockups/`. Five grids, not four: the hostile size is drawn
twice, because at 40 columns you can have whole names or one more field and not
both, and that is a choice rather than a rule.

    cat .scratch/squire-hud/mockups/README.md

`mock.py` draws all five. Changing the drop order is one edit to `COLUMNS` and a
rerun.

Six questions are written up in the README: the drop order itself, whole names
against one more field, the one-glyph status, the roomy threshold, centring
against stretching, and whether a character may take two rows when rows are
plentiful and columns are not. The fourth is the one 036 turns into assertions.

**The answers go here.** 036 reads this heading, not the map, and it stays
`ready-for-human` until they are written down.

## Second pass: cards

`cards.py` and `cards-*.txt` in the same folder. Eight grids.

**The card.** Up to five lines: name with class and level, hit points with a
bar and armour class, one line per condition, then ability scores. It drops
lines from the bottom as it shrinks. Whether class shares the name line, and
whether the ability line is long or short, is decided once for the whole party
so that no card is shaped differently from its neighbours.

**The layout.** Six cards, one across, two, three, or six. The rule picks the
number across that gets each card closest to the width it wants to hold
everything, and rows are a hard limit. There is no strip mode and no sidecar
mode; those are what the ends of the range look like.

**Conditions.** Squire reads one status byte today, so the mockups show one
condition per character. The card holds a list, one line each, sitting above
the ability line so that when the effects read lands the ability scores are
what falls off a crowded card rather than the thing hurting you. The README
shows what a card with four conditions looks like.

**Sidecar widths.** 50, 60 and 80 are drawn. 60 is the first width where the
full ability line fits, and my suggestion for the size Squire remembers on a
first run.

Four questions are left in the README: whether cards are right at all, the
sidecar width, six across against three across two down at 160x14, and whether
to keep the wordmark.

**The answers still go under this heading.** 036 reads it, not the map.

## Answered by Jeff, 2026-08-25

- **Cards, not tables.** Confirmed. The tables stay under `table-*.txt` as a
  fallback only.
- **The sidecar is 60 columns.** That is the width Squire starts at on a first
  run, before it has any remembered geometry.
- **Two columns of cards is a wanted answer, not an accident.** 80x40 and
  110x50 both land there and both are right.
- **160x14 keeps both answers.** Three across is what the rule picks. Six
  across is available because people should have the choice.
- **Ability scores are off by default and are a key, not a width rule.** They
  make the cards look busy, and they barely change during play. There is no
  half measure: no card ever shows one score. When the key is pressed the whole
  party gets one line of six numbers, slashed, no labels, in the order every
  Gold Box screen prints them. Percentile strength goes in brackets, `18(72)`,
  because it has to survive the slashes.

036 must take the toggle as an input rather than deciding it. The layout plan
answers "does this fit". The toggle answers "did the user ask for it". Those
are different questions and mixing them puts a preference inside a rule.

Still open, and none of it blocks 036: whether the wordmark survives, what else
gets a key when THAC0 and encumbrance land, and whether the tables go.
