# Goldbox Squire

A native Linux tool that reads live party state out of a running DOSBox process
for the twelve SSI AD&D Gold Box games. It replaces the memory-reading part of
Gold Box Companion, which is Windows-only and works with vanilla DOSBox 0.74-3
only.

`CLAUDE.md` is one line that imports this file. Claude Code, OpenCode, and
anything else that reads `AGENTS.md` all get the same instructions.

## Agent skills

### Issue tracker

Local markdown under `.scratch/<feature>/`, committed. Not GitHub Issues, by
choice, for now. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical roles, unchanged, written as a `Triage:` line in the issue
file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the repo root. See
`docs/agents/domain.md`.

## How Jeff works

The agent writes the code. Jeff reads it to understand how it works, and asks
questions about it. Code must therefore be plain and commented, and a comment
must say why rather than what. Tests are written first, at seams agreed before
the work starts.

Ask before editing. Diagnose and offer; do not fix unbidden. When Jeff asks why
something is the way it is, answer the question rather than changing the code.

Simple beats clever. The wizard asks; arguments only skip questions the wizard
would have asked. No hidden traps.

## Where things are

- `docs/roadmap.md` — every feature the program might grow, in order. A standing
  document, not a ticket.
- `.scratch/squire-v1/map.md` — the v1 wayfinder map and its 32 tickets.
- `docs/findings/` — the reverse-engineering write-ups.
- `docs/gbc/` and `docs/hackdocs/` — third-party reference, gitignored. See
  `docs/README.md` to rebuild.
- `squire-core/tables/` — one TOML character-record table per game. Another game
  is a table, not code.
