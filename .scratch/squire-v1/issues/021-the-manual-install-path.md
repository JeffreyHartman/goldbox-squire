# 021 — The manual install path

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 019

## What to build

A user with a hand-written DOSBox conf (Jeff is one) passes `--conf` and
`--game-dir` once. That does not bypass the wizard; it feeds it: the pair is
remembered as a manual install in the config, appears in the wizard's install
list with kind `manual`, and is the default next run like any other install.

Two guards that only this path needs:

- The folder-name check. The game's own DOS config pins where it reads data
  and writes saves (`POOL.CFG` line 3 says `C:\POOLRAD\`; the file name and
  line come from the game registry, 014). If the save folder's name does not
  match that path's leaf, refuse before launching, and say exactly what is
  wrong and the two fixes: rename the folder, or adjust the conf's mount. A
  discovered install cannot have this mismatch, so the check runs only here.
- The first-run note. The first time an install is used, name the conf files
  being launched, with full paths, and say emulator settings live there and
  are the user's to edit. One line, printed once per install.

## Acceptance criteria

- [x] `gbs --conf F --game-dir D` runs, and the pair appears in the wizard
      list on the next bare `gbs`.
- [x] The migrated flat config from 015 behaves identically to a fresh
      `--conf`/`--game-dir` pair.
- [x] A game folder named `por` with a DOS config saying `C:\POOLRAD\` is
      refused with a message naming both and both fixes.
- [x] The first run of an install names its conf files; later runs do not.

## Answer

`--conf`/`--game-dir` feed `Config::remember_manual` (built in 015), which
stores the pair as `manual:pool-of-radiance` and makes it the default; a
test proves the migrated v1 config lands byte-for-byte where a fresh pair
does. `squire_cli::manual::folder_name_check` runs only on kind `manual`: it
reads the registry-named DOS config (case-insensitively), takes the
registry-named line, compares that path's leaf with the save folder's name,
and refuses a mismatch naming the folder, the config line, and both fixes
(rename the folder, or adjust the conf's mount). A missing or short config
proves nothing and passes. `manual::first_run_note` prints the conf files
with full paths once per install, gated by the `introduced` flag, which is
then saved.
