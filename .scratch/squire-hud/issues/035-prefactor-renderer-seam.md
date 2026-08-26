# 035 — Prefactor: the watch loop gets a renderer seam

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: none

## What to build

Nothing changes for the user. `gbs` runs exactly as it does today, prints the
same table, answers Enter the same way, and every existing test passes
unaltered.

What changes is that the thing which draws is no longer welded to the loop
which polls. Today one long function in the binary polls the session, decides
whether the party was ever found, clears the screen, prints, and listens for a
keypress, all in one place, with the escape sequences written inline. A second
way of drawing cannot be added to that without copying it.

Separate the loop from the drawing, so that the printed table is one
implementation of drawing and the HUD can be another. Make the change easy
first; the easy change is 037.

The loop also lives in the binary rather than the library, which is why none of
it is tested today. Moving it into `squire-cli`'s library is part of this.

## Acceptance criteria

- [x] `gbs` behaves identically: same output, same keys, same messages, same
      exit conditions
- [x] The polling loop is in the library, not the binary, and has tests
- [x] Drawing is reached through one seam that a second implementation can
      satisfy without touching the loop
- [x] No escape sequence is written inline in the loop
- [x] The full test suite passes with no test changed to accommodate the move

## Answer

The loop is `squire_cli::watch::watch`, in the library, with nine tests in
`squire-cli/tests/watch.rs`. `main.rs` keeps only the wiring: it builds the
timings, the screen and the keyboard, and calls the loop.

Two seams, not one.

- `Screen` is the drawing seam the ticket asked for. Two methods: a party, and
  a notice. `output::Plain` is the printed implementation, and it owns the
  clear-screen escape and the `gbs: ` prefix, because both are decisions about
  how the party is shown. The HUD is the second implementation.
- `Keys` is the pause. The loop's wait and its ear for Enter were always one
  wait, so splitting them would have meant a thread. `keys::Stdin` is the real
  one and owns the repick question; a test hands the loop a scripted keyboard
  and the whole loop runs in microseconds with no clock and no terminal.

`Alive` is a one-method trait over `Launched`, so a test can end the loop after
a fixed number of polls.

**One behaviour did change, and it is a fix.** Under `--pid`, and after stdin
hit end of file, the old loop skipped the pause entirely and swept the
emulator's memory as fast as the CPU allowed. `keys::Stdin` sleeps the interval
when there is nobody to listen to. Nothing else moved: same output, same keys,
same messages, same exit conditions, and no existing test was edited.
