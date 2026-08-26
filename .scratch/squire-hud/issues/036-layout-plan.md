# 036 — The layout plan: size and party in, what to show out

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: 034

## What to build

The seam this effort agreed to write tests at, before any drawing exists.

A pure function: rows, columns and a party go in, a description of what is
shown and where comes out. No terminal, no ratatui, no input, no clock. It can
be called in a test with any size at all, including sizes no real terminal
would produce, and it answers instantly.

Every decision this effort made about what a HUD shows lives here rather than
scattered through drawing code. That is what makes them testable, and it is why
the drop order stops being something an agent remembers and starts being
something the build checks.

The rules answer "does this fit". They never name a size, a monitor, or a
layout mode. If a reviewer can find a constant in this module that came from
somebody's screen, the module is wrong.

Read 034's recorded answer for the drop order and the roomy threshold before
writing the tests. The map's version is the pre-mockup draft.

## Acceptance criteria

- [x] A pure function from rows, columns and a party to a description of what
      is shown
- [x] Tests assert the drop order: which fields survive at which widths, in the
      order 034 settled
- [x] Tests assert that the wordmark appears only when roomy
- [x] Tests assert that a lost anchor is expressed in the description, so 038
      has something to draw
- [x] Tests cover a hostile size, an enormous size, a zero-character party and
      a partial party
- [x] No constant in the module traces to any particular display
- [x] The module has no dependency on a terminal library

## Answer

`squire-cli/src/layout.rs`, with `squire-cli/tests/layout.rs` beside it.

`plan(size, party, sitting, toggles) -> Plan`. Nothing in the module imports a
terminal library, and nothing in it can.

**The shape of the answer.** A `Plan` carries the header, the status line, an
optional `Grid`, whether the wordmark is drawn, and whether the party dims. The
`Grid` says how many cards across and down, how wide each column is, how many
lines a card gets, and which lines each card has, as a `CardLine` per line.
Presence is an enum rather than text, so the drop-order tests read as the drop
order rather than as string matching.

`line_text` turns one `CardLine` into the text for a width. It lives in the
same module rather than in the drawing code so that truncation and alignment
are decided once and tested without a terminal. Deciding *what* is on a line
and deciding *how wide the ellipsis goes* are the same decision at a hostile
size, and splitting them would have put half of it out of reach of the tests.

**Four things worth knowing.**

1. The number of cards across is chosen by the miss from the width the card
   wants, exactly as `cards.py` does it, and the seven sizes 034 recorded are
   pinned as a test. A key override is honoured when it fits and ignored when
   it does not, because a key that empties the screen is a trap.
2. Only arrangements that divide the party evenly are offered. A ragged last
   row would make one card differ from its neighbours, which the party-wide
   shape rule already forbids.
3. Every card is shaped from the narrowest column, so the spare cell a division
   leaves over is always padding and never a field.
4. `Liveness` has four values, not three. `Waiting` is a run that has not found
   a party yet and `Lost` is one that had one and lost the anchor. The session
   returns an empty character list either way, so without the split the first
   frame of every run would have said "anchor lost". Only `Lost` dims.

**The one judgement constant** is `CARD_AIR`, four cells, carried over from the
mockups with its argument attached. Everything else comes from the party or
from the game data. No constant names a size, and the sizes that do appear in
the tests are 034's, where they belong.
