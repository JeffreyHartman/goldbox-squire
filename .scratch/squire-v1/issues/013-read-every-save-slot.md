# 013 — Read every save slot, not just A

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

Fix the crossed concepts in the save reader. A save file is
`CHRDAT{slot}{index}.SAV`, where the save slot is a letter A through J and the
character index is 1 through 6. The current code holds the letter at A and
iterates the character position 1 through 10, so a player who saved into any
slot but A gets slot A's names or nothing.

The core gains two operations. Enumerate the populated save slots of a game
folder, each with its party's names in marching order, for the wizard's slot
list. And read one named slot's party names. Both follow `CONTEXT.md`: "slot"
means the letter, never the digit.

A slot counts as populated when at least one of its `CHRDAT` files parses.
`SAVGAM{slot}.DAT` is not required; a slot missing it is still readable, and
refusing it would be guessing.

## Acceptance criteria

- [x] Reading slot J of a folder holding GOG's A and J returns the J party, in
      marching order.
- [x] Enumeration returns each populated slot's letter and names, and skips
      letters with no parseable `CHRDAT` file.
- [x] Character indexes stop at 6. No slot reads ten files.
- [x] Case-insensitive filename matching still works.
- [x] The test asserting the old ten-file behaviour
      (`reads_the_files_in_numeric_order_not_alphabetical_order`) is rewritten
      to assert the correct shape.
- [x] No identifier uses "slot" for the character index.

## Answer

`saves::slot_party_names(dir, letter)` reads one slot, and
`saves::populated_slots(dir)` enumerates the populated slots with their names
in marching order. The letter is validated against A through J, lower case is
accepted, and the character index stops at 6. An empty slot errors with the
populated letters, so the CLI can print the message unchanged. One directory
listing serves every lookup, which keeps the case-insensitive match and drops
the per-file fallback scans. The old `party_names` and its ten-file loop are
gone; `main.rs` reads slot A until 018 adds `--slot`.
