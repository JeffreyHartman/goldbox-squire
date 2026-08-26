# 038 — Stale numbers dim

Type: `wayfinder:task` (AFK)
Status: resolved
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

- [x] A lost anchor dims the party block and states the reason on the status
      line
- [x] A partial party is distinguishable from a lost one
- [x] Recovering the anchor restores full brightness with no flicker
- [x] The dim state is decided in 036's layout plan, not in drawing code, and
      is covered by its tests
- [x] Dimming survives at a hostile size, where the status line may be the
      only thing that fits

## Answer

Folded into 036 and 037, which is where the ticket asked for it.

`Liveness` in the layout plan has four values rather than three. `Live` and
`Partial` are the session's own states. The other two split what the session
reports as not-found: `Waiting` is a run that never found a party, and `Lost`
is one that had a party and lost the anchor. Only `Lost` dims. Without the
split, the first frame of every run would have said "anchor lost".

The session returns an empty character list when the anchor is gone, so the
HUD keeps the last characters it read and hands them back with the current
state. That is what puts numbers on the dimmed screen at all.

Dimming greys the whole party block and drops the colour coding, including the
red of a bad condition and the highlight. A dimmed red still reads as an alarm,
and the point of dimming is that none of it is live. The status line turns red
and says "anchor lost, rescanning", and it survives at a size where it is the
only row that fits.

A partial party keeps its colours and its highlight, and the status line says
how many are shown. It is not the same thing as a lost one and does not look
like one.
