# 003 — Research: what every GBC tool does

Type: `wayfinder:research` (AFK)
Status: CLOSED 2026-08-22
Blocked by: none

## Question

Jeff wants the roadmap to cover everything GBC does. Produce the full inventory
from GBC's own documentation, which is local:
`~/goldbox/gbc/*.txt` (13 files) and <https://gbc.zorbus.net/>.

For each of the 13 tools, record what it does, whether it reads process memory
or only edits files, and how large a job it would be to replace.

Answer specifically:

- What does `GBC_Audio.exe` do? Jeff asked about music support and does not know
  what GBC uses. `~/Downloads/hackdocs.zip` contains `PCSPKR.TXT`, a write-up on
  Gold Box PC speaker sound, which may be relevant.
- What do the paladin, ranger, and export-to-Curse-of-the-Azure-Bonds features
  actually change? Jeff wants these.

## Answer

Full write-up: [`.scratch/squire-v1/research/003-gbc-tool-inventory.md`](../research/003-gbc-tool-inventory.md)

Gist:

- **13 executables.** Four touch process memory: `GBC.exe`, `GBC_Audio.exe`,
  `ECL_Monitor.exe`, `FRUA_Tool.exe`. The other nine are file editors. The DAX
  codec is the shared dependency behind most of the file editors.
- **`GBC_Audio.exe` is `GBC.exe` plus a music player.** The music is host-side
  audio, not game audio: MP3, OGG, WAV, and tracker formats. The user downloads
  a pack by hand. GBC does not generate or download it. Sync is coarse: GBC
  reads the game state it already reads for the HUD, picks a folder, and plays a
  random file from it. One config maps world, combat, victory, and per-area
  folders.
- **`PCSPKR.TXT` is not related.** It covers replacement of FRUA's own `.XMI`
  files inside the DOS game. That is the opposite approach.
- **Paladin and ranger already exist in the Pool of Radiance engine.** The class
  byte at `$02F` holds paladin `$03` and ranger `$04`, and separate level bytes
  exist at `$099` and `$09A`. Only character creation refuses them. GBC converts
  a level 1 fighter and writes the level 1 row from `Levels.txt`, including the
  Item Use Indicator at `$0B0`, where 64 means "Paladin or Ranger". Two pieces
  stay manual: the paladin protection effect and the ranger damage bonus.
- **There is no GBC export button.** "Export" is the game's own Pool of Radiance
  to Curse of the Azure Bonds transfer. It works because Curse uses the same
  class values and supports both classes natively.
- **Documented GBC gaps:** Windows only, a hard assumption of DOSBox 0.74, a
  hardcoded address range, no command line and no machine-readable output, no
  undo, no confirmation prompts, admitted ECL parsing errors, and EGA-only icon
  editing.
