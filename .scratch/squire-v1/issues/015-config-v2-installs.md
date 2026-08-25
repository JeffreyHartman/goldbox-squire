# 015 — Config v2: installs, not one game folder

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 014

## What to build

The config holds one flat `game_dir`/`dosbox`/`conf`. As soon as two games or
two installs exist, every switch overwrites the other's settings. Replace the
flat keys with a map of installs plus the last choice, so the wizard can
default every question to Enter.

Shape (settled during design, keep the meaning if not the letter):

```toml
last_install = "steam:pool-of-radiance"

[installs."steam:pool-of-radiance"]
game = "pool-of-radiance"
kind = "steam"            # gog | steam | manual
root = "/path/to/install"
saves = "GAME/POOLRAD"    # relative to root
confs = ["base.conf", "graphics.conf", "game.conf"]   # ordered
emulator = "dosbox"       # optional override

extra_roots = []          # extra folders discovery searches
```

A config written by the current version migrates on load: its `game_dir` and
`conf` become one manual install, so an existing user upgrades without typing
anything.

The save slot is never stored. A slot describes one sitting, and a remembered
slot would pin the user to a save they stopped playing, which is the bug this
whole effort removes.

## Acceptance criteria

- [x] Old flat config loads as one manual install and round-trips.
- [x] Installs are keyed, ordered confs preserved, `last_install` remembered.
- [x] No field anywhere in the config stores a save slot.
- [x] A missing or unreadable file still gives defaults, as today.

## Answer

The config is `last_install` plus a keyed map of installs, with
`extra_roots` for discovery. `Config::from_toml` reads both file versions: a
v1 file's `game_dir`/`conf`/`dosbox` become the manual install
`manual:pool-of-radiance` with the game folder as root and an empty `saves`
(the old game_dir was the save folder). `Install::save_dir()` joins root and
saves. `remember_manual` replaces `merge`: `--game-dir`/`--conf`/`--dosbox`
update or create the manual install and set `last_install`. Nothing stores a
slot, and a test asserts the written file never mentions one.
