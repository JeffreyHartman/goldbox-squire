# Context

The language Goldbox Squire uses. One word for one thing. This file is a
glossary and nothing else: no offsets, no module names, no decisions.

## The game on disk

**Game.** One of the twelve SSI Gold Box titles. A game has an id used in
configuration and on the command line, a display name, and a character record
layout. Pool of Radiance is a game.

**Install.** One copy of one game on disk, as a publisher laid it out. An
install knows which game it holds, where its saves are, which emulator
configuration files start it, and in what order. Two installs of the same game
can sit side by side.

**Install kind.** Who laid the install out. GOG, Steam, found, and manual.
Found means Squire recognized the shape but no launch script named a publisher.
Manual means the user named the pieces themselves rather than letting Squire
find them.

**Save folder.** The folder inside an install holding the save files. The game
writes here during play.

**Save slot.** One saved game, named by a letter A through J. This is what the
player picks in the game's own load menu. GOG ships slots A and J populated.

**Character index.** One character's position in a save slot's marching order,
numbered 1 through 6. A save slot holds up to six.

> Slot and character index were one word once, and crossing them was a real bug.
> "Slot" on its own means the letter. Never use it for the digit.

**Emulator configuration.** The ordered list of configuration files that start
one install's game. Later files override earlier ones, so the order is part of
the install, not a detail. GOG ships two. Steam ships three.

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
