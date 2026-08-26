# 037 — The HUD draws the party and reflows as you resize

Type: `wayfinder:task` (AFK)
Status: resolved
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

- [x] `gbs` with no arguments shows the party in a full-screen terminal
      interface
- [x] `--plain` produces the pre-existing printed table, unchanged
- [x] Resizing the terminal reflows immediately, with fields dropping and
      returning in the order 036 encodes
- [x] The interface is usable at a hostile size without panicking, truncating
      mid-character, or drawing outside its area
- [x] No sixteen-column menu, and no wordmark except when roomy
- [x] The terminal is restored on exit, including after an error
- [x] Drawing code contains no rule about what to show

## Answer

`squire-cli/src/hud/`, three files: `mod.rs` holds the terminal and the two
watch-loop seams, `draw.rs` turns a plan into cells, `theme.rs` is the palette
carried over from the spike. Tests are `squire-cli/tests/hud.rs`, drawn into a
`Buffer` with no terminal anywhere.

`gbs` opens the HUD. `--plain` prints the table, and `--json` implies it. There
is no `--tui`.

**One thing was added that the ticket did not ask for.** When standard output
is not a terminal, `gbs` prints the table and says so on standard error. A HUD
cannot take over a pipe, and `gbs | head -1` working is a thing ticket 035's
review already fought for. The message is what keeps it from being a hidden
behaviour.

**`draw.rs` decides colour and nothing else.** It asks the plan which lines a
card has and where the cards go, and it draws that. Its only judgements are
which colour a hit point line gets from how many points are left, and which
one a condition gets from whether it is `okay`. Both come from the numbers,
never from the size.

The terminal is restored from a `Drop` impl and from a panic hook that chains
to the one it replaced, so an error and a panic both leave a usable shell.

## Review, answered

Two-axis review, 2026-08-25.

- `Hud::start` could return an error with raw mode still on, because the two
  failures between the takeover and `Inner` existing had no `Drop` to hang
  from. Both undo the takeover by hand now.
- `draw.rs` did plain `u16` subtraction to place the wordmark, using a plan
  made for the size the terminal last reported. A resize landing between the
  question and the draw underflowed it. The room is measured again at draw
  time and a wordmark that no longer fits is not drawn.
- The HUD's state moved out of the terminal into `hud::view::View`. The
  terminal module now owns the terminal and nothing else, and the keyboard
  contract is tested with no terminal at all.

## Why the key hints vanish as the window narrows, 2026-08-26

Jeff noticed that two columns of cards collapse to one on a resize and still
read well, while the key hints on the status line sometimes disappear. The
collapse is the rule working. The hints going is deliberate and worth writing
down, because it looks like a bug.

The status line is two pieces pushed to opposite ends: what the party is doing
on the left, the keys on the right. `layout::two_up` draws both when both fit
and drops the **right** one when they do not, then truncates the left. Half of
each is worse than all of one, and of the two the keys are the reminder while
the left half is the data. So the order of what survives is: the panel and its
number, the party state, the arrangement, the watch loop's latest word, and
last of all the keys.

`--help` lists the keys, which is why losing them from a narrow window costs
nothing. Every card in the party surviving a resize is the same `two_up` rule
one level down: the class and level share the name line until they cannot, and
then they get their own.
