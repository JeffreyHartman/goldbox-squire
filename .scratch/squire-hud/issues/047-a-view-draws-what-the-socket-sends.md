# 047 — A view draws what the socket sends

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: 043

## What to build

A second way to run `gbs`: as a **view**. It takes a view kind and a socket
path, connects, and draws the party it is sent using the HUD built in 037.
Nothing spawns it yet. You open a second terminal and run it by hand, which is
also how it is demonstrated.

The view never reads the emulator and never learns an address. It draws, and
it reflows on a resize exactly as the HUD does today.

Keys are not wired back in this ticket. That is 049.

## Acceptance criteria

- [x] `gbs` run as a view against a live host's socket draws the party, and
      keeps drawing as the numbers change
- [x] The caption names the game and the slot the host is watching
- [x] The watch's notices reach the view
- [x] Resizing the view's window reflows it
- [x] The host quitting ends the view without an error message
- [x] A socket path that is not there says so plainly and exits non-zero

## Answer

`squire-cli/src/view.rs`, reached by `gbs --view hud --socket <PATH>`.
Fifteen tests in `squire-cli/tests/view.rs` cover everything below the
terminal: how a view is asked for, and what it makes of every line it can be
sent.

`Hud::pump_resizes` is new and small: a view waits on a socket rather than on
a keyboard, so nothing else would notice a drag until the next poll.

A message kind this build does not know is skipped rather than fatal, which is
what lets the wire grow a message without every view being rebuilt for it.
