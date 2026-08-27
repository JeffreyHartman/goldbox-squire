# 050 — A real terminal proves the keys work

Type: `wayfinder:task`
Status: open
Triage: `needs-grilling`
Blocked by: none

## What to build

A test that types into a running view and checks the HUD acted on it. Every
test Squire has stops below the terminal, so nothing it owns has ever pressed
a key or resized a window.

## Why

The keys in the spawned HUD window did nothing at all, and 31 test binaries
were green while it was broken. The cause was a second reader draining the
terminal's event queue and throwing keypresses away. No test below the
terminal can see that, because the fault was entirely in the terminal.

## What is not yet decided

This is filed as a question, not as work to pick up. The shape has to be
settled before it is worth writing:

- **What drives it.** A pseudo-terminal opened by the test, with the view as
  its child, is the smallest thing that works and has no new dependency. A
  terminal test harness crate is the other option and brings assertions on
  what is on screen.
- **What it asserts.** That the socket carried a quit after `q` is the
  cheapest true assertion and needs no screen scraping. Asserting on drawn
  cells is what would also catch a reflow that stopped happening.
- **Whether it runs in CI.** A test that needs a pseudo-terminal may need
  marking `#[ignore]` and a named way to run it.
- **How far it goes.** Driving a view against a real host over a real socket
  is one test. Driving a real terminal emulator, kitty or foot, opened by
  `spawn::open`, is a much larger one and probably not worth it.

## Acceptance criteria

- [ ] A test presses a key at a running view and the host sees the result
- [ ] A test resizes the window and the HUD reflows
- [ ] The old fault, a swallowed keypress, makes it fail
- [ ] `cargo test` on a plain clone either runs it or says how to
