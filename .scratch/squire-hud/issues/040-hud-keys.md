# 040 — Keys: quit, character selection, and the slot repick

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 037

## What to build

The keyboard contract for the HUD: quit, and move the highlight through the
party.

The part with the trap in it is the slot repick. Today, pressing Enter during
the watch returns to the slot question and the watch resumes against the new
slot's names. That path polls stdin for readiness and then reads a line. A HUD
puts the terminal in raw mode, where reading a line does not work and the poll
is no longer the right ear. The repick has to keep working, which means it
either suspends the HUD to ask its question or asks it on screen.

This is its own ticket rather than a footnote on 037 because it is the one
place where the existing behaviour quietly breaks, and it would be easy to ship
a HUD that lost a working feature without anyone noticing.

The keys follow the Gold Box idiom the spike found: number keys jump straight
to a destination, which costs nothing to reserve now and means the second panel
does not need a menu built for it.

## Acceptance criteria

- [ ] A documented key quits, and the terminal is restored
- [ ] The highlight moves through the party and does not run off either end
- [ ] The slot repick still works, and the wizard's question is readable while
      the terminal is in raw mode
- [ ] Repicking retargets the session and the HUD shows the new party
- [ ] Number keys are reserved for panels, even though only one panel exists
- [ ] The keys in use are visible somewhere without reading the source
