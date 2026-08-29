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

**Sitting.** What one run of the wizard resolves beyond the install: the save
folder (a design's, for a designs game) and the save slot. A fresh install
has no sitting yet; the game can still be started, and the sitting is picked
mid-watch after the first in-game save.

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

## Showing

**HUD.** Squire's live party on screen beside the game, in a window the user
places and sizes. The concept, not the technology. It is the HUD whether it is
drawn in a terminal or in pixels.

**TUI.** The terminal implementation of the HUD. A pixel implementation, if one
is ever written, is a second implementation of the same HUD. There is no other
word for this idea. Squire does not call it a dock.

**Plain output.** The printed table and the JSON: text written to a stream, for
a pipe, a script, or a reader that is not a person watching a screen. What the
HUD replaces for a person, and never replaces for a program.

**Layout plan.** What the HUD shows at a given size: which fields are present,
and where they sit. A size, a party, and a layout axis go in, a plan comes
out. The plan is computed by rules that answer "does this fit"; the axis is
the one thing those rules take as a preference rather than decide themselves.

**Layout axis.** Which way the party's cards flow: `Horizontal` fills a row
before starting a new one, `Vertical` fills a column before starting a new
one. A key flips between the two. Whichever axis is active, the rule still
packs in as many cards along it as fit, and a party that does not divide
evenly leaves its last row or column short rather than forcing an even one.

**Drop order.** The order fields leave the layout plan as the size shrinks, and
return as it grows. One order, settled once, so that a narrow HUD and a wide one
disagree about how much is shown and never about what matters.

**Roomy.** A size with room to spare after everything the party needs. Roomy is
a question the rules answer, not a measurement. What is roomy for a party panel
alone is cramped for the same panel beside a map.

**Caption.** The words that say which run is on screen: the game, the save
slot, the panel, and the watch's latest word. The layout plan fits them to the
header and to the status line. Not a sitting, which is what the wizard
resolved.

**Liveness.** What the HUD says about the numbers on screen. Live and partial
are the party state as the session reports it. The session's not-found splits
in two here, because the two need different words: **waiting** is a run that
has never found a party, and **lost** is one that had a party and lost the
anchor. Only lost dims.

**Wordmark.** Squire's name drawn large. It appears only when the size is roomy,
because a HUD that reminds you what it is called is spending rows that the party
wants.

**Terminal table.** Which terminals Squire can open a window in, and how each
one is asked for a window name and a size in character cells. Data, like a
record table, with one difference: a record table is Squire's, and a terminal
entry can be the user's. A terminal Squire does not know is still launched, at
whatever size the terminal chose.

**App id.** The name a view's window reports to the desktop. Squire sets it so
that the user can write one compositor rule and have the window land where they
want it every launch. Squire cannot place its own window and never will. There
is one name per view kind, `goldbox-squire-hud` and one for each view added
after it, from a fixed list Squire owns, because a rule written by hand against
a name breaks silently if the name drifts, and one shared name would place the
map wherever it places the HUD.

**Host.** The gbs process that launched the emulator and reads it. There is one
per run. It owns the anchor, the watch loop, the emulator handle and the socket,
and it is the only process permitted to read the emulator's memory, because it
is the only one that is its parent. Its own terminal shows the log.

**View.** A window Squire draws in, as a process of its own: the HUD today, a
map or a journal later. A view connects to the host's socket, draws the party
it is sent, and sends the user's decisions back. It never reads the emulator.
Views are throwaway. Closing one does not end the run, and quitting the host
closes them all. Not a panel, which is one region inside a single view.

**Log.** What the host's own terminal shows once the views have the party:
rescans, anchor losses, and the emulator's own output. The window the user
typed in does not go back to a prompt, and this is what it becomes.
