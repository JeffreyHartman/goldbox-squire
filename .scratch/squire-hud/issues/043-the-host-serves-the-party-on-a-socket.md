# 043 — The host serves the party on a socket

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 037, 041, 042

Renamed and rewritten 2026-08-26. The old body had gbs keeping the emulator
while the HUD moved to a second window, and that cannot work: a sibling of
DOSBox may not read it. See [ADR 0005](../../../docs/adr/0005-one-host-reads-many-views-draw.md),
which settles the shape. The file keeps its number because 042's answer and
the map both cite it.

## What to build

`gbs` becomes the **host**. It launches DOSBox and reads it exactly as it does
today, and it also listens on a unix socket and hands the party out as data.
Nothing opens a window in this ticket.

The socket is `$XDG_RUNTIME_DIR/goldbox-squire/<pid>.sock`, one per run. The
host removes it on the way out.

The host sends the party and the watch's notices. It receives the user's
decisions: quit, and a slot repick that has already been resolved to a slot
letter and a list of names. Those are exactly `watch::Interrupt`, so the host
is a `Screen` and a `Keys` and the loop is untouched.

A view connecting late gets told what it needs to draw straight away, and to
run a repick: which game, which slot, and the install's save folder. A view
that disconnects is dropped and the run carries on.

`--plain` opens no socket.

## Acceptance criteria

- [ ] `gbs` listens on `$XDG_RUNTIME_DIR/goldbox-squire/<pid>.sock`, and the
      file is gone after the run
- [ ] Anything that connects receives the party as it changes, and the
      watch's notices
- [ ] A client connecting after the party was found is caught up at once,
      without waiting for the next poll
- [ ] A client that connects, disconnects and connects again does not disturb
      the run
- [ ] Quit and repick sent by a client arrive at the loop as `Interrupt`
- [ ] Every one of the above is tested with no terminal involved
- [ ] `--plain` creates no socket
