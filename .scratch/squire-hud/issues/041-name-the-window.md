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

## Read this before starting

Noted 2026-08-26.

**"Squire sets a stable app-id" is not a call Squire makes.** The HUD is drawn
inside a terminal, and the terminal is what reports an app id to the desktop.
So the mechanism is 042's table: the `app_id` arguments of the entry for
whichever terminal is spawned, filled with the `{id}` placeholder. Squire's job
is to own the name and pass it. There is no Wayland call to go looking for.

**Blocked on a decision, not on code.** This wants a README section and the
repo has no README; ticket 012 owns writing one. Jeff said on 2026-08-26 that
the README is handled separately and is not part of the HUD effort. So the
app-id half can be worked without it, and the placement explanation waits for
012.

**Read 043's own note before either.** The two are one piece of work, and 043's
stated process topology does not survive contact with ticket 011.
