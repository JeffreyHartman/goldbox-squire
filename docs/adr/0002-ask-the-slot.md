# ADR 0002 — Ask for the save slot, never union-scan

Status: accepted
Recorded: 2026-08-23, retroactively. The decision predates this file and was
referenced from code and tickets before it was written down.

## Context

The scanner hunts character names, and names come from one save slot's `CHRDAT`
files. The tool must know which slot the player loaded.

## Decision

A watch session targets one save slot, chosen by the user each run. The wizard
lists the populated slots with their party names, and the user picks one. A slot
describes one sitting, so it is asked every run and never stored in the config.

## Considered options

The rejected alternative is the union scan: read every populated slot's names,
search for all of them, and report which slot's party turned up. It removes the
question from the user and reports a fact instead of a selection, which fits
this codebase's refusal to guess. It was rejected anyway, for three reasons.

- Two slots can hold the same names, because a copied party is normal play. The
  union scan would anchor on one of them with no way to tell which.
- The scan is linear in the name count, and it runs repeatedly while waiting for
  the game to load, so ten populated slots make the wait ten times heavier.
- The party state logic (Live, Partial, Not found) compares found characters
  against the names searched for, so a union of slots forces the whole session
  to become slot-aware just to keep "Partial" honest.

## Consequences

A future reader will re-propose the union scan; this file exists so the
re-proposal starts from these costs. Picking the wrong slot is handled in the
open: after ten seconds without a match, the output names the chosen slot and
offers to choose again.
