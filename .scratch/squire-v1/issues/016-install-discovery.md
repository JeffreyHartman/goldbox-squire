# 016 — Install discovery

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 014, 015

## What to build

Find the user's installs so the wizard can list them. See ADR 0001: Squire uses
the publisher's conf files and never generates its own.

Discovery is structural, not a list of publishers. An install is a directory
holding a conf file with an `[autoexec]` section containing a `mount` line, and
a save folder (identified per game via the registry's save-folder name)
containing `CHRDAT*.SAV` files. Which game it holds comes from that folder
name.

Search a fixed list of roots, plus `extra_roots` from the config, capped at
four levels deep:

- `~/.local/share/Steam/steamapps/common/`
- `~/.steam/steam/steamapps/common/`
- `~/GOG Games/`, `~/Games/`, `~/gog/`, `~/goldbox/`
- `/opt/`

Conf order matters, because later files override earlier ones. Take it from the
publisher's launch script when one is present (`start.sh` for GOG,
`run-game.bat` for Steam). When absent, settings files come first and the one
holding `[autoexec]` comes last, which is the rule both publishers follow.

Record an emulator binary found inside the install (GOG ships DOSBox 0.74
under `dosbox/`). It is the fallback when no `dosbox` is on PATH; the system
dosbox, when present, is preferred. (Amended after field testing; see the
review notes.)

Discovered installs are written into the config so a normal run does no
filesystem scan. Rediscover when the wizard asks, or when a stored root no
longer exists.

## Acceptance criteria

- [x] A synthetic GOG-shaped tree is found: two confs ordered per `start.sh`,
      saves under `data/POOLRAD`.
- [x] A synthetic Steam-shaped tree is found: three confs ordered per
      `run-game.bat`, saves under `GAME/POOLRAD`.
- [x] A same-shape tree with no launch script is found, autoexec conf last.
- [x] A tree missing `CHRDAT` files or missing an autoexec mount is not an
      install.
- [x] The scan never descends more than four levels below a root.
- [x] A bundled emulator binary is recorded, as the fallback for a machine
      with no system dosbox. (Amended: it originally won over PATH.)
- [x] Results are cached in the config; a vanished root triggers rediscovery.

## Answer

`squire_core::discover` walks the fixed roots plus `extra_roots`, at most
four levels below each, and treats a directory as an install when it holds a
`.conf` with an `[autoexec]` mount and, per game, a folder named like the
registry's save folder holding `CHRDAT*.SAV` files. Conf order comes from
`start.sh` (GOG) or `run-game.bat` (Steam) when present, else settings first
and the autoexec conf last. A bundled executable named `dosbox` is
recorded; `Install::emulator_command` prefers the system dosbox and uses the
bundle only when PATH has none. `Config::absorb` caches results and drops a stale
discovered install whose root vanished; `Config::needs_rediscovery` reports a
vanished root. A scriptless discovery gets the new install kind `found`,
added to CONTEXT.md.

### Review notes

Later superseded in part: ADR 0003 (2026-08-23) removed conf ordering,
launch-script conf parsing, and the bundled-emulator hunt. Discovery now
finds the game folder and the saves; ticket 026 did the deletion.

Field testing on the real GOG install reversed the bundled-wins call:

- GOG's bundle is DOSBox 0.74. It needs `libFLAC.so.8`, which GOG's `libs/`
  folder does not carry and current distros no longer ship, so it dies on
  load. GOG's `dosbox` wrapper script exits 0 anyway, so the breakage cannot
  be detected from the exit code. The system dosbox now wins; the bundle is
  the fallback for a machine with none. A manual install's `--dosbox` choice
  still wins over both.
- The `dosbox-staging/` folder that suggested GOG ships a staging build was a
  leftover AUR checkout from Jeff's own setup, not GOG's. GOG ships only 0.74.

Two readings settled during code review, open to veto:

- The four-level cap binds the walk that hunts install roots. Inside a
  recognized candidate (a conf-bearing folder), the save-folder and bundled
  emulator lookups walk up to four levels below the install itself, so a deep
  install keeps its saves findable. Scan cost stays bounded: the inner walk
  only runs where a conf already matched.
- "Rediscover when the wizard asks" is not yet wired to a wizard control: the
  rescan triggers are an empty config and a vanished root. The wizard has two
  questions (019) and no rediscovery question was designed; a `rescan` wizard
  entry or flag is future work.
