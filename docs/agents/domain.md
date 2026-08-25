# Domain docs

How the engineering skills should consume this repo's domain documentation when
exploring the codebase.

This repo is **single-context**. One glossary at the root covers both crates:
`squire-core` and `squire-cli` are a layering boundary, not two domains. There
is no `CONTEXT-MAP.md` and there should not be one.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root. The glossary, and nothing else: no offsets,
  no module names, no decisions.
- **`docs/adr/`**. Read the ADRs that touch the area you are about to work in.
  Four exist, and two of the four are partly superseded, so read the
  `Status:` line before you trust the body.

## File structure

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/
│   │   ├── 0001-use-the-publishers-files.md
│   │   ├── 0002-ask-the-slot.md
│   │   ├── 0003-gbs-owns-the-conf.md
│   │   └── 0004-a-game-picks-a-directory.md
│   ├── roadmap.md
│   ├── findings/          ← our reverse-engineering write-ups
│   ├── gbc/               ← third-party reference, gitignored
│   └── hackdocs/          ← third-party reference, gitignored
├── squire-core/
└── squire-cli/
```

## ADR conventions in this repo

- Filename `NNNN-<short-slug>.md`, four digits, numbered in order recorded.
- Title line `# ADR NNNN — <the decision, as a sentence>`.
- A `Status:` line directly under the title: `accepted`, or
  `partly superseded by ADR NNNN`, or `superseded by ADR NNNN`.
- A `Supersedes:` line when the ADR overturns an earlier one, naming what it
  overturns rather than the whole ADR.
- Superseded text is not deleted. It is marked, so a reader can see what was
  believed and why it stopped being true.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal,
a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Do not
drift to synonyms the glossary explicitly avoids. "Slot" is the save letter A
through J, never the character index. "Install" is a game directory. "Game" is
one of the twelve titles.

If the concept you need is not in the glossary yet, that is a signal: either you
are inventing language the project does not use (reconsider) or there is a real
gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than
silently overriding:

> _Contradicts ADR 0002 (ask the slot), but worth reopening because…_

ADRs 0003 and 0004 exist because field testing broke ADR 0001 in one evening.
Overturning one is normal. Overturning one silently is not.

## Reference material

`docs/gbc/` and `docs/hackdocs/` are third-party and gitignored, so they are
present on Jeff's machines and absent from a fresh clone. `docs/README.md`
records how to rebuild them. Do not cite a path under either directory as
though a reader will have it; cite the fact and say where it came from.
