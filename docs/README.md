# docs/

Two kinds of thing live here, and `.gitignore` splits them.

**Committed, and ours.** `adr/` holds the decision records. `roadmap.md` lists
every feature the program might grow. `agents/` tells the agent skills where the
issue tracker and the domain docs are. `findings/` holds our
reverse-engineering write-ups.

**Gitignored, and not ours.** `gbc/` and `hackdocs/` are third-party reference
material, so they stay out of the repository. They are present on Jeff's
machines only. To put them on a new machine, see "How to rebuild" at the end of
this file.

## hackdocs/

57 files from the Unlimited Adventures Hacker's Guide. These describe the file
formats and the data structures of the Gold Box engine.

Start with these files:

- `CCHFORM.TXT` — the character record format
- `SAVGAM.TXT` — the save game format
- `ITEM.TXT` — the item record format
- `SPELLEFF.TXT` — spell effects
- `OPCODES.TXT` — the ECL script opcodes
- `PCSPKR.TXT` — the PC speaker music format

Source: `~/Downloads/hackdocs.zip`

## gbc/

Reference data from Gold Box Companion v2.65. The executable files are removed.
Only the documentation and the data tables remain.

- `tool-docs/` — the manual for each of the 13 GBC tools, one text file each.
  `GBC.txt` is the main one. `GBC_Audio.txt` describes the music tool.
- `Resources/Character file formats/` — the character record offsets for all
  twelve games. `01. Pool of Radiance.txt` is our first target.
  `Still unknown offsets/` lists the fields that GBC never identified.
- `Resources/Effects/`, `Resources/Item lists/`, `Resources/Monster Manuals/` —
  lookup tables.
- `Games/` — per-game data. Each folder holds item files, icons, level tables,
  experience tables, and the ECL tool tables.
- `Music/` — one text file for each game, plus empty folders for the audio
  tracks that GBC downloads at run time.
- `Data/Configuration.txt` — the GBC configuration format.
- `Tutorial/index.html` — the GBC user tutorial with screenshots.

Source: `~/goldbox/gbc/` (extracted from `~/Downloads/gbc.zip`)

## findings/

Our own write-ups from the earlier reverse-engineering sessions.

- `FINDINGS.md` — the full report. Section 6 holds the confirmed facts about the
  character record in live memory: the 285-byte size, the deterministic layout,
  and the current-HP offset at 283.
- `gbc-wine-handoff.md` — the brief that started the Wine work.

## How to rebuild

```
cd ~/git/goldbox-squire
mkdir -p docs/{hackdocs,gbc}
unzip -q -o ~/Downloads/hackdocs.zip -d docs/hackdocs
cp -r ~/goldbox/gbc/{Resources,Games,Music,Tutorial} docs/gbc/
mkdir -p docs/gbc/tool-docs docs/gbc/Data
cp ~/goldbox/gbc/*.txt docs/gbc/tool-docs/
cp ~/goldbox/gbc/Data/{Configuration.txt,VK.txt} docs/gbc/Data/
find docs -type f \( -iname '*.exe' -o -iname '*.dll' \) -delete
```
