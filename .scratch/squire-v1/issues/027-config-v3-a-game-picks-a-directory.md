# 027 — Config v3: a game picks a directory

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

The config model behind ADR 0004.

**Install** shrinks to a game directory: game id, kind, root, saves. The
`confs`, `emulator`, and `introduced` fields go away. Manual now means the
user named the directory.

**Config** gains: `chosen`, a map from game id to the install key that game
uses; `last_game`, the game menu's Enter default; and an optional `dosbox`,
the permanent emulator override (hand-edited for now). `last_install` goes
away.

**Migration.** A v2 file's `last_install` becomes `last_game` plus a
`chosen` entry. A v2 manual install keeps its root as a plain directory
install; its confs are dropped. A v1 file's `game_dir` still becomes a
manual install, now also chosen; its `dosbox` becomes the config's `dosbox`
field; its `conf` is dropped.

**Dedup.** A manual install whose canonical game folder equals a discovered
install's is the same install; `absorb` drops the manual one. A `chosen`
entry pointing at a key that no longer exists is cleared, so the wizard asks
again.

**Emulator.** `Install::emulator_command` goes away. A free function decides:
`--dosbox` argument, then the config's `dosbox`, then the PATH search, then
the error naming the three names.

## Acceptance criteria

- [x] A v2 config (with a conf-carrying manual install and `last_install`)
      loads as: directory installs, `chosen` filled, `last_game` set.
- [x] A v1 config still migrates, and its `dosbox` lands in the config field.
- [x] `absorb` drops a manual install duplicating a discovered game folder.
- [x] A dangling `chosen` entry is cleared on absorb.
- [x] The emulator precedence is argument, config field, PATH, error.

## Answer

`Install` is now game, kind, root, saves. `Config` carries `last_game`,
`dosbox`, `chosen` (game id to install key), `installs`, `extra_roots`.
`from_toml` reads all three versions: v2's `last_install` becomes the game's
chosen key plus `last_game`, a v2 manual install keeps its root as a plain
directory, and v1's `game_dir`/`dosbox` become a chosen manual install and
the config's `dosbox`. `absorb` replaces discovered installs wholesale,
drops a manual entry whose canonical game folder a discovered install also
names, and clears dangling `chosen` keys. `needs_rediscovery` fires on no
discovered installs, a vanished discovered root, or any two installs
reaching one canonical game folder, so an old config's duplicates self-heal
on the next run. `emulator::command` decides the emulator: argument, config
field, PATH search, error.
