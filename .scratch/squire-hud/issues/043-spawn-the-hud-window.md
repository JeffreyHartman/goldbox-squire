# 043 — `gbs` spawns the HUD in its own window and the parent becomes the log

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 037, 041, 042

## What to build

`gbs` opens the HUD in a second terminal window, at the size it remembered,
carrying the app-id a compositor rule can match. The window the user typed in
does not go back to a prompt: it keeps the emulator handle and becomes the log.

Keeping the parent matters for two reasons. It already owns the emulator child,
and dropping that handle is exactly what ticket 011 warned about. And the log
is the thing a user cannot see at all today: when the anchor died, when a
rescan ran, what the emulator printed. That makes the kept window worth having
rather than merely occupied. Launching something that opens a window and holding
the terminal as its log is ordinary behaviour on Linux, so it needs no
explaining.

An unrecognised terminal is still launched. Squire says it cannot set the size
and carries on. There is no second code path for the unknown case, because two
routes through the same feature is how one of them rots.

## Acceptance criteria

- [ ] `gbs` opens the HUD in a new window at the remembered size, with the
      app-id from 041
- [ ] The launching terminal keeps the emulator handle and shows a log of
      rescans, anchor losses and emulator output
- [ ] Quitting the HUD ends the run cleanly, and closing the parent does not
      orphan a window
- [ ] An unrecognised terminal is still launched, with one message saying the
      size could not be set
- [ ] A user entry from 042 is honoured over the compiled-in default
- [ ] `--plain` still runs in place and spawns nothing
