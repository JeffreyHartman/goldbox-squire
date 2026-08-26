# 039 — Squire remembers the window size

Type: `wayfinder:task` (AFK)
Status: open
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

- [ ] The size at exit is written to the config, under a global key
- [ ] The key is human-readable and its name says what it is
- [ ] `--help` or the config file itself makes clear where the size is
      remembered, so it is not a hidden behaviour
- [ ] A missing or nonsensical stored size is ignored, not fatal
- [ ] Existing config files load unchanged, and the migration path already in
      the config module keeps working
