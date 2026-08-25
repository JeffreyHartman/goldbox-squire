# 023 — Verify the Steam install runs under a native DOSBox

Type: `wayfinder:task` (HITL)
Status: closed, superseded by ADR 0003
Blocked by: 017

## Question

The Steam install ships a Windows `DOSBox\DOSBox.exe`, so on Linux the
discovered confs will run under a native `dosbox` from PATH instead. The conf
files are plain DOSBox format, but two things in Steam's autoexec are untested
against a native build: `config -securemode` and `@start`. Whether the game
launches, runs, and writes saves under native DOSBox and dosbox-staging is a
claim to verify with a real session, not a fact.

If it fails, the finding decides what discovery does with a Steam install
whose only emulator is a Windows binary: refuse with a message, or filter the
offending lines. Record what happens either way.

This needs a display and a human at the keyboard, so it is not agent work.

## Answer

First finding, from Jeff's session of 2026-08-23 (dosbox-staging 0.83.0-RC1):
the launch fails before either untested line is reached. Steam's `game.conf`
autoexec opens with `mount c .\GAME`, a Windows-style relative path. On a
Linux host the backslash is a literal character, so the mount fails
(`MOUNT: .\GAME isn't a directory or valid image file.`), `c:` has no drive,
and the trailing `exit` closes the emulator a fraction of a second after it
starts. `config -securemode` and `@start` therefore remain untested.

Resolution: ADR 0003. gbs no longer launches any publisher conf; a
discovered install runs gbs's own settings conf plus a computed autoexec,
so the Windows-style mount, `config -securemode`, and `@start` are never
executed. The paragraph below records the fix candidate considered before
that decision.

Considered at the time: decide what gbs does with it. The candidate that keeps ADR 0001
intact is a small gbs-owned conf, stored under gbs's own config folder and
passed first on the command line, whose `[autoexec]` mounts `GAME` with a
host path; the conf's later Windows-path mount then fails harmlessly against
an already-mounted drive. Appending `-c` commands does not work, because
DOSBox runs them after the confs' autoexec, which ends in `exit`.
