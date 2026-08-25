# 018 — Watch is what the tool does

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 013, 014

## What to build

Today a bare `gbs` starts the emulator, scans a game that cannot be loaded
yet, prints "No party in memory", and exits 0. `--watch` is the only useful
mode, and an argument required to make the program work is not an argument.

Remove `--watch` and the once-mode entirely. The tool starts the game, waits
for a party, then redraws until the emulator exits or the user stops it.

- While no party has been found yet, poll slowly (around every two seconds),
  because each failed poll is a full memory sweep through DOS boot, title
  screen, and load menu. After the first success, poll at `--interval`.
- The emulator exiting during the watch is a clean stop: say so on stderr and
  exit 0. Every other read failure stays fatal and non-zero, so a permission
  error stays loud.
- Add `--game <ID>` and `--slot <A-J>`. Each answers a wizard question in
  advance (the wizard itself is 019). Validate `--slot` against the populated
  slots and error with the populated list if it is not one of them.
- Update the usage text to the new surface. `--game-dir` and `--conf` are
  documented as the manual escape hatch, not the front door.

## Acceptance criteria

- [x] `--watch` is gone; an unknown-option error names `--help`.
- [x] A bare run waits for a party rather than declaring failure at launch.
- [x] Emulator exit ends the run with status 0 and a message on stderr.
- [x] A permission error still exits non-zero with the existing guidance.
- [x] `--slot B` against a folder with only A and J populated errors and names
      A and J.
- [x] `--json` and `--interval` behave as before.

## Answer

`--watch` and the once-mode are gone; `Mode` itself is gone and `--help` is
a bool. The tool launches, polls every two seconds until the first party
appears, then redraws at `--interval`. The emulator ending (checked via the
child handle, and via `NoSuchProcess` on the `--pid` path) prints a clean
stop on stderr and exits 0; every other read failure stays fatal. `--game`
and `--slot` answer the wizard's questions in advance; `--slot` is validated
against the populated slots and errors naming them. Until 019 lands, a bare
run with several populated slots errors naming them instead of asking.
