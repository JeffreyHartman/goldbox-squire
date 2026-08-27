# 043 — The host serves the party on a socket

Type: `wayfinder:task` (AFK)
Status: resolved
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

- [x] The host listens on `$XDG_RUNTIME_DIR/goldbox-squire/<pid>.sock`, and
      the file is gone after the run
- [x] Anything that connects receives the party as it changes, and the
      watch's notices
- [x] A client connecting after the party was found is caught up at once,
      without waiting for the next poll
- [x] A client that connects, disconnects and connects again does not disturb
      the run
- [x] Quit and repick sent by a client arrive at the loop as `Interrupt`
- [x] Every one of the above is tested with no terminal involved
- [ ] `--plain` creates no socket — moved to 048, which is where `main` first
      builds a host. Building one here would mean a run with a socket, a log
      and no window at all, or a second `Screen` that fans out to both and is
      deleted again one ticket later.

## Answer

`squire-cli/src/host.rs` is the host and `squire-cli/src/wire.rs` is what
crosses the socket. Seventeen tests in `squire-cli/tests/host.rs`, none of
which opens a terminal or an emulator.

The host is the two watch-loop seams wearing a socket. `HostScreen` is the
`Screen` and writes the party out; `HostKeys` is the `Keys` and spends the
pause listening to the views. `watch::watch` is unchanged, which is what 035
was for.

**The wire is one JSON object per line, both ways.** A view can be anything
that opens a socket and splits on a newline, and a person can watch a run with
`socat`. The party's shape is `wire`'s, and `--json` now prints the same
shape: one party format in the program rather than two that can drift.

**A view is caught up on connect, not at the next poll.** It gets a hello
naming the game, the slot and the save folder, then the last party and the
last notice. That is the state, not a transcript: a window opened an hour in
wants what is true now.

**Nothing a view sends can end a sitting.** An unparseable line is skipped,
because a view is another program and 011 forbids the tool taking the game
down with it.
