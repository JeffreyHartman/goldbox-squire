# 019 — The wizard

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 013, 015, 016, 018

## What to build

A bare `gbs` asks its questions instead of demanding flags. Two questions, in
order, then launch and watch:

1. **Which game?** One numbered entry per discovered install: display name,
   install kind, path. The remembered last choice is the default; Enter
   accepts it.
2. **Which save slot?** One entry per populated slot, showing the letter and
   the party's names (see ADR 0002), read via the slot enumeration from 013.
   Picking by recognising your party beats remembering a letter.

Rules that make it unobtrusive:

- An argument answers its question in advance and the question is skipped.
  `--game` answers question 1, `--slot` answers question 2.
- Every question with a remembered or discoverable default accepts Enter. A
  returning user is two Enters from a running game.
- An empty line where there is no default, or a lone `b`, goes back one
  question. No raw terminal mode; input is plain lines ended by Enter.
- No installs discovered: say what was searched and point at the manual path
  (`--conf` plus `--game-dir`).
- The save slot is asked every run and never remembered.

## Acceptance criteria

- [x] First run: pick install by number, pick slot by letter, game launches.
- [x] Second run: Enter, Enter, game launches.
- [x] `gbs --slot J` asks only the install question.
- [x] `gbs --game pool-of-radiance --slot J` asks nothing.
- [x] The slot list shows party names next to each letter.
- [x] Wizard prompts are testable without a terminal (lines in, lines out).

## Answer

`squire_cli::wizard::choose` takes any `BufRead` and `Write`, so the tests
drive it with strings. Question 1 numbers the installs (display name, kind,
root) and defaults to the remembered last choice, or to the only install;
question 2 lists populated slots with party names and defaults to the first.
`--game` and `--slot` each skip their question; both given asks nothing. A
lone `b`, or an empty line with no default, goes back. `main` runs discovery
before the wizard when the config holds no installs or a stored root
vanished, saves the absorbed results, names the searched roots plus the
manual path when nothing was found, and remembers the pick as
`last_install`. Prompts go to stderr so `--json` output stays clean.
