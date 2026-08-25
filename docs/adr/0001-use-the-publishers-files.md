# ADR 0001 — Use the publisher's files, never write into an install

Status: partly superseded by ADR 0003
Recorded: 2026-08-23, retroactively. The decision predates this file and was
referenced from code and tickets before it was written down.

## Context

`gbs` must start the game, not just DOSBox, or the user lands at a `Z:\>`
prompt. The obvious move is to generate a DOSBox conf from the game folder. We
decided instead to discover the install on disk and launch it with the conf
files the publisher shipped, in the publisher's order.

Both real-world layouts were inspected and drove the decision. GOG ships
`dosbox_por.conf` plus `dosbox_por_single.conf` and runs them in that order via
`start.sh`. Steam ships `base.conf`, `graphics.conf`, `game.conf` and runs them
in that order via `run-game.bat`. Both autoexecs use relative mounts, so the
emulator must be started with its working directory at the install. A generated
conf would have to reproduce all of that per publisher and per game, and it
would still lose the publisher's tuned settings.

## Decision

Three rules, made while designing install discovery and the launcher:

1. Launch a discovered install with the publisher's own conf files, in the
   order the publisher's launch script names them.
2. Never generate conf files.
3. Never write into a game install.

## Considered options

- **Generate a conf from the game folder.** Rejected. It duplicates what the
  publisher already wrote, drifts when a publisher changes layout, and needs a
  per-game launch entry in the record tables. ADR 0003 later accepted exactly
  this cost, because the publisher's files turned out not to run at all.
- **Symlink the game folder under the name its DOS config expects.** Works, but
  puts an invisible link between the user and their save files. Rejected.
- **Rewrite the game's own config file (`POOL.CFG`).** Editing a user's game
  install is off the table.

## Standing

Rule 3 stands untouched, and it is the rule that outlived the rest. Rules 1 and
2 are superseded by ADR 0003: field testing showed the publisher's files cannot
be trusted to run under a native Linux emulator, and gbs now owns the
configuration it launches with.

The consequences recorded below are the 2026-08-23 consequences of rules 1 and
2. Read them as history, not as current behavior.

- A hand-assembled setup stays supported: `--conf` plus `--game-dir` names the
  pieces manually and is remembered as a manual install. Deleted by ADR 0004.
- Discovery is structural. Any folder holding an `[autoexec]` conf and a save
  folder is an install, whoever made it. A layout we have never seen fails
  cleanly to the manual path.
- Squire never writes into a game install, and never owns emulator settings.
  The note "edit these files to change settings" points at the publisher's
  files. The second half is reversed by ADR 0003: gbs owns a per-game settings
  conf, and the note points at that instead.
