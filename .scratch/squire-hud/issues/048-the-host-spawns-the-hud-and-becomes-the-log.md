# 048 — The host spawns the HUD view, and its own window becomes the log

Type: `wayfinder:task` (AFK)
Status: open
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

- [ ] `gbs` opens the HUD in a new window at the remembered size, with the
      app id from 046
- [ ] The host's own window shows rescans, anchor losses and emulator output
- [ ] A user entry from 042 is honoured over the compiled-in default
- [ ] An unrecognised terminal is still launched, with one message saying the
      size could not be set
- [ ] Closing the HUD window does not end the run
- [ ] Quitting the host closes the HUD window and leaves the emulator running
- [ ] `--plain` still runs in place and spawns nothing
