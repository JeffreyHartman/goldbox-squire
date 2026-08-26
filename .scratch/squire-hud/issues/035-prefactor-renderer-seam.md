# 035 — Prefactor: the watch loop gets a renderer seam

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: none

## What to build

Nothing changes for the user. `gbs` runs exactly as it does today, prints the
same table, answers Enter the same way, and every existing test passes
unaltered.

What changes is that the thing which draws is no longer welded to the loop
which polls. Today one long function in the binary polls the session, decides
whether the party was ever found, clears the screen, prints, and listens for a
keypress, all in one place, with the escape sequences written inline. A second
way of drawing cannot be added to that without copying it.

Separate the loop from the drawing, so that the printed table is one
implementation of drawing and the HUD can be another. Make the change easy
first; the easy change is 037.

The loop also lives in the binary rather than the library, which is why none of
it is tested today. Moving it into `squire-cli`'s library is part of this.

## Acceptance criteria

- [ ] `gbs` behaves identically: same output, same keys, same messages, same
      exit conditions
- [ ] The polling loop is in the library, not the binary, and has tests
- [ ] Drawing is reached through one seam that a second implementation can
      satisfy without touching the loop
- [ ] No escape sequence is written inline in the loop
- [ ] The full test suite passes with no test changed to accommodate the move
