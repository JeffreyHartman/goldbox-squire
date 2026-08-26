# 034 — Mockups: the party at four sizes

Type: `wayfinder:prototype`
Status: claimed
Triage: `ready-for-human`
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
- [ ] Jeff has looked at them and either confirmed the drop order or changed it
- [ ] Whatever he decides is recorded on this ticket, because 036 reads it

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
