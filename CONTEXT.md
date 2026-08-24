# Context

The language Goldbox Squire uses. One word for one thing. This file is a
glossary and nothing else: no offsets, no module names, no decisions.

## The game on disk

**Game.** One of the twelve SSI Gold Box titles. A game has an id used in
configuration and on the command line, a display name, a character record
layout, and a save shape. Pool of Radiance is a game.

**Install.** One copy of one game on disk: a game directory and which game it
holds. Two installs of the same game can sit side by side, and each game
remembers which one it uses (the chosen directory).

**Install kind.** Who laid the install out. GOG, Steam, found, and manual.
Found means Squire recognized the shape but no launch script named a publisher.
Manual means the user typed the directory themselves rather than letting
Squire find it.

**Chosen directory.** The install a game uses, remembered per game the first
time the wizard's directory question is answered. The question is then
skipped; `--game-dir` or a config edit changes it.

**Game folder.** The game's own DOS folder, named `POOLRAD` for Pool of
Radiance. Discovery identifies which game an install holds by this name, and
the computed autoexec enters it to start the game.

**Save folder.** The folder inside an install holding the save files. The game
writes here during play. GOG's game folder is also its save folder; Steam
nests a `SAVE` folder inside the game folder. A game with designs has one
save folder per design.

**Save shape.** How a game writes a save slot to disk. Ten games write one
file per character (`CHRDAT` plus slot letter and character index); Unlimited
Adventures and The Dark Queen of Krynn write the whole party into one party
file. The shape is data in the game's table, like the record layout.

**Party file.** One save slot's whole party in a single file, named `SAVGAM`
plus the slot letter. Reading it yields the party's names in marching order.

**Design.** One Unlimited Adventures adventure module, a `.DSN` folder inside
the game folder, each with its own save folder. The wizard asks which design
every run, like the slot: both describe one sitting.

**Save slot.** One saved game, named by a letter A through J. This is what the
player picks in the game's own load menu. GOG ships slots A and J populated.

**Character index.** One character's position in a save slot's marching order,
numbered 1 through 6. A save slot holds up to six.

> Slot and character index were one word once, and crossing them was a real bug.
> "Slot" on its own means the letter. Never use it for the digit.

**Settings conf.** The per-game emulator configuration file Squire creates
once, in its own config folder, and never touches again. It belongs to the
user. Nobody else's configuration files are ever launched: not a publisher's,
not a hand-written one. Every install launches with this file.

**Computed autoexec.** The DOS commands that start the game: mount, enter the
game folder, run the game's start command, exit. Squire computes them each
launch from where it found the game, so they cannot go stale.

## The game in memory

**Emulator.** The DOSBox process running the game. Squire starts it, which is
what makes reading its memory permitted.

**Party.** The characters of the loaded save slot, in marching order.

**Character.** One party member's live numbers: name, class, levels, hit points,
armor class, status, ability scores.

**Anchor.** The place in the emulator's memory where one character's record was
found. An anchor is found by searching for the character's name, because the
player wrote that name and it never changes during play.

**Stale anchor.** An anchor whose name is no longer there. Loading a save or
restarting the emulator moves a record. A stale anchor causes a fresh search,
never a stale number.

**Party state.** How much of the party Squire can currently see.

- **Live.** Every character of the chosen save slot was found.
- **Partial.** Some were found. The game is mid-load, or the party changed.
- **Not found.** None were found. The game is running, but no party is loaded.

## Reading

**Record table.** The description of one game's character record: which field
sits at which offset, how wide it is, how to read it, and what its legal values
are. Data, not code. Adding a game means adding a table.

**Transform.** A stored value that is not the shown value. The engine stores
armor class and THAC0 as sixty minus the real number.

**Validation.** The check that a candidate record really is a character record,
and not a copy of the name sitting in a file buffer. Every field with a known
legal range is checked.

**Unknown value.** A byte the record table has no name for. Squire reports it as
unknown and shows the raw byte. It never guesses.
