# 037 — The HUD draws the party and reflows as you resize

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 035, 036

## What to build

Running `gbs` gets you the live party on a screen you can glance at, rather
than a table reprinted into your scrollback. Drag the window narrower and
fields drop away in the settled order. Drag it wider and they come back. Nothing
is configured and no flag is passed to get this.

`--plain` gets the old printed table, for pipes, scripts and anything reading
`gbs` as text. The tool's own argument parser already argues that an argument
required to make the program work is not an argument, which is why there is no
`--watch`; the same reasoning makes the HUD the default rather than a `--tui`
opt-in.

The drawing code asks 036 what to show and draws that. It makes no layout
decisions of its own. If a field's presence is decided in the drawing code,
this ticket is done wrong.

From the spike, at `/home/jeff/git/goldbox-squire-temp`: the Gold Box palette
and the double gold rule come across. The sixteen-column tab list, the
always-on wordmark, and the layout code do not.

No left menu. The panel name and the game live on the status line, which is
needed anyway and costs one row rather than sixteen columns.

## Acceptance criteria

- [ ] `gbs` with no arguments shows the party in a full-screen terminal
      interface
- [ ] `--plain` produces the pre-existing printed table, unchanged
- [ ] Resizing the terminal reflows immediately, with fields dropping and
      returning in the order 036 encodes
- [ ] The interface is usable at a hostile size without panicking, truncating
      mid-character, or drawing outside its area
- [ ] No sixteen-column menu, and no wordmark except when roomy
- [ ] The terminal is restored on exit, including after an error
- [ ] Drawing code contains no rule about what to show
