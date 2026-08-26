# Roadmap

Everything Goldbox Squire might grow, and roughly in what order. This is a
document to point at, not a promise, and it is edited as often as the program
changes.

The goal is stated once, plainly: do everything Gold Box Companion does, on
Linux, without Wine. GBC is Windows-only and works with vanilla DOSBox 0.74-3
only, which is why this program exists. Where a GBC feature does not fit a Linux
desktop or does not fit taste, this file records the tweak rather than dropping
the feature.

Every feature below is named in GBC's own feature list unless it is marked
**Squire only**.

## Excluded

- **The experimental monk class for Pool of Radiance.** Jeff's call, from the
  start.
- **`VK.exe`.** It reports Windows virtual keycodes for GBC's hotkey config.
  Virtual keycodes are a Windows concept, so a Linux tool needs a different key
  syntax, not a port of this tool.
- **Reading a DOSBox that runs under Wine.** Squire exists to remove the Wine
  layer, not to read through it.

## Built

- Launch a game, find the party in emulator memory by character name, and print
  it live. `--watch` is the default; `--json` is the seam the HUD and any other
  reader will use.
- The wizard asks three things: which game, which directory, which save slot.
- All twelve games have character-record tables. Another game is a table, not
  code.
- Install discovery for GOG and Steam, a typed-path escape hatch, and a
  Squire-owned emulator conf per game.

## Next, and cheapest

These need no new capability. The memory reader already works and the record
tables already exist.

1. **Verify the last three tables.** Nine of the twelve are checked against real
   saves. `matrix-cubed`, `countdown-to-doomsday`, and
   `gateway-to-the-savage-frontier` still say UNVERIFIED, because their game
   folder names are guesses until a real install confirms them.
2. **Conditions and effects.** Show what is on a character: good effects, bad
   effects, and level drain. GBC shows good in green and bad in red. This is a
   separate structure from the character record and needs its own read.
3. **Full character display.** Everything in the record, not just name, class,
   level, and HP. Ability scores, AC, XP, encumbrance, saving throws.
4. **Inventory display.** Read-only first. Items, equipped state, charges.
5. **The save-game list.** List a game's saves chronologically with the map name
   as a description, which is what GBC does, so a player can tell one save from
   another before loading it.
6. **Save backup.** Copy the latest save with a description attached.
7. **A README.** Ticket 012, still open.

## The HUD

GBC pins its HUD above the DOSBox window. Squire will not, because Wayland
forbids a client positioning its own window. There is no flag and no protocol
for it, so no amount of work makes the pin-above trick portable here.

What Squire can do is name its window. The HUD sets an app id, and the user
writes one KWin or Hyprland rule against that name, once, after which the window
lands where they want it on every launch. Squire sets the size itself and
remembers the last one. Position stays the compositor's to decide.

**One layout, driven by rules.** There are no named modes to switch between. The
rules ask whether a thing fits at the current size and add or drop it
accordingly, in a settled order, so a window that lands between two sizes gets
the right answer rather than the nearer name. Content differs by size rather
than merely reflowing. Small is the party and nothing else, and only a roomy
size earns a map or a journal beside it.

## Writing to memory

Everything above is read-only. These features write, so they need the write
path, an "is combat running" check (GBC refuses several of these during combat),
and more care.

1. **The Fix command.** Instantly heal the party, optionally curing level drain.
   GBC ships this for Pool of Radiance and it works in the other games too. This
   is the smallest useful write, so it is the one that proves the write path.
2. **Spell memorisation save and restore.** Store the memorised list, restore it
   in one action.
3. **Level up without a training hall**, optionally ignoring race and game level
   limits.
4. **Temporary race change to human**, to dodge the demihuman experience limits.
5. **Character editor.** GBC edits memory rather than save files, so the change
   is instant. Same posture here.
6. **Teleport.** To a chosen map location, and on the world map where the game
   has one.
7. **Auto-identify items**, optional.
8. **Auto-ammo.** Top arrows, bolts, and darts up to a chosen count.
9. **Auto-disable quickfight after combat.** The older games left quickfight on
   between fights, so a player could walk into a dangerous fight with it
   enabled.
10. **Paladin and ranger in Pool of Radiance.** The research found both classes
    already exist in the engine, so enabling them is a set of byte writes rather
    than new game logic. Exportable to Curse of the Azure Bonds.

## Maps and views

Each of these is a renderer plus a data source, and each is a bigger job than
anything above.

- **Automap** with the party's location, and notes the player can attach.
- **World map** with the party's location, in the games that have one.
- **Combat view** showing character and monster positions, and whether each one
  is held or helpless.
- **Journal entries**, read from the game's own journal data.

## File tools

These do not touch the running game. They read and write files on disk, and they
all sit on one shared dependency: the DAX container format and its run-length
encoding. Build that codec once, first.

- **Save-game editor.** Swap party members in and out, edit inventories and
  effects. GBC ships this as a separate program (`SGE.exe`) and it needs the same
  twelve record layouts, with no memory access.
- **Item and monster editing** in DAX files, including bulk identify.
- **Font and combat-icon replacement.** EGA only, bitmap in and bitmap out.
- **FRUA module manager.** Download modules from the UA File Archive, install
  them, apply module hacks. No game knowledge, but it needs network access and a
  binary patch format.
- **ECL script tooling.** A browser and editor for the `ECL*.DAX` bytecode, and a
  live monitor that follows the program counter in memory. The largest item on
  this page, and the one with the least payoff for a player.

## Squire only

Features GBC has no equivalent for, because they follow from being a native
Linux program rather than a Windows overlay.

- **The HUD.** The obvious first interface past the printed table, drawn as a
  TUI. See the section above.
- **A pixel HUD**, if the TUI proves the idea is worth more polish. The map is
  the part most likely to want real pixels.
- **A structural anchor.** Today the anchor is a character name read from the
  save files, which is why Squire works where GBC fails. An anchor that needs no
  save file at all would let Squire attach to a game in progress it never
  launched.
- **Verify the anchor across DOSBox builds.** Ticket 001, still open. The whole
  design rests on the claim that any DOSBox build works.
- **GOG flat installs** and the **Steam Unlimited Adventures overlay mount**.
  Tickets 031 and 032, both open, both real layouts that discovery does not yet
  handle.
- **Tandy intro music.** Ticket 030, open.
- **Packaging.** Crates.io, GitHub releases, AUR, or none of them.
- **The Windows platform layer.** `ReadProcessMemory` instead of
  `process_vm_readv`. Cheap if planned for, painful if bolted on. Low priority,
  because Windows users already have GBC.

## Adjacent, and probably not ours

`GBC_Audio.exe` looks like a game-audio feature and is not one. The research
found it to be a host-side music player driven by game state, playing a music
pack the user downloads by hand. Squire could do the same thing once it knows
the game state well enough. It changes nothing about the game.
