# 044 — Formatting is a standard the build checks, not a habit

Type: `wayfinder:task` (AFK)
Status: open
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

- [ ] rustfmt's version is pinned so two machines format the same file the same
      way
- [ ] Something the developer cannot forget rejects or fixes unformatted code
- [ ] Whatever every clone has to do once is written down where a new clone
      will read it
- [ ] The decision about clippy is made at the same time, either way
- [ ] The repo is formatted once, in a commit that does nothing else, so the
      next feature diff is clean
