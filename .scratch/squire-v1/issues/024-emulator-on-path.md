# 024 — Find the emulator on PATH by its common names

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

The system emulator is the one that runs a discovered install (ADR 0003).
Find it on PATH under its common names, in order of preference: `dosbox`,
then `dosbox-staging`, then `dosbox-x`. On Arch-likes the first two conflict
over the `dosbox` name, but other distros and hand installs use the longer
names, and dosbox-x always installs as `dosbox-x`.

`--dosbox` overrides the search. A manual install's stored emulator choice
still wins for that install. When nothing is found and nothing is stored,
the error names the three names and the `--dosbox` escape hatch.

## Acceptance criteria

- [x] Each of the three names is found when it is the one present.
- [x] `dosbox` is preferred when several are present.
- [x] A manual install's stored emulator wins over the PATH find.
- [x] No emulator anywhere is an error naming the three names and `--dosbox`.

## Answer

`squire_cli::emulator::find_on_path` tries the three names in preference
order across the whole PATH, so `dosbox` in a late folder beats
`dosbox-staging` in an early one. `Install::emulator_command` takes the find
result: a manual install's stored choice wins, then the system emulator. A
discovered install takes the system emulator only; a stored bundled path from
an old config is never launched (ADR 0003, tightened during code review).
Nothing found is an error naming the three names and `--dosbox`, and
`--dosbox` on the command line bypasses all of it.
