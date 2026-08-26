# 039 — Squire remembers the window size

Type: `wayfinder:task` (AFK)
Status: resolved
Triage: `ready-for-agent`
Blocked by: 037

## What to build

Resize the terminal to where you want the HUD, quit, run `gbs` again, and it
comes back the size you left it. Nothing is asked and no flag is passed.

The wizard asks about things Squire cannot observe: which game, which
directory, which slot. Window size is not one of those. Asking "how many
columns would you like" is a worse question than noticing the answer.

Global, not per game. Window size is a property of where the user sits, not of
which game they loaded, and a per-game key would mean fixing twelve entries
after changing a monitor.

This ticket only records and reports the size. Acting on it is 043's job, since
a program cannot resize the terminal it was launched inside.

## Acceptance criteria

- [x] The size at exit is written to the config, under a global key
- [x] The key is human-readable and its name says what it is
- [x] `--help` or the config file itself makes clear where the size is
      remembered, so it is not a hidden behaviour
- [x] A missing or nonsensical stored size is ignored, not fatal
- [x] Existing config files load unchanged, and the migration path already in
      the config module keeps working

## Answer

`[hud]` in the config file, with `columns` and `rows`. Global, one entry, and
the key says what it is.

The size is read from the terminal at every draw and written on the way out of
`run_watch`, whether the run ended well or badly. The user resized the window
either way, and losing that to an error would mean resizing it again next
launch.

A stored zero is ignored: that is what a terminal reports when it does not
know its own size, and it reaches the file when a run ends before the first
draw. A hand-edited nonsense value is ignored too, and the rest of the config
still loads, because the field is parsed on its own rather than as part of the
whole file. `--help` says where the size lives.

Nothing acts on the remembered size yet. A program cannot resize the terminal
it was launched inside, which is 043's job.

## Review, answered

The warning about a config that could not be saved was printed while the
alternate screen was still up, which is a warning nobody can read. The size is
now read before the terminal goes back and written after.
