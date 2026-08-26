# 033 — Write down the HUD vocabulary and correct the roadmap

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: none

## What to build

Four words were used for one idea while this work was designed: HUD, TUI, dock,
and window. `CONTEXT.md` holds none of them, so the code is free to grow a
`hud.rs` beside a `tui/` beside a `dock` field, and a reader has no way to know
those are the same thing.

`CONTEXT.md` gains the settled vocabulary. `docs/roadmap.md`'s "The HUD"
section is corrected: it promises three named layouts, which this effort ruled
out, and it does not mention that Squire can name its own window so that a
compositor rule can find it.

`CONTEXT.md` is a glossary and nothing else. No offsets, no module names, no
decisions. The decisions live in the map.

## Acceptance criteria

- [x] `CONTEXT.md` defines HUD as the concept, and TUI as the terminal
      implementation of it
- [x] `CONTEXT.md` defines the layout plan: what is shown at a given size
- [x] "Dock" appears nowhere in the repo as a name for the HUD
- [x] The roadmap's "The HUD" section describes one rules-driven layout, not
      three named ones
- [x] The roadmap says Wayland forbids self-placement, that Squire sets an
      app-id, and that a compositor rule is the user's one-time answer
- [x] Nothing in `CONTEXT.md` names a module, a file, or a number of columns

## Answer

`CONTEXT.md` gained a **Showing** section: HUD, TUI, plain output, layout plan,
drop order, roomy, wordmark, app id. It stays a glossary. No module, file, or
column count is named in it.

The roadmap's "The HUD" section now says Wayland forbids self-placement, that
Squire sets an app id so one compositor rule answers placement forever, and that
there is one rules-driven layout rather than Wide, Thin and tall, and Fullscreen.
The "Squire only" list said "A TUI" and "A GUI"; it now says the HUD drawn as a
TUI, and a pixel HUD later.

"Dock" appears in the repo only in this effort's map, saying the project does not
use the word, and in a v1 research file describing GBC's own map docking.
