# 040 — Keys: quit, character selection, and the slot repick

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: 037

## What to build

The keyboard contract for the HUD: quit, and move the highlight through the
party.

The part with the trap in it is the slot repick. Today, pressing Enter during
the watch returns to the slot question and the watch resumes against the new
slot's names. That path polls stdin for readiness and then reads a line. A HUD
puts the terminal in raw mode, where reading a line does not work and the poll
is no longer the right ear. The repick has to keep working, which means it
either suspends the HUD to ask its question or asks it on screen.

This is its own ticket rather than a footnote on 037 because it is the one
place where the existing behaviour quietly breaks, and it would be easy to ship
a HUD that lost a working feature without anyone noticing.

The keys follow the Gold Box idiom the spike found: number keys jump straight
to a destination, which costs nothing to reserve now and means the second panel
does not need a menu built for it.

## Acceptance criteria

- [x] A documented key quits, and the terminal is restored
- [x] The highlight moves through the party and does not run off either end
- [x] The slot repick still works, and the wizard's question is readable while
      the terminal is in raw mode
- [x] Repicking retargets the session and the HUD shows the new party
- [x] Number keys are reserved for panels, even though only one panel exists
- [x] The keys in use are visible somewhere without reading the source

## Answer

In `squire-cli/src/hud/mod.rs`, `HudKeys`.

`q`, Escape and Ctrl-C quit, through a new `Interrupt::Quit` that the watch
loop turns into a clean return. Ending the run is the loop's job because the
loop is what holds the emulator handle. Quitting the HUD does not stop the
game: ticket 011 settled that a tool which reads the game never takes the game
down with it.

Up and down, or `k` and `j`, move the highlight. It stops at each end rather
than wrapping, because a HUD glanced at sideways should not move the highlight
somewhere surprising.

`a` toggles the ability scores. `c` steps through the arrangements the party
divides into evenly and then back to letting the rule decide, which is how a
short wide window gets both of 034's answers. The number keys are reserved for
panels; only `1` has a screen behind it. The keys are printed on the status
line and in `--help`.

**The slot repick.** The HUD steps aside for the length of the question. It
leaves raw mode and the alternate screen, the wizard prints its menu and reads
a line exactly as it always has, and the HUD takes the terminal back
afterwards, whether the wizard succeeded or failed. That is far less code than
a second copy of the menu drawn on screen, and it keeps one wizard rather than
two that can disagree about what a save slot is.

## Review, answered

The keys were only half documented: the status line omitted the arrows and
`--help` omitted Escape and Ctrl-C. Both list the whole set now, and a test
checks `--help` against it.

The HUD half of the repick had no test, because it needed a terminal. It does
not any more: `hud::view::View` holds everything a key changes and
`tests/hud_keys.rs` drives it directly. Sixteen tests cover quitting, both ends
of the highlight, a party that shrank underneath it, both toggles, the reserved
number keys, and a retarget clearing the old slot's party.

## Changed after Jeff used it, 2026-08-26

Three of the keys were wrong in practice, and one acceptance criterion above is
now void. It stays ticked and struck through by this section rather than
rewritten, because the reasoning matters.

**The highlight is gone, and so are the arrow keys.** "The highlight moves
through the party and does not run off either end" is no longer a requirement.
A highlight that always sits on somebody makes that character look like the
party leader, which in a Gold Box game means something and here means nothing.
There is nothing yet to select a character *for*, so the selector was a mode
with no action behind it, which is the same mistake as the spike's left menu:
screen furniture advertising a feature that does not exist. Deleted rather than
made optional. When a character-level action lands, the highlight comes back
with it, and re-adding it is a field on `View` and a background colour.

**The slot repick moved off Enter and onto `s`.** Jeff pressed Enter to find
out what it did, because the highlight had suggested he was picking a character
for something, and it threw him back to the wizard. Enter is the key people
press to discover what a key does, and going back to the wizard is not a thing
to discover by accident. Enter now means nothing at all.

**The status line names the arrangement.** `2 across, auto` or `6 across,
chosen`. Cycling with `c` and having to count the cards is a key you press until
something looks right, and there is no way to tell that `c` has come back round
to letting the rule decide. `auto` says a resize may change it; `chosen` says a
key asked for it and `c` will move off it.

There are still no named layouts. The honest name for an arrangement is how
many cards are across and who chose it, which is what this prints.
