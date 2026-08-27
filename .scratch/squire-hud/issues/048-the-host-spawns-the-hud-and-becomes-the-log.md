# 048 — The host spawns the HUD view, and its own window becomes the log

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: 046, 047

## What to build

The thing the effort was for. `gbs` opens the HUD in a window of its own, at
the size 039 remembered, carrying the app id from 046, so that one compositor
rule places it beside the game. The window the user typed in does not go back
to a prompt: it becomes the log.

The log is worth having rather than merely occupied. It is the thing a user
cannot see at all today: when the anchor died, when a rescan ran, what the
emulator printed.

An unrecognised terminal is still launched. Squire says it could not set the
size and carries on. There is no second code path for the unknown case,
because two routes through the same feature is how one of them rots.

## Acceptance criteria

- [x] `gbs` opens the HUD in a new window at the remembered size, with the
      app id from 046
- [x] The host's own window shows rescans, anchor losses and emulator output
- [x] A user entry from 042 is honoured over the compiled-in default
- [x] An unrecognised terminal is still launched, with one message saying the
      size could not be set
- [x] Closing the HUD window does not end the run
- [x] Quitting the host closes the HUD window and leaves the emulator running
- [x] `--plain` still runs in place and spawns nothing

## Note

`--plain` creating no socket came from 043, which is where a host is first
built. It is satisfied here: `Host::start` is called only on the windowed
path.

One thing changed that the criteria did not ask for. The emulator's output
used to go to a file beside the config, because the HUD owned this terminal.
On the windowed path the HUD no longer does, so DOSBox writes straight into
the log window, which is what "the host's own window shows emulator output"
means. `--plain` still redirects to the file, because a printed table and an
emulator writing to one terminal is a mess.

## Answer

`squire-cli/src/spawn.rs` chooses the terminal and builds the command line;
`main.rs` starts the host, opens the window, and runs the loop. Ten tests in
`squire-cli/tests/spawn.rs`, and none opens a window.

**Which terminal**, in order: `--terminal`, then `TERMINAL`, then the first
terminal Squire knows that is on PATH. A terminal the user named is used
whether Squire knows it or not, because naming one is the whole answer to
"which terminal".

**The window opens after the socket exists**, so a view never races the host
and finds nothing to connect to.

**A first run gets eighty by twenty-four**, which is what a terminal opens at
when nobody says otherwise, and the next run uses whatever the window was
dragged to. The view is the process that remembers it now, because the view
is the process that knows it.
