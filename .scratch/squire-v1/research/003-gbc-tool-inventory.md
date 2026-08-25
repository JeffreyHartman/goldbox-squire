# 003 — Gold Box Companion: full tool inventory

Ticket: `.scratch/squire-v1/issues/003-research-gbc-full-tool-inventory.md`
Subject: Gold Box Companion (GBC) v2.65, 10-Jun-2021, by Joonas Hirvonen.
Date: 2026-08-22

## Sources

All statements below come from these sources. Each section names the source.

| Tag | Source |
|---|---|
| `GBC.txt` | `docs/gbc/tool-docs/GBC.txt` |
| `SGE.txt` | `docs/gbc/tool-docs/SGE.txt` |
| `GBC_Audio.txt` | `docs/gbc/tool-docs/GBC_Audio.txt` |
| `ECL_Tool.txt` | `docs/gbc/tool-docs/ECL_Tool.txt` |
| `ECL_Monitor.txt` | `docs/gbc/tool-docs/ECL_Monitor.txt` |
| `FRUA_Tool.txt` | `docs/gbc/tool-docs/FRUA_Tool.txt` |
| `FRUA_Module_Manager.txt` | `docs/gbc/tool-docs/FRUA_Module_Manager.txt` |
| `ItemMod.txt`, `MonsterMod.txt`, `IconMod.txt`, `FontMod.txt` | `docs/gbc/tool-docs/` |
| `DAXBuilder.txt` | `docs/gbc/tool-docs/DAXBuilder.txt` |
| `VK.txt` | `docs/gbc/Data/VK.txt` |
| `Configuration.txt` | `docs/gbc/Data/Configuration.txt` |
| `Levels.txt` | `docs/gbc/Games/01. Pool of Radiance/Levels.txt` |
| `PoR format` | `docs/gbc/Resources/Character file formats/01. Pool of Radiance.txt` |
| `CCHFORM.TXT` | `docs/hackdocs/CCHFORM.TXT` |
| `PCSPKR.TXT` | `docs/hackdocs/PCSPKR.TXT` |
| `TUNES.TXT`, `UASOUND.TXT` | `docs/hackdocs/` |
| `Tutorial` | `docs/gbc/Tutorial/index.html` |
| `Web` | <https://gbc.zorbus.net/>, read 2026-08-22 |

---

## 1. The 13 executables

"Memory" means that the tool reads or writes the memory of the running
DOSBox process. "Files" means that the tool only reads and writes files on
disk.

| Executable | Purpose | Memory or files | Replacement size |
|---|---|---|---|
| `GBC.exe` | The main tool. HUD, automap, character editor, level up, teleport, journals. | Memory (plus files for save lists, notes, and backups) | Large. It is the whole product. It needs a memory scanner, 12 record formats, a map renderer, and an overlay window. |
| `GBC_Audio.exe` | The same tool as `GBC.exe` plus a music player driven by game state. | Memory | Small on top of `GBC.exe`. It adds an audio player and one configuration file per game. |
| `SGE.exe` | Save Game Editor. Edits party composition, inventories, items, and effects in save files. | Files | Medium. It needs the same 12 record formats, but no memory access and no live sync. |
| `ECL_Tool.exe` | Browser and editor for `ECL*.DAX` bytecode scripts. Commands, strings, flags, monsters, items, treasure. | Files | Large. It needs a per-game bytecode disassembler with hand-maintained data-offset tables. |
| `ECL_Monitor.exe` | Live view of the running ECL script and its flags. Follows the program counter. Edits flags and code in memory. | Memory | Medium, but only after `ECL_Tool` exists. It is `ECL_Tool` plus a memory read loop. |
| `FRUA_Tool.exe` | Debug tool for Unlimited Adventures. Reads keys, special items, quests, and events from memory. Saves and restores parameter states. | Memory | Medium. It is a second, smaller record layout, plus event and quest tables. |
| `FRUA_Module_Manager.exe` | Downloads FRUA modules from the UA File Archive, installs them, applies module hacks, and patches `CKIT.EXE`. | Files, plus network | Medium. The work is download, unzip, file copy, and a binary patch format. No game knowledge. |
| `MonsterMod.exe` | Views and edits monster statistics in `DAX`/`GLB` files. Exports a monster to SGE. | Files | Medium. It needs the monster record layout for each game. |
| `ItemMod.exe` | Views and edits item statistics and item property rows in `DAX` files. Bulk-identifies items. | Files | Medium. It needs the item record layout and the item property table. |
| `IconMod.exe` | Replaces combat icons in a game. EGA only. Exports and imports bitmaps. | Files | Medium. It needs the DAX image codec and a paint-friendly import path. |
| `FontMod.exe` | Replaces the font used by a game. Exports and imports a bitmap. | Files | Small. It is one image in and one image out, over the DAX codec. |
| `DAXBuilder.exe` | Decodes the run-length encoding of a `DAX` file, extracts the members, and rebuilds the file. | Files | Small. It is the DAX codec with no editor on top. Every other file tool needs this codec first. |
| `VK.exe` | Reports the Windows virtual keycode of a pressed key, for the `hotkey_vk` value in `Configuration.txt`. | Files (none, in practice) | Not applicable on Linux. Virtual keycodes are a Windows concept. A Linux tool uses a different key syntax. |

Notes on the table.

- `DAXBuilder`, `FontMod`, `IconMod`, `ItemMod`, `MonsterMod`, and `ECL_Tool`
  share one dependency: the DAX container format and its run-length encoding
  (`DAXBuilder.txt`, `ECL_Tool.txt`). Build that codec once.
- `DAXBuilder`, `SGE`, `FRUA_Tool`, `ECL_Tool`, and the mod tools must sit in
  the GBC folder because they share GBC's data files
  (`DAXBuilder.txt`, `SGE.txt`, `FRUA_Tool.txt`).
- `GBC.exe` does not modify the game executables (`GBC.txt`).

---

## 2. Feature list for `GBC.exe`

Source: `GBC.txt` and `Web`, unless stated. The Steam build of GBC omits some
features (`Web`).

### 2.1 Setup and search (universal)

- Search wizard. Select a game, set the game folder, select a save slot A–J,
  and search DOSBox memory for the character and map data (`GBC.txt`).
- `SETUP GOG` sets the folders for all GOG-installed games at once.
- `GOG SETTINGS`, `DOSBOX SETTINGS`, and `START GAME` launch GOG helpers.
- Address range and DOSBox window title are configurable.
- Read test (`Control + R`) and debug logging (`Control + D`).
- For Unlimited Adventures, a design is selected from a dropdown list.
- Support for the GOG cloud saves folder, with a checkbox.

GBC assumes a DOSBox memory footprint. The manual states that plain DOSBox
0.74 is necessary for non-GOG installs, and that a change to the sound card
settings can move the data out of the expected place (`GBC.txt`).

### 2.2 Read-only display

Universal unless marked.

- HUD above the DOSBox window: character name, hit point bar, experience bar,
  combat icons, and effects. Good effects in green, bad effects in red. Shows
  level drain.
- Custom combat icons from `<name>_1.bmp` and `<name>_2.bmp` bitmaps.
- Automap with party location, explored state, and doors.
- World map view with party location. **Per game.** Curse of the Azure Bonds
  shows a static picture with captions only.
- Combat view with character and monster positions. Shows held and helpless
  states. Mouse-over shows monster stats and effects.
- Journal entries, displayed over the DOSBox window.
- Password list.
- PDF viewer hooks for `Manual.pdf`, `Adventurers Journal.pdf`, and
  `Cluebook.pdf`.
- Saved game list in chronological order, with the map name as a description.
- Experience tables per game, as text files (`XP.txt`).
- Monster manuals per game, in `Resources/Monster Manuals/`.
- Random name generator.
- Map event numbers, shown on the map by right click.
- `SAVE MAP` writes the current map to a PNG in `Screenshots/`.
- `DEBUG DUMP` writes a text file of the found memory addresses.

### 2.3 Write features against live memory

All of these change the running game. All of them are blocked during combat
unless stated.

- **Encamp-fix.** Instantly heals all characters. Optionally repairs level
  drain (`FIX DRAIN`). In Pool of Radiance, drain repair works only in the same
  session in which the drain happened.
- **Race hack.** Sets the race of all characters to human, to escape demihuman
  level limits. `RESTORE RACES` puts the races back.
- **Store and restore spells.** Records the memorized spell list for all
  characters and writes it back with one click. Spells are stored automatically
  after a successful search.
- **Level up without a training hall.** Ignores demihuman and game level limits.
  Costs nothing. Rolls maximum hit points. Applies the constitution bonus up to
  CON 25, wisdom bonus spells, and the intelligence limit on maximum spell
  level. Handles multi-class and dual-class rules. Driven by a per-game
  `Levels.txt`. **Not available in the Buck Rogers games.**
- **Fix stats.** Sets THAC0, saving throws, and spells per level to match the
  experience level. Necessary after an over-level, or for the hacked classes.
- **Auto-identify.** Marks items identified when the item count in a character
  inventory changes. Inventory only, not loot lists.
- **Auto-ammo.** Raises arrows, bolts, and darts to a set amount. Per-game
  maximum "plus" value in `Configuration.txt`. **Not available in the Buck
  Rogers games.**
- **Auto-disable quickfight** after combat.
- **Teleport.** `Control + left click` on the map. World map teleport in the
  Krynn games, the Savage Frontier games, Pools of Darkness, and Champions of
  Krynn. **Per game.**
- **Set map event.** `Control + right click`. Not available in FRUA.
- **Character editor.** Edits abilities, class levels, saving throws, THAC0,
  armor class, money, gems, jewelry, spellbook, memorized spells, inventory,
  and effects. Also edits monsters during combat.
- **Class conversion.** Fighter to paladin or ranger, thief to monk. Pool of
  Radiance only. See section 4.
- **Explore map / unexplore map.** Reveals or hides the current map.
- **Clear explored.** Clears all explored maps for the game.
- **Weaken enemies to 1 HP.** `Control + W` during combat, in debug mode.

### 2.4 Write features against files

- Map notes. A 3-character identifier and a longer description per map cell.
- `BACKUP SAVE` copies the last saved game to `(game folder)\SAVE STORAGE`.
- Screenshots of the map.
- `Levels.txt` is editable, and `RELOAD LEVELS` re-reads it. This is a modding
  hook. It changes only GBC's level up, never the in-game training hall.

### 2.5 Interface

- Quick menu on a configurable hotkey. It carries encamp-fix, store and restore
  spells, a numbered journal entry, and level up. It is keyboard-only.
- Settings menu: icons on or off, XP meter, effects, HUD on top, title hack,
  map docking and size.

### 2.6 Per-game against universal

Universal: HUD, automap, character editor, encamp-fix, spell store and restore,
race hack, auto-identify, journals, save list, save backup, teleport on the
local map.

Per game: world map view and world map teleport, level up (absent in the Buck
Rogers games), auto-ammo (absent in the Buck Rogers games), event setting
(absent in FRUA), item editing (added for Dark Queen and FRUA only in v2.60),
the paladin, ranger, and monk conversions (Pool of Radiance only), and every
byte offset in `Resources/Character file formats/`.

Level-up bugs that GBC corrects are also per game (`GBC.txt`): the Dark Queen
of Krynn zeroed saving throws at level 19 and higher, the Dark Queen paladin
level 4 cleric spell count, the Gateway to the Savage Frontier thief skills,
and the Unlimited Adventures thief saving throws. GBC deliberately leaves the
Secret of the Silver Blades paladin experience anomaly and the Krynn spells per
day differences unchanged.

---

## 3. What `GBC_Audio.exe` does

Source: `GBC_Audio.txt`, `Music/*.txt`, `Web`, `FRUA_Module_Manager.txt`.

### 3.1 What it is

`GBC_Audio.exe` is a second build of `GBC.exe` with a music player added
(`GBC.txt`, v2.60 entry). It has every feature in section 2. The music is the
only difference. It does not work on Windows XP (`GBC_Audio.txt`).

### 3.2 Where the music comes from

GBC does not generate music, and it does not download music. The manual tells
the user to fetch a music pack by hand from a URL and to extract it into the
GBC folder (`GBC_Audio.txt`). The web page offers two packs: `gbc_music.zip`
at 40 megabytes and `gbc_music_2.zip` at 430 megabytes (`Web`).

The `Music/` subfolders in this copy of GBC are empty. That matches the
manual: no sound files ship with GBC.

**On the `curl.exe` hypothesis.** The bundled `curl.exe` is not for music.
`FRUA_Module_Manager.txt` states directly that `Tools\curl.exe` downloads the
FRUA module list and the module files, and that `Tools\7za.exe` extracts them.
No GBC document mentions any network use for audio. `GBC.txt` states that GBC
does not connect to the internet.

### 3.3 What format

Supported types, quoted from `GBC_Audio.txt`: MP3, OGG, WAV, AIFF, XM, IT,
S3M, MOD, MTM, UMX.

These are finished audio files and tracker modules. None of them is a MIDI or
XMI file, and none of them drives the game's own sound hardware.

### 3.4 How it syncs with the game

The sync is one-way and coarse. GBC already knows the game state, because it
reads DOSBox memory for the HUD. It uses that state to choose a folder, then
plays a random file from that folder (`GBC_Audio.txt`).

The mapping lives in one text file per game under `Music/`. Each file has:

- `<subfolder>` — an optional custom folder for the game.
- `<world_folder>` — played at random on the world map.
- `<combat_folder>` — played at random during combat.
- `<victory_folder>` — one random file after combat.
- `<area_NNN_folder>` — one entry per game area, each pointing at a folder.
  The shipped files point every area at `Town\` or `Dungeon\`.

Example from `Music/01. Pool of Radiance.txt`: area `000` is "Civilized Area"
and area `003` is "Valjevo Castle NW". Both point at `Town\`.

Each FRUA design gets its own configuration file, created when the design is
picked in GBC.

Control is on the HUD window: right click or `ESC` stops the music, `F1` and
`F2` change the volume. There is no visual indicator (`GBC_Audio.txt`).

The manuals are silent on beat-level or event-level sync. There is no evidence
of a cue for a door, a chest, or a spell. The granularity is the area, combat
start, and combat end.

### 3.5 Is `PCSPKR.TXT` relevant?

No, not to `GBC_Audio.exe`. `PCSPKR.TXT` describes how Unlimited Adventures
plays its own music through the PC speaker: the `PCDQ1.XMI` to `PCDQ9.XMI`
files, and a handler that reads only channel 2 of the MIDI sequence. It also
documents `SOUNDS.GLB`, the PC speaker sound effect file. `TUNES.TXT` names
the tune slots: `??DQ1` overture, `??DQ2` treasure, foes, battle, mystery,
uh-oh, evil march, `??DQ3` victory. `UASOUND.TXT` describes replacing the
digital effects file `SFXDQ.VOC`.

Those documents are about replacing the music **inside** the DOS game. GBC
takes the opposite approach: it leaves the game's audio untouched and plays
external files on the host.

### 3.6 The underlying question: can a tool add better music to the DOS game?

There are two distinct routes. The docs support both, but for different games.

1. **Host-side overlay, the GBC route.** Read game state from emulator memory,
   play modern audio files on the host, and choose the track from the current
   area. This needs no change to any game file. It works for all 12 games. On
   Linux the pieces exist already: the memory reader that Goldbox Squire is
   built around, plus any audio backend. This route is proven by
   `GBC_Audio.exe`.
2. **In-game replacement, the FRUA route.** Replace the `XMI` tune files and
   `SFXDQ.VOC` inside the game data. `PCSPKR.TXT`, `TUNES.TXT`, and
   `UASOUND.TXT` document this for Unlimited Adventures only. The hackdocs
   collection is a FRUA collection. **Hypothesis, not confirmed:** the other 11
   Gold Box games use a similar XMI-based music handler, so a similar
   replacement is possible. Nothing in these sources confirms that. Each game
   needs separate work.

Route 1 is far cheaper and does not touch the game. Route 2 changes what the
game itself emits, including through the emulator's own sound card.

---

## 4. Paladin, ranger, and export to Curse of the Azure Bonds

Source: `GBC.txt`, `Levels.txt`, `PoR format`, `CCHFORM.TXT`, `Web`.

### 4.1 Why this is possible at all

Pool of Radiance does not let the player create a paladin, a ranger, or a monk.
The engine still knows about them. The Pool of Radiance character record has a
class byte and a separate level byte for every class, including the three that
character creation refuses (`PoR format`):

| Offset (hex) | Offset (dec) | Field |
|---|---|---|
| `02F` | 047 | class |
| `096` | 150 | level cleric |
| `097` | 151 | level druid |
| `098` | 152 | level fighter |
| `099` | 153 | level paladin |
| `09A` | 154 | level ranger |
| `09B` | 155 | level mage |
| `09C` | 156 | level thief |
| `09D` | 157 | level monk |
| `0B0` | 176 | item limits |
| `06B` | 107 | attack level |

The class byte values, from the same file:

```
00 cleric   01 druid   02 fighter   03 paladin   04 ranger
05 mage     06 thief   07 monk      08 cleric/fighter ...
```

So paladin is `$03`, ranger is `$04`, and monk is `$07`.

### 4.2 What the conversion changes

`GBC.txt` states the user-visible behavior: in the character editor there are
`CONVERT TO PALADIN`, `CONVERT TO RANGER`, and `CONVERT TO MONK` buttons. A
paladin or ranger is made from a **1st level fighter**. A monk is made from a
**1st level thief**.

The manual does not print a byte-level list of the writes. The byte-level
detail below comes from the record format and from `Levels.txt`, which is the
table that GBC applies on every level up, including level 1.

`Levels.txt` for Pool of Radiance, level 1 of each class:

```
@ paladin 01
hp_add = 1d10,  thac0_base = 40,  attacks = 2,  save_1 = 12,  save_2 = 13,
save_3 = 14,  save_4 = 15,  save_5 = 15,  item_limits = 64

@ ranger 01
hp_add = 2d8,   thac0_base = 40,  attacks = 2,  save_1 = 14,  save_2 = 15,
save_3 = 16,  save_4 = 17,  save_5 = 17,  item_limits = 64

@ monk 01
hp_add = 1d8,   thac0_base = 40,  attacks = 2,  save_1 = 13,  save_2 = 12,
save_3 = 14,  save_4 = 16,  save_5 = 15,  item_limits = 0,  ac_base = 50,
unarmed_rolls = 1, unarmed_dice = 2, unarmed_modifier = 0, movement_base = 12
```

These field names map one to one onto the record offsets: `thac0_base` is
`$02D`, `save_1` to `save_5` are `$06D` to `$071`, `movement_base` is `$072`,
`attacks` is `$0A1`, the unarmed fields are `$0A3` to `$0A8`, `ac_base` is
`$0A9`, and `item_limits` is `$0B0`.

**`item_limits = 64` is the key value.** `CCHFORM.TXT` documents the same byte
in the FRUA character format as the "Item Use Indicator", a bit mask that
decides which equipment a character can use:

```
1 = Magic-User   2 = Cleric   4 = Thief   8 = Fighter
16 = Knight      32 = ???     64 = Paladin or Ranger   128 = ???
```

So the conversion writes `64` into `$0B0` to give the character the
paladin-and-ranger equipment set, and writes `0` for the monk, which is why
the manual says the monk "can not use ANY items".

The header of `Levels.txt` also raises the class ceilings:

```
@ max levels
cleric = 6, fighter = 8, mage = 6, monk = 9, paladin = 8, ranger = 8, thief = 9
```

**Summary of the change, stated as the docs support it.** GBC writes the class
byte `$02F`, sets the matching class level byte in the `$096`–`$09D` block,
and applies the level 1 row from `Levels.txt` to the stat bytes, including
`item_limits` at `$0B0`. The manual itself is silent on the exact write list.
The mapping from `Levels.txt` field names to offsets above is a
**hypothesis**, well supported by the two format documents but not stated in
any manual.

### 4.3 What is missing after the conversion

`GBC.txt` is explicit about the gaps.

- **Paladin.** The "protected from evil" effect, effect id `$08`, is not
  added. The user must add it with the Save Game Editor, with one parameter
  set to `255` for a permanent effect and the rest zero.
- **Ranger.** The "ranger bonus damage" effect is unknown in Pool of Radiance
  and possibly not implemented there. In Curse of the Azure Bonds it is effect
  `$86`. In Gateway to the Savage Frontier it is also `$86`. In the other games
  it is `$69`.
- **Sweep.** Version 2.31 added a hack that gives paladins and rangers fighter
  levels during combat, so that they get the sweep ability against low-level
  monsters. GBC removes those levels after combat. `CCHFORM.TXT` documents an
  equivalent field at FRUA offset 148, "Warrior Level, equivalent fighter level
  for sweep". The Pool of Radiance record has "attack level" at `$06B`.
  **Hypothesis:** `$06B` is the field GBC drives for sweep. The manual does not
  say which byte it writes.
- **Restoration scrolls corrupt these characters.** The manual says to use
  `FIX DRAIN` instead.
- **Training halls refuse them.** In Pool of Radiance, the GBC level up dialog
  must be used.
- **`FIX STATS` must be on**, or THAC0 and saving throws will not match the
  level.

### 4.4 The export to Curse of the Azure Bonds

There is no GBC "export" button. No manual describes one. The word "export" in
`GBC.txt` and on the web page describes the **game's own** character transfer
from Pool of Radiance to Curse of the Azure Bonds.

What makes the transfer work is that the target game accepts the class. The
Curse of the Azure Bonds character format has the identical class value list,
paladin `$03`, ranger `$04`, monk `$07`
(`Resources/Character file formats/02. Curse of the Azure Bonds.txt`). Curse
of the Azure Bonds supports paladins and rangers as real playable classes. So
a Pool of Radiance character whose class byte is `$03` arrives in Curse as an
ordinary paladin.

The web page states the outcome precisely:

> Imported paladins and rangers can be trained in training halls in Curse, but
> monks always have to use GBC to level up, and can not be further exported to
> Secret.

`GBC.txt` adds: "They can be imported to Curse of the Azure Bonds where they
should work normally. You probably need to add the effects."

So the chain is:

1. GBC converts a level 1 Pool of Radiance fighter into a paladin or ranger in
   memory.
2. The player uses GBC's level up dialog for every level, because the Pool of
   Radiance training halls refuse the class.
3. The player uses the game's own save and transfer path into Curse of the
   Azure Bonds.
4. Curse of the Azure Bonds treats the character as a native paladin or ranger.
   Training halls work. The player adds the missing effect with SGE.

The monk is a dead end. It stops at Curse of the Azure Bonds and never reaches
Secret of the Silver Blades.

**What the docs do not say.** No source here describes the transfer file
format, or which files Pool of Radiance writes for a transfer. That is a
separate investigation.

---

## 5. What GBC does not do

Every item below comes from a statement in the docs or the tutorial, not from
speculation.

**Platform and emulator.**

- GBC is Windows-only (`Web`, requirements section). There is no Linux build.
- GBC requires plain DOSBox 0.74 for non-GOG installs, and it warns that a
  change to the DOSBox sound card settings can move the data so that GBC no
  longer finds it (`GBC.txt`). It assumes a memory footprint rather than
  searching for a stable anchor.
- GBC hardcodes an address range in the search wizard. The user must widen it
  by hand when the search fails (`GBC.txt`).
- GBC breaks on specific game builds. Curse of the Azure Bonds v1.2 is listed
  as not working. One Gateway to the Savage Frontier build is listed as not
  working (`GBC.txt`).
- GBC polls the DOSBox window position constantly and can raise an "Out of
  system resources" error when the window is minimized and restored
  (`GBC.txt`, FAQ).

**Scriptability.**

- There is no command line interface, no machine-readable output, and no API in
  any manual. Every feature is a button or a hotkey. `DEBUG DUMP` writes a text
  file of addresses, and that is the only export of internal state (`GBC.txt`).
- The configuration is a bespoke `<tag> value </tag>` text format in
  `Configuration.txt` and `Levels.txt`. It is not a standard format.
- The hotkey is a Windows virtual keycode number, and a separate `VK.exe`
  exists only to discover that number (`VK.txt`).

**Data and coverage gaps stated by the author.**

- The character formats have unknown offsets. `Resources/Character file
  formats/Still unknown offsets/` has a file per game.
- Effects have unknown ids. `GBC.txt` says effect names were still being
  corrected in v2.60.
- ECL parsing is admitted to be imperfect. `ECL_Tool.txt`: "there are most
  certainly parsing mistakes", and it needs hand-maintained data offset files
  per game.
- `ECL_Tool` is not a compiler. New code can only overwrite existing code
  (`ECL_Tool.txt`).
- `IconMod` supports EGA only, and "even those don't work perfectly".
- `FontMod` does not support Treasures of the Savage Frontier.
- `MonsterMod` cannot edit spell data in Death Knights of Krynn.
- `SGE` cannot add or replace characters, and cannot add items, in The Dark
  Queen of Krynn or Unlimited Adventures.
- `SGE` cannot handle scroll bundles, in either tool. `GBC.txt` repeats the
  warning for the editor.
- The XP bar in the HUD ignores game and demihuman level limits (`GBC.txt`).
- GBC cannot detect the inventory screen, so auto-ammo can undo a deliberate
  split of an ammunition stack (`GBC.txt`).
- Auto-identify does not reach the loot list (`GBC.txt`).
- Editing effects can crash the game (`GBC.txt`).
- Teleporting can skip the event tied to a location (`GBC.txt`).
- `FRUA_Module_Manager` is described by its own manual as experimental.

**Safety.**

- `SGE.txt` states that there are no "Are you sure?" prompts, and that save
  corruption is easy. There is no undo. There is only a manual backup folder.
- Nothing validates a character record after an edit. The manual tells the user
  to eyeball the stats and judge whether the search found the right addresses
  (`GBC.txt`, editor section).

**What a Linux tool could add, grounded in the above.**

- An anchor-based memory search that does not assume a DOSBox build or a
  memory footprint. This removes the DOSBox 0.74 requirement and the address
  range field.
- Machine-readable output and a command line, which no GBC tool offers.
- Record validation, so a bad search or a bad edit is caught rather than shown.
- Undo, or an automatic snapshot before each write, replacing the manual
  backup.
- One tool instead of 13, sharing one DAX codec and one set of record tables.
- Data tables that ship as data rather than as hardcoded offsets, so that a new
  game or a new build is a table change.

---

## Open questions

1. The exact byte writes performed by `CONVERT TO PALADIN` are not documented.
   Confirmation needs either a memory diff against a live game or a look at the
   GBC binary.
2. The Pool of Radiance to Curse of the Azure Bonds transfer file format is not
   documented in any source read here.
3. Whether the 11 non-FRUA games use XMI music files, as FRUA does, is not
   established by these sources.
4. Which byte carries the sweep fighter level in Pool of Radiance. `$06B`
   "attack level" is the candidate.
