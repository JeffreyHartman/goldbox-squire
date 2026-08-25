# 025 — Launch a discovered install with gbs's own configuration

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 024

## What to build

ADR 0003's two parts, for discovered installs only. The manual `--conf` path
keeps its current behavior.

**The settings conf.** One per game, at gbs's config folder (for example
`~/.config/goldbox-squire/pool-of-radiance.conf`). Created from a
compiled-in template on first use; never overwritten once it exists. The
template is minimal, modeled on the proven `~/goldbox/por.conf`: windowed,
and comments telling the user this file is theirs. The first-run note names
this path instead of the publisher's confs.

**The computed autoexec.** The registry gains a per-game DOS start command
(`start = "START.EXE"` for Pool of Radiance). The launcher passes, as `-c`
commands after the conf: mount C on the folder above the game's folder,
`c:`, `cd` into the folder, the start command, `exit`. The mount target is
computed from the install's recorded save path each launch. The `exit` is
what makes quitting the game end the watch session cleanly (ticket 018).

`Emulator` needs a `-c` builder to carry the commands.

## Acceptance criteria

- [x] The registry parses and exposes the start command; a game without one
      fails to parse.
- [x] The mount folder is computed correctly for the GOG shape
      (`data/POOLRAD`) and the Steam shape (`GAME/POOLRAD/SAVE`).
- [x] A missing settings conf is created from the template; an existing one
      is left alone.
- [x] The launch command line holds the gbs conf and the five `-c` commands
      in order.
- [x] A manual install still launches the user's conf, with no `-c` commands.

## Answer

The registry gained `start = "START.EXE"` and `Game::from_toml` refuses an
empty one. `Emulator::command` carries each DOS command as `-c`.
`squire_cli::conf::ensure` creates `<config dir>/<game id>.conf` from the
template (windowed, commented as the user's file) and never touches an
existing one; the run that creates it says so. `squire_cli::conf::autoexec`
computes mount, `c:`, `cd`, start, `exit` from the install's recorded save
path, so the GOG shape mounts `data` and the Steam shape mounts `GAME`, with
quotes around paths holding spaces. `main` branches on install kind: manual
launches the user's confs exactly as before, everything else launches the
gbs conf plus the computed autoexec.
