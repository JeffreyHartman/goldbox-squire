# 038 — Stale numbers dim

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 037

## What to build

Ticket 007 settled that Squire re-reads the name before every read, rescans
when that fails, and never shows stale numbers as live. A reprinting table
satisfied that by not printing. A HUD cannot: it is a persistent screen, and a
screen that keeps showing the last known hit points while the anchor is gone is
lying.

When the anchor is lost, the party block dims and the status line says why. The
numbers stay visible while dimmed, because a rescan takes a moment and the last
known values are still worth something. They are just visibly not live.

Dimming rather than a status word alone, because a HUD is read from the corner
of the eye while looking at the game, and a word is easy to miss there.

Demoable: quit to the game's main menu with `gbs` running, and watch the HUD go
grey rather than keep asserting hit points that no longer exist.

## Acceptance criteria

- [ ] A lost anchor dims the party block and states the reason on the status
      line
- [ ] A partial party is distinguishable from a lost one
- [ ] Recovering the anchor restores full brightness with no flicker
- [ ] The dim state is decided in 036's layout plan, not in drawing code, and
      is covered by its tests
- [ ] Dimming survives at a hostile size, where the status line may be the
      only thing that fits
