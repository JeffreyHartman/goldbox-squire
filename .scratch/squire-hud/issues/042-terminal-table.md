# 042 — The terminal table

Type: `wayfinder:task` (AFK)
Status: resolved
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

- [x] Compiled-in entries for the terminals that support an app-id and a
      cell-based size
- [x] A user file, merged over the defaults by terminal name
- [x] Adding an unknown terminal needs no rebuild
- [x] A malformed user entry names the file and the entry, and does not stop
      Squire from running
- [x] The difference from the game tables is recorded where a reader will find
      it

## Answer

`squire-cli/terminals.toml` is the compiled-in table, `squire-cli/src/terminals.rs`
reads it, and ten tests cover the merge. The user file is
`terminals.toml` in Squire's config folder, beside `config.toml`, merged over
the defaults by terminal name. Squire never writes it.

Three entries: foot, alacritty and kitty. Each one gets three fields, because
two were not enough. `app_id` and `size` are what the ticket asked for, and
`exec` is what goes between the options and the command, since alacritty needs
`-e` there, kitty needs nothing, and 043 cannot spawn anything without knowing
which. Placeholders are `{id}`, `{cols}` and `{rows}`.

**What is verified and what is not.** alacritty and kitty were checked against
the installed binaries: `alacritty --help`, `man 5 alacritty` for
`window.dimensions` in cells, and `/usr/share/doc/kitty/kitty.conf` for the `c`
suffix that makes `initial_window_width` mean cells. foot is not installed here
and its two flags come from its documentation. Somebody should run it before
043 ships.

Deliberately not compiled in: konsole and gnome-terminal, which have no
per-instance window name; wezterm and ghostty, which probably can do both but
which I could not check. All four are still launchable, and all four are one
user file entry away.

The difference from the game tables is in `CONTEXT.md` under **Terminal table**
and in `AGENTS.md` under "Where things are": a record table is Squire's, a
terminal entry can be the user's.

## Review note

Two findings from the spec review, both fixed.

A user file was read in one go, so one misspelled field threw away every other
entry in the file, and `exec` was required even though most terminals need
nothing there. Each block is now read on its own, missing fields default to
empty, and a bad block names itself and its position while the blocks around it
still take effect.

`command_line`, `find` and `load` are 043's seam decided here rather than
there. That is deliberate: the placeholders cannot be tested without something
that substitutes them, and a table nobody can read is not data. 043 is free to
change the shape.
