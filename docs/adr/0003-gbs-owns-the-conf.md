# ADR 0003 — gbs owns the configuration it launches with

Status: accepted
Date: 2026-08-23
Supersedes: rules 1 and 2 of ADR 0001

## Context

Field testing on 2026-08-23, against a real GOG install and a real Steam
install, broke both halves of the publisher-files policy in one evening:

- GOG bundles DOSBox 0.74. It needs `libFLAC.so.8`, which GOG's `libs/`
  folder does not carry and current distros no longer ship, so it dies on
  load, and GOG's wrapper script exits 0 anyway. The breakage cannot be
  detected from the exit code.
- Steam's confs are written for DOSBox on Windows. The autoexec's
  `mount c .\GAME` is a literal filename on a Linux host, so the mount
  fails and the trailing `exit` closes the emulator instantly.
- GOG's conf forces fullscreen, a surprise nobody asked for.

Meanwhile the thing both publishers' autoexecs actually do is identical and
tiny: mount the folder above `POOLRAD` as C:, `cd POOLRAD`, run `START.EXE`.
A working hand-written conf (`~/goldbox/por.conf`) proved a current
dosbox-staging needs no publisher settings at all to run the game well.

## Decision

For a discovered install, gbs launches a system emulator with configuration
gbs owns. Publisher conf files, launch scripts, and bundled emulators are no
longer read, ordered, or launched. Two parts:

1. **A per-game settings conf, owned by gbs.** Created on first use from a
   compiled-in template, under gbs's own config folder (never in the game
   install), and named to the user so they know where emulator settings
   live. The user edits it freely; gbs never overwrites an existing one.
2. **A computed autoexec, passed as `-c` commands.** Mount the folder above
   the game's folder as C:, enter the folder, run the game's start command
   from the registry, `exit`. Computed fresh each launch from the install's
   recorded paths, so it can never go stale.

The emulator is found on PATH by its common names: `dosbox`, then
`dosbox-staging`, then `dosbox-x`. `--dosbox` overrides.

The manual path is untouched: an install set up with `--conf` launches the
user's conf with the user's emulator, exactly as given. ADR 0001's rule 3,
never write into a game install, stands.

## Consequences

- gbs carries per-game launch knowledge: the registry gains a DOS start
  command per game.
- Discovery shrinks to what it is good at: finding the game folder and the
  saves. Conf ordering, launch-script parsing, and bundled-emulator hunting
  are deleted.
- Publisher quirks (Windows paths, fullscreen, `config -securemode`) can no
  longer break a launch. Ticket 023's open questions become moot.
- Publisher-tuned settings are lost. Accepted: the template is modeled on a
  conf proven in play, and the game predates every setting that matters.
