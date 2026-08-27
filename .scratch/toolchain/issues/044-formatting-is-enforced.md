# 044 — Formatting is a standard the build checks, not a habit

Type: `wayfinder:task` (AFK)
Status: done
Triage: `ready-for-agent`
Blocked by: none

## Question

`cargo fmt` is run when somebody remembers. That is not a standard, and the
cost showed up in commit `c98eac9`: a feature commit for the HUD's layout plan
also reflowed seven files nobody touched, because the last person to format
them used an older rustfmt. Whitespace churn in a feature diff is noise a
reviewer has to read past, and it makes `git log -p` and `git blame` worse for
exactly the files nobody changed.

Two separate problems live under this, and fixing one without the other leaves
the churn in place.

**The version drifts.** There is no `rust-toolchain.toml`, so rustfmt is
whichever one the machine has. Two developers, or a developer and an agent on a
different box, format the same file two ways and each reformats the other's
work. A toolchain file pins the compiler and the components, and that is what
stops the churn rather than merely making it loud.

**Nothing checks.** `cargo fmt --check` is never run, so an unformatted commit
lands silently and the next `cargo fmt` sweeps it up into whatever commit is in
progress. Where the check belongs is the open question.

- A **git pre-commit hook** catches it before the commit exists, which is the
  earliest and least annoying place. Hooks are not committed, so this needs a
  hooks directory in the repo and `core.hooksPath` pointed at it, which every
  clone has to set once. A hook that formats rather than complains is friendlier
  and also rewrites files under the developer, which is its own surprise.
- **CI** catches everything and needs no per-clone setup, but the repo has no
  `.github/workflows` at all, so this is a first CI job rather than one more
  step in an existing one. It also catches the problem after the commit, which
  means a fixup commit.
- **Both**, with the hook as the fast path and CI as the thing that cannot be
  skipped.

`cargo clippy` has the same shape of problem and is worth deciding at the same
time, since whatever runs one can run the other.

## What decides it

Jeff's own working style, which the agent should ask about rather than guess:
whether a hook rewriting files during a commit is welcome or infuriating, and
whether the project wants CI at all yet.

## Acceptance criteria

- [x] rustfmt's version is pinned so two machines format the same file the same
      way
- [x] Something the developer cannot forget rejects or fixes unformatted code
- [x] Whatever every clone has to do once is written down where a new clone
      will read it
- [x] The decision about clippy is made at the same time, either way
- [x] The repo is formatted once, in a commit that does nothing else, so the
      next feature diff is clean — nothing to format under the pinned rustfmt

## Answer

Check-only hook, no CI yet.

- `rust-toolchain.toml` pins 1.98.0 with `rustfmt` and `clippy`.
- `.githooks/pre-commit` runs `cargo fmt --all --check` and stops the commit.
  It does not rewrite files, because a hook that formats during a commit can
  sweep in hunks the developer deliberately left unstaged.
- `git config core.hooksPath .githooks` is the one-time per-clone step, written
  into `AGENTS.md` under "Setting up a clone".
- Clippy stays out of the hook. It compiles the crate, and a slow hook gets
  bypassed with `--no-verify`.
- No CI. One developer on one box, and the hook covers the failure in
  `c98eac9`. CI is the thing to add when a second contributor appears.

No repo-wide format commit was needed. Under the pinned rustfmt,
`cargo fmt --all --check` was already clean across both workspace members.

Two known gaps, recorded rather than hidden, and one closed.

- A clone that skips `core.hooksPath` has no check at all. That is the price of
  not running CI yet.
- `cargo fmt` reads the working tree, not the index, so an unstaged unformatted
  edit still blocks the commit. Accepted: it fails loud and in the safe
  direction.
- Anything outside the workspace members escapes `cargo fmt --all` and so
  escapes the hook. The one file in that position, `hello.rs`, was a first
  build from the start of the project and is now deleted. Nothing at the root
  is tracked Rust any more, so the gap is closed rather than merely known.
