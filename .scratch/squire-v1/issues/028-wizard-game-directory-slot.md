# 028 — The wizard asks game, directory, slot

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 027

## What to build

ADR 0004's flow, replacing the install menu.

**Question 1, every run:** "Which game?" One numbered entry per compiled-in
game, listed even when the game has no directory yet. Enter repeats
`last_game`. `--game` answers it in advance.

**Question 2, only when the game has no valid chosen directory:** "Where is
<game folder>?" The discovered directories of that game, numbered, plus a
final "somewhere else (type a path)" entry. A typed path must hold the
game's `CHRDAT*.SAV` files directly or in one direct child; it becomes a
manual install and the game's chosen directory. `--game-dir` answers the
question in advance and re-points the game permanently. The POOL.CFG
folder-name check guards typed directories exactly as it guarded the old
manual path.

**Question 3, every run:** the slot, as today, except back is `0`. This
fixes a live bug: `b` meant back, so save slot B could not be picked by
letter.

The first-run note about conf files goes away; the settings-conf creation
message already names where emulator settings live. `--conf` is deleted
from the arguments and the usage text is rewritten. `--pid` resolves its
save folder through the game's chosen directory.

## Acceptance criteria

- [x] The game question lists every compiled-in game and repeats the last
      game on Enter.
- [x] A game with a chosen directory skips straight to the slot question.
- [x] The directory menu offers the discovered directories and a typed path,
      and remembers the pick.
- [x] Slot B is pickable by typing `b`; `0` goes back.
- [x] `--conf` is gone; `--game`, `--slot`, `--game-dir` skip their
      questions.
- [x] `--pid` works against the chosen directory.

## Answer

`wizard::choose` asks game (every run, all compiled-in games listed, Enter
repeats `last_game`), directory (only while the game has none chosen; the
discovered directories plus a typed path validated by
`discover::saves_within` and checked by `manual::folder_name_check` at
launch), and slot (every run). Back is `0` everywhere; slot B is pickable by
typing `b`. `--game`, `--game-dir` (re-points permanently), and `--slot`
skip their questions. `--conf` and `remember_manual` are gone, the first-run
conf note with them; the settings-conf creation message names where settings
live. `--pid` resolves through `last_game` and the chosen directory. Both
the migrated-v2 and fresh flows were verified end to end with a stub
emulator.
