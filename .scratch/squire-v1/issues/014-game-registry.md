# 014 — A game registry instead of a hardcoded table

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

The wizard's first question needs a list of games, and today the code has one
hardcoded constructor for Pool of Radiance's record table. Replace it with a
registry: a function returning every compiled-in game, each knowing its id (the
key used in configuration and by `--game`), its display name, and its record
table.

The per-game TOML grows the data that install discovery and the manual-path
check need:

- The name of the folder the game's saves live in (`POOLRAD` for Pool of
  Radiance), used to identify which game a discovered install holds.
- The name of the game's own DOS config file and which line of it holds the
  DOS data path (`POOL.CFG`, line 3, `C:\POOLRAD\`), used to explain a
  folder-name mismatch on the manual path.

Tables stay compiled in with `include_str!`. Adding a game stays a data change.

## Acceptance criteria

- [x] The registry lists Pool of Radiance with a stable id.
- [x] The TOML carries the save-folder name and the DOS config file name and
      line, and the loader validates them.
- [x] No caller constructs the Pool of Radiance table by name anymore.
- [x] A malformed table entry fails at load, as today.

## Answer

`squire_core::games` is the registry. `games::games()` lists every
compiled-in game; `games::find(id)` looks one up. A `Game` carries the id, the
display name, the save-folder name, the DOS config location (`POOL.CFG`, line
3) and its record table. The TOML gained `id`, `save_folder` and a
`[dos_config]` table, and `Game::from_toml` validates them before handing the
text to the unchanged `Table::from_toml`. `Table::pool_of_radiance()` is
deleted; every caller goes through the registry.
