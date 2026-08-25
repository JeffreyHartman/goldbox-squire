# 004 — GOG and Steam install layouts of the Gold Box games

Question: for each SSI Gold Box game as GOG and Steam ship it today, which
directory does the game live in under the mounted C: drive, which file starts
it, and does it ship a config file that pins its DOS paths?
Date: 2026-08-24. Pool of Radiance and Unlimited Adventures were already
known and appear only as calibration rows.

Evidence classes used below:
- `[Steam]` — read directly from the installs on this machine:
  `~/.steam/steam/steamapps/common/Forgotten Realms The Archives - Collection Two/`
  and `.../Dungeons & Dragons Krynn Series/`. **High confidence.**
- `[GOG-depot]` — file trees of the current GOG Windows builds, read from
  gogdb.org manifest listings (build IDs in Sources). Names and byte sizes are
  exact; file contents are not visible. **High confidence for names/layout.**
- `[GOG-size]` — CFG contents reconstructed from the depot byte size plus the
  known line format; the arithmetic is byte-exact but still inference.
  **Medium confidence.**
- `[Community]` — DOSBoxWiki / forum statements. **Low-medium confidence.**

## Summary

| Game | GOG dir under C: | Steam dir under C: | Start file | Config file |
|---|---|---|---|---|
| Curse of the Azure Bonds | `\` (root) | `\CURSE` | `START.EXE` | `CURSE.CFG` |
| Secret of the Silver Blades | `\` | `\SECRET` | `START.EXE` | `BLADES.CFG` |
| Pools of Darkness | `\` | `\DARKNESS` | `START.BAT` → `GAME.EXE` | `POOL4.CFG` |
| Champions of Krynn | `\` | `\COK` | `START.EXE` | `KRYNN.CFG` |
| Death Knights of Krynn | `\` | `\DKK` | `START.EXE` | `DKK.CFG` |
| The Dark Queen of Krynn | `\` (+ `DISK1..3`) | `\DQK` (+ `DISK1..3`) | `START.BAT` → `DQK.EXE` | `DISK1\DQK.CFG` |
| Gateway to the Savage Frontier | `\` | `\GATEWAY` | `GO.BAT` → `GAME.EXE` | `GAME.CFG` |
| Treasures of the Savage Frontier | `\` | `\TREASURE` | `START.BAT` → `GAME.EXE` | `TREASURE.CFG` |
| Buck Rogers: Countdown to Doomsday | — not sold | — not sold | `START.EXE` | `BUCK.CFG` |
| Buck Rogers: Matrix Cubed | — not sold | — not sold | `START.EXE` | `MATRIX.CFG` |
| (Pool of Radiance) | `\POOLRAD` | `\POOLRAD` | `START.EXE` | `POOL.CFG` |

## 1. GOG layout: the install root is the game directory

Every current GOG Windows build except Pool of Radiance puts the DOS files
flat in the install root — `START.EXE`, `GAME.OVR`, the DAX files, and the
CFG file all sit beside `dosbox_*.conf` and the `DOSBOX\` folder.
`[GOG-depot]` No per-game subfolder exists, so under the mounted C: the game
runs from `C:\`. GBC's readme confirms it from the other side: only "in the
GOG-version of Pool of Radiance, you'll have to select the POOLRAD-folder as
the game folder" (`docs/gbc/tool-docs/GBC.txt`). Saves live in a root-level
`SAVE\` folder that the conf depot ships pre-seeded (`save/SAVE/CHRDATA*` in
every conf depot). The Dark Queen of Krynn keeps its three data-disk folders
`DISK1`, `DISK2`, `DISK3` under the root, with `DQK.EXE`, `START.BAT`, and
`INSTALL.EXE` beside them. `[GOG-depot]`

Per-game GOG conf names (verified in both the depots and GBC's `Game.dat`
files): `dosbox_coab`, `dosbox_sotsb`, `dosbox_pod`, `dosboxChampionsOfKrynn`,
`dosbox_krynnKnights`, `dosbox_krynnQueen`, `dosbox_gateway`,
`dosbox_treasures` — each as `X.conf` plus `X_single.conf`, where the tiny
`_single.conf` (82 B – 1.0 KB) holds the `[autoexec]`. The GOG Linux Pool of
Radiance on this machine shows the pattern: `mount c "data"` / `c:` /
`cd POOLRAD` / `start` / `exit`. For the flat games there is no `cd`.
I did not capture the flat games' autoexec text verbatim, so the exact start
command is inferred from the shipped launch files: **medium confidence**, and
unambiguous everywhere except Gateway (see §3).

## 2. Steam layout: SNEG nests the original DOS tree one level down

Verified on disk. `[Steam]` Each game lives at
`<collection>/games/<Game name>/` holding `base.conf`, `graphics.conf`,
`game.conf`, `run-game.bat`, `gbc.bat`, `DOSBOX\` (vanilla DOSBox 0.74-3),
`GBC\`, `Default files\`, `Documentation\`, and `GAME\`. `run-game.bat` runs
`DOSBox.exe -noconsole -conf base.conf -conf graphics.conf -conf game.conf`.
`game.conf` holds only `[autoexec]`: `mount c .\GAME`, `config -securemode`,
`c:`, `cd <DIR>`, then the start command. The `<DIR>` names are the summary
table's Steam column — these are the classic SSI default directory names
(`CURSE`, `SECRET`, `DARKNESS`, `COK`, `DKK`, `DQK`, `GATEWAY`, `TREASURE`,
`POOLRAD`, `UA`). Sequels ship a stub of the predecessor next to their own
folder for party import (`GAME\SECRET\SAVE` inside Pools of Darkness,
`GAME\GATEWAY\SAVE` inside Treasures, `GAME\COK\SAVE` inside Death Knights,
`GAME\DKK\SAVE` inside Dark Queen). Steam cloud saves sync from
`<collection>/SavesDir/<steamid>/<appid>/`.

## 3. Start files

- **Curse, Secret, Champions, Death Knights: `START.EXE`.** In the depots and
  the Steam autoexecs (`@start` / `START`); Curse and Secret `INSTALL.BAT`
  even say "Run START.EXE to start playing". **High.**
- **Pools of Darkness and Treasures: `START.BAT`**, which runs `STARTUP.EXE`
  (intro), then `CONTROL.EXE`, which launches the engine `GAME.EXE` — the
  binary GBC watches. Steam's autoexec does `call START.BAT`; GOG ships the
  same 82/83-byte batch. **High for Steam, medium for GOG's start command.**
- **Gateway: `GO.BAT`** (`sblaster` / `game UseStart` / `sblaster U`) →
  engine `GAME.EXE`. Steam's autoexec calls `GO.BAT`. Steam also ships
  `START.BAT` (`start1` then `go` — `START1.EXE` is the intro), but the GOG
  depot has `GO.BAT` and `START1.EXE` and **no** `START.BAT`, so GOG must
  launch via `GO.BAT` too. **High for Steam, medium for GOG.**
- **Dark Queen: `START.BAT`**, one line: `dqk SB220 SI7 RB330` — `DQK.EXE`
  with sound parameters. Same 35-byte file in the GOG depot. **High.**

These agree with GBC's per-game `Game.dat` engine binaries (`start.exe`,
`game.exe`, `dqk.exe`, `ckit.exe`) extracted from `docs/gbc/Games/`.

## 4. The config files

All are written by the game's setup and pin DOS paths. Steam contents read
directly `[Steam]`; GOG contents are `[GOG-size]` reconstructions that match
the depot byte size exactly.

| File | GOG size | Pinned paths (Steam value / GOG reconstruction) |
|---|---|---|
| `CURSE.CFG` | 19 B | line 3 save path: `C:\CURSE\SAVE\` / `C:\SAVE\` |
| `BLADES.CFG` | 64 B | line 4 save path: `C:\SECRET\SAVE\` / `C:\SAVE\` |
| `POOL4.CFG` | 81 B | lines 4–5 save + Secret-import path: `C:\DARKNESS\SAVE\`+`C:\SECRET\SAVE\` / `C:\SAVE\`+`C:\BLADES\SAVE\` |
| `KRYNN.CFG` | 64 B | line 4 save path: `C:\COK\SAVE\` / `C:\SAVE\` |
| `DKK.CFG` | 77 B | lines 4–5 save + CoK-import path: `C:\DKK\SAVE\`+`C:\COK\SAVE\` / `C:\SAVE\`+`C:\SAVE_CH\` (GOG ships a `SAVE_CH\` stub) |
| `DQK.CFG` | 144 B | fixed-width binary in `DISK1\`: sound fields, DKK-import path (`C:\DKK\SAVE` + `_KN` on Steam; GOG ships `SAVE_KN\`), and the data root (`C:\DQK` on Steam) |
| `GAME.CFG` (Gateway) | 76 B | line 3 save path plus **two data paths** on lines 5–6: `C:\GATEWAY\SAVE\`+`C:\GATEWAY\`×2 / `C:\SAVE\`+`C:\`×2 |
| `TREASURE.CFG` | 82 B | lines 4–5 save + Gateway-import path: `C:\TREASURE\SAVE\`+`C:\GATEWAY\SAVE\` / `C:\SAVE\`+`C:\GATEWAY\SAVE\` (GOG ships a `GATEWAY\SAVE\` stub) |

Only Pool of Radiance (`POOL.CFG` line 3) and Gateway (`GAME.CFG` lines 5–6)
pin the game's **data** path; Dark Queen pins its data root inside the binary
`DQK.CFG`. Every other file pins save paths only, so those games find their
data in the current directory.

## 5. Buck Rogers

Neither Buck Rogers game is sold on GOG or Steam; the license sits with the
Dille estate, not WotC (GOG wishlist page; Steam Collection One discussion
thread). **High.** For manual installs: both start with `START.EXE` (GBC
`Game.dat`; DOSBoxWiki says "run START.EXE" for Matrix Cubed). Config files
`BUCK.CFG` (Countdown) and `MATRIX.CFG` (Matrix Cubed) hold the save path
`[Community]`, **medium**. There is no publisher folder name; community
setups use `C:\BUCK` and `C:\MATRIX` `[Community]`, **low** — squire should
treat the directory as user-chosen for these two.

## Sources

- Steam installs on this machine (read 2026-08-24): Collection Two
  (app 1882280) and Krynn Series (app 1904610) under
  `~/.steam/steam/steamapps/common/`.
- GOG Windows build manifests on gogdb.org: Curse 52095402405777333, Secret
  52112735097730082, Pools of Darkness 52095541401974798, Champions
  52095391730271042, Death Knights 52095427753782728, Dark Queen
  52095626164514469, Gateway 52095461861559797, Treasures 52095635462262502
  (products 1432642138, 1432641528, 1432643408, 1432722131, 1432722599,
  1432722998, 1432649588, 1432641771).
- Local GOG Linux Pool of Radiance: `~/goldbox/pool-of-radiance/`
  (`dosbox_por_single.conf`, `data/POOLRAD/POOL.CFG`).
- GBC v2.65: `docs/gbc/tool-docs/GBC.txt` and the per-game
  `docs/gbc/Games/*/Game.dat` binaries (conf names, engine binaries,
  `cloud_saves` paths, GOG product IDs).
- Buck Rogers: `https://www.dosbox.com/wiki/GAMES:Buck_Rogers:_Matrix_Cubed`,
  `https://www.gog.com/wishlist/games/buck_rogers_countdown_to_doomsday`,
  dosgames.com forum thread 26246.
