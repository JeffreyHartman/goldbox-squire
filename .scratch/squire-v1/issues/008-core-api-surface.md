# 008 — The core crate's public interface

Type: `wayfinder:grilling` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: 007

## Question

Design what `core` exposes. It must know nothing about a terminal, a window, or
a compositor.

- The vocabulary: what a Session, a Party, a Character, and an Anchor are, and
  which of them are types.
- What the caller does to get party state, and what it gets back.
- How errors are expressed, and which are recoverable.
- Whether the core polls, or whether the caller drives the loop.
- What is deliberately not exposed.

Consult the `codebase-design` skill for this ticket.

## Answer

**`Session<R: Reader>`, `Party`, `Character`, and a private `Anchor`.**

`squire-core/src/session.rs`.

- `Session::new(reader, table, names)` starts a view. Nothing is read yet.
- `Session::party()` returns a `Party`, which holds a `PartyState` and the
  characters found.
- `Character` is a plain struct of named fields. It exposes no offsets, and a
  front end never learns one.
- `Anchor` is private. Addresses do not leave the crate.
- Errors are one `Error` enum in `lib.rs`. `PermissionDenied` carries the
  remedy in its message.
- The caller drives the loop. The core never polls and never sleeps.

`Reader` is a trait, so a test supplies memory without a live emulator. That is
the seam the session tests use.
