# 009 — The CLI contract and the JSON schema

Type: `wayfinder:prototype` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: 008, 011

## Question

Decide what the command looks like and what it prints. Build a rough artifact to
react to, rather than arguing in the abstract.

- The command shape, arguments, and flags.
- The human-readable table: what it shows for six characters, and how it reads
  at a glance during play.
- The `--json` schema. This is the seam a TUI or GUI will later depend on, so it
  outlives the table.
- What is printed when no emulator is running, when the game has not begun, and
  when permission is denied.

## Answer

**`gbs [--game-dir DIR] [--dosbox CMD] [--conf FILE] [--pid N] [--watch] [--interval MS] [--json]`**

`squire-cli/src/args.rs` and `squire-cli/src/output.rs`.

The table shows name, class, level, current and maximum hit points, armour
class, and status. Columns are as wide as their widest cell, so rows line up.
`--watch` redraws in place.

The JSON is written by hand rather than derived, so its shape is decided in one
place and does not follow the internal types. An unknown enumeration value is
`null`, with the raw byte alongside it, so a front end never shows a guess.

`--game-dir`, `--dosbox` and `--conf` are stored in
`$XDG_CONFIG_HOME/goldbox-squire/config.toml` after the first run.

No emulator running: the launcher reports it cannot start the command. Game not
begun: "No party in memory. Load a save and begin the game." Permission denied:
the error names the cause and says to let `gbs` start the game.
