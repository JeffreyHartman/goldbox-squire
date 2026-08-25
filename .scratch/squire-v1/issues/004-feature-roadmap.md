# 004 — The feature roadmap and its order

Type: `wayfinder:grilling` (HITL)
Status: open, narrowed 2026-08-25
Blocked by: none (003 closed 2026-08-22)

## Question

Originally: enumerate every feature GBS might have, and put them in an order.

The enumeration is done and it moved out of this ticket. It lives at
[`docs/roadmap.md`](../../../docs/roadmap.md), which is a standing document
edited for as long as the program is worked on. A ticket closes; a roadmap does
not, so the roadmap was the wrong thing to hold here.

What is still a decision, and still open: **which three features come after v1,
and why does each earn its place ahead of the others?**

The roadmap's "Next, and cheapest" section is a first cut ordered by build cost,
not by value. Cost is the wrong sole axis. Grill the order.

## Constraints already settled

- The monk class is excluded. `VK.exe` is excluded. Reading DOSBox under Wine is
  excluded.
- The HUD is not pinned. Several layouts switched by keyboard shortcut, the user
  places the window. See the roadmap's HUD section.
- Everything read-only is cheaper than anything that writes to memory, because
  the write path and the combat check do not exist yet.

## Answer

<!-- filled on resolution -->
