# 022 — The --pid path: attach, no wizard

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 018

## What to build

`--pid N` reads an emulator this tool did not start. It works only where the
system already permits it, and it is the path automation and tests use, so it
must never prompt.

- No wizard. The game and the save slot come from arguments or from the
  config's remembered values. When either cannot be resolved, error naming the
  missing flag; never ask.
- No launch, no conf handling, no install discovery.
- The existing permission-denied guidance (start the game through `gbs`
  instead) is preserved.

## Acceptance criteria

- [x] `gbs --pid N --game pool-of-radiance --slot J --game-dir D` reads the
      party with no interaction.
- [x] A missing, unresolvable slot errors with the flag name and the populated
      slots; it does not prompt.
- [x] The permission error text is unchanged.

## Answer

`squire_cli::attach::resolve` turns arguments plus remembered config into
the game id, the save folder, the slot and its names. It takes no input
stream, so it cannot prompt by construction; a gap errors naming the missing
flag (`--game`, `--game-dir`, or `--slot` with the populated letters, a lone
populated slot resolving on its own). `main` branches to it before
remembering, discovery and the wizard, so `--pid` writes nothing to the
config, launches nothing, and touches no conf. The watch runs without a
repick listener there, and the permission-denied text in `squire_core` is
untouched.
