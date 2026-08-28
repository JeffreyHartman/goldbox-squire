# Goldbox Squire

A native Linux tool that reads live party state out of a running DOSBox process
for the twelve SSI AD&D Gold Box games. It replaces the memory-reading part of
Gold Box Companion, which is Windows-only and works with vanilla DOSBox 0.74-3
only.

`CLAUDE.md` is one line that imports this file. Claude Code, OpenCode, and
anything else that reads `AGENTS.md` all get the same instructions.

## Agent skills

### Issue tracker

Local markdown under `.scratch/<feature>/`, gitignored. Not GitHub Issues, by
choice, for now. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical roles, unchanged, written as a `Triage:` line in the issue
file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the repo root. See
`docs/agents/domain.md`.

## Setting up a clone

Run this once, in every clone:

```sh
git config core.hooksPath .githooks
```

That turns on `.githooks/pre-commit`, which rejects a commit whose code is not
formatted. It never edits your files. Run `cargo fmt --all` yourself and stage
the result. It reads the working tree rather than the index, so an unformatted
edit you left unstaged still blocks the commit.

`rust-toolchain.toml` pins the compiler, `rustfmt`, and `clippy`, so two
machines format the same file the same way. Cargo installs that toolchain on
its own the first time you build.

`cargo clippy` is deliberately not in the hook. It has to compile the crate,
and a slow hook is a hook people bypass.

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
- `squire-cli/terminals.toml` — the terminals the HUD can open a window in.
  Also a table, and the only one a user can extend: their own
  `terminals.toml` in Squire's config folder is merged over it by name.
