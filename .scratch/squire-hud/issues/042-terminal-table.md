# 042 — The terminal table

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: none

## What to build

The data 043 needs: for each terminal Squire knows, how to give it an app-id
and how to ask for a size in character cells. Every terminal spells both
differently, and there is no standard.

Compiled-in defaults for the terminals that do both cleanly, merged under a
user file keyed by terminal name. Shipping the defaults compiled in means a
fresh clone works; merging rather than writing the whole table out on first run
means a user's stale copy never silently overrides an improvement they would
have wanted.

The user file is the point of the ticket. Somebody's favourite terminal in five
years does not exist yet, and adding it must be a file they write, not a pull
request they send and not a build they run. This is the same principle as
another game being a table rather than code, with one difference worth writing
down: a game table is ours, a terminal entry can be theirs.

A terminal Squire does not recognise is not an error. 043 still launches it and
says the size could not be set.

## Acceptance criteria

- [ ] Compiled-in entries for the terminals that support an app-id and a
      cell-based size
- [ ] A user file, merged over the defaults by terminal name
- [ ] Adding an unknown terminal needs no rebuild
- [ ] A malformed user entry names the file and the entry, and does not stop
      Squire from running
- [ ] The difference from the game tables is recorded where a reader will find
      it
