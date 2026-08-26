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

## Read this before starting: the topology in the body is wrong

Noted 2026-08-26, from the code review of 037.

**"The launching terminal keeps the emulator handle" cannot hold as written.**
Ticket 011 settled that gbs launches DOSBox as its own child, because Yama's
descendant rule is what makes the memory read permitted with no system change.
If the HUD moves to a second window it is a second process, and a process that
is DOSBox's *sibling* has no permission to read it: `process_vm_readv` wants
`PTRACE_MODE_ATTACH_REALCREDS`, and under `kernel.yama.ptrace_scope = 1`, the
distro default, only an ancestor gets it. Yama permits descendants of the
tracer, and a sibling is not one.

So this ticket as written works only at `ptrace_scope = 0`. That is the exact
thing 011 refuses to ship as an install step, and it is the difference between
"works on the author's machine" and "works".

**Two ways out. Neither is decided. Decide before writing code.**

1. **Invert who launches.** The parent runs the wizard, spawns the terminal,
   and the *child* launches DOSBox and reads it. The parent holds a pipe to the
   child and prints its log. The permission model is untouched and the parent
   still becomes the log. What it costs is this ticket's stated reason for the
   parent keeping the handle. 011's actual requirement is that somebody owns
   the handle and never kills the game with it, and this satisfies that.
2. **Keep DOSBox in the parent and pass an open `/proc/<pid>/mem`.** The kernel
   checks the ptrace permission when that file is opened and caches the mm, so
   an inherited descriptor keeps working in the child. It costs
   `process_vm_readv`, which `mem.rs` says is roughly three times faster for a
   scan, and it is a trick rather than a design.

Jeff's leaning as of 2026-08-26 is that 043 is the ticket that is wrong rather
than 011, and that this is settled when 043 is actually worked. Do not treat
the body above as gospel.
