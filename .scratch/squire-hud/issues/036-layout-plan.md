# 036 — The layout plan: size and party in, what to show out

Type: `wayfinder:task` (AFK)
Status: open
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

- [ ] A pure function from rows, columns and a party to a description of what
      is shown
- [ ] Tests assert the drop order: which fields survive at which widths, in the
      order 034 settled
- [ ] Tests assert that the wordmark appears only when roomy
- [ ] Tests assert that a lost anchor is expressed in the description, so 038
      has something to draw
- [ ] Tests cover a hostile size, an enormous size, a zero-character party and
      a partial party
- [ ] No constant in the module traces to any particular display
- [ ] The module has no dependency on a terminal library
