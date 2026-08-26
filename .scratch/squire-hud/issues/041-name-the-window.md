# 041 — Squire names its window, and the README says how to place it

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: none

## What to build

The HUD is meant to sit beside the game. Squire cannot put it there: Wayland
does not let a client position its own window, and there is no flag, protocol
or workaround for that. It is the same restriction that makes GBC's
pin-above-DOSBox trick impossible to port, and it is not going away.

What Squire can do is give its window a stable name, so the user's compositor
can recognise it. One KWin window rule or one Hyprland rule, written once,
pins it to the right edge at a chosen size on every launch afterwards.

So: Squire sets an app-id, and the README explains the restriction in two
sentences and gives a worked rule for KDE and for Hyprland. The point of the
explanation is that a user who expected Squire to place its own window
understands why it does not, rather than filing it as a bug.

## Acceptance criteria

- [ ] Squire sets a stable app-id, and the README states what it is
- [ ] The README explains in two sentences why placement is the compositor's
      job and not Squire's
- [ ] A worked KWin window rule and a worked Hyprland rule are given
- [ ] The README does not promise placement, automatic or otherwise
