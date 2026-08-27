# 041 — Squire names its window

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: none

## What to build

The HUD is meant to sit beside the game, and Squire cannot put it there.
Wayland does not let a client position its own window. What Squire can do is
give its window a stable name, so the user's compositor can recognise it and a
window rule written once pins it on every launch afterwards.

This ticket is the name. The explanation of why placement is the compositor's
job is a README section, and it moved to
[045](045-readme-says-how-to-place-the-window.md).

**The app-id is not a call Squire makes.** The HUD is drawn inside a terminal,
and the terminal is what reports an app id to the desktop. So the mechanism is
042's table: the `app_id` arguments of the entry for whichever terminal is
spawned, filled with the `{id}` placeholder. Squire's job is to own the name
and pass it. There is no Wayland call to go looking for.

## Acceptance criteria

- [x] Squire owns one stable app-id string and passes it through 042's
      `app_id` arguments for the spawned terminal
- [x] A terminal entry with no `app_id` arguments is still spawned
- [x] The name is asserted in a test, because a compositor rule breaks
      silently when it changes

## Read this before starting

Read 043's own note first. The two are one piece of work, and 043's stated
process topology does not survive contact with ticket 011.

## Answer

The name is `goldbox-squire`, held as `terminals::APP_ID`.

`Terminal::command_line` no longer takes an id. It fills `{id}` from the
constant, because a caller free to pass any name is a caller free to break a
compositor rule the user already wrote, and there was never more than one name
to pass. The tests that used to pass `"gbs-hud"` by hand are exactly the drift
this closes.

`terminals.toml` now says what `{id}` is filled with, and `CONTEXT.md` records
the value beside the App id definition.

Three tests: the name is pinned, every compiled-in terminal's command line
carries it, and a user entry with no `app_id` arguments still produces a
command line that spawns.

The README half is [045](045-readme-says-how-to-place-the-window.md).

## Later note, 2026-08-26

This ticket's reasoning for a single constant was that "there was never more
than one name to pass". [ADR 0005](../../../docs/adr/0005-one-host-reads-many-views-draw.md)
ends that: the map and the journal get windows of their own, and the user
writes one compositor rule per window, so one shared name would place the map
wherever it places the HUD. `046` puts the parameter back as a view kind from
a fixed list, which still leaves no caller free to invent a name. The answer
above is otherwise unchanged.
