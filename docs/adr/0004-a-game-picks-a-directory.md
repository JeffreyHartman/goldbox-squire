# ADR 0004 — A game picks a directory, and every launch is gbs's

Status: accepted
Date: 2026-08-23
Supersedes: ADR 0003's manual-conf exception; ticket 021's manual path

## Context

After ADR 0003 the wizard listed three entries for one game: GOG, Steam, and
a "manual" entry that was really a hand-written conf file whose directory was
GOG's own game folder. Two of the entries were game directories, one was a
way of launching. Mixing the two concerns in one menu was confusing in
practice, on the first real evening of use.

Meanwhile the reason the manual-conf path existed had evaporated: the
gbs-owned settings conf is user-editable, so everything the hand-written
conf did (machine, speaker model, compressor, mouse capture) belongs in the
per-game settings conf as defaults.

## Decision

**The wizard asks three things, in this order.**

1. **Which game.** Always asked, one numbered entry per compiled-in game,
   Enter repeats the last game. Every game is listed even before it has a
   directory.
2. **Which directory.** Asked only when the game has no remembered
   directory. The menu lists the discovered game directories plus a
   type-a-path entry for one discovery missed. The choice is remembered per
   game and the question is then skipped; changing it is `--game-dir` or an
   edit of the config file until a fuller TUI exists.
3. **Which save slot.** Every run, never remembered (ADR 0002).

**Back is `0`, never `b`.** `b` collided with save slot B, which the old
wizard made unpickable. `0` is never a slot letter and never a 1-based menu
number.

**An install is a game directory and nothing else.** Manual means the user
typed the directory; GOG and Steam mean discovery found it. All launch the
same way: the gbs settings conf plus the computed autoexec. `--conf` is
deleted, and the hand-written conf's proven settings move into the settings
conf template as defaults. The user tweaks that file after it is created.

**The emulator override is one config field.** `--dosbox` overrides for one
run; a `dosbox` field in the config file overrides permanently, edited by
hand for now. Installs no longer carry an emulator.

## Consequences

- One game, one directory, one way to launch. The menu shows game
  directories only.
- A hand setup whose directory is inside a discovered install stops being a
  third entry; same-folder installs collapse to the discovered one.
- Quitting the game now always ends the session, because every launch uses
  the computed autoexec's `exit`. The old no-`exit` DOS-prompt workflow is
  gone; it was a hand-conf feature, and the hand conf is gone.
- The config format changes again (chosen directory per game, `last_game`,
  global `dosbox`); v1 and v2 files migrate on load.
