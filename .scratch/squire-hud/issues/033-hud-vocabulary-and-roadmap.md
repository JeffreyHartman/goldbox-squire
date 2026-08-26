# 033 — Write down the HUD vocabulary and correct the roadmap

Type: `wayfinder:task` (AFK)
Status: open
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

- [ ] `CONTEXT.md` defines HUD as the concept, and TUI as the terminal
      implementation of it
- [ ] `CONTEXT.md` defines the layout plan: what is shown at a given size
- [ ] "Dock" appears nowhere in the repo as a name for the HUD
- [ ] The roadmap's "The HUD" section describes one rules-driven layout, not
      three named ones
- [ ] The roadmap says Wayland forbids self-placement, that Squire sets an
      app-id, and that a compositor rule is the user's one-time answer
- [ ] Nothing in `CONTEXT.md` names a module, a file, or a number of columns
