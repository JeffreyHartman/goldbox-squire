# 006 — What makes a candidate a real character record

Type: `wayfinder:grilling` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: 005

## Question

A name match alone is not proof. The same bytes can appear in a file buffer, in
a second copy of the save, or by chance.

Decide the invariants that promote a candidate to a confirmed record:

- Which fields are range-checked, and what the legal ranges are.
- Whether the six records must sit in a plausible arrangement. Note that the
  gaps are uneven, because each character's inventory follows their record.
  Measured gaps: 528, 432, 496, 480, 352 bytes. These were identical across two
  sessions and two emulator builds.
- What happens when more than one candidate set passes.
- What happens when none pass.

## Answer

**Range checks on six kinds of field. No arrangement check.**

`record::validate` in `squire-core/src/record.rs`. A candidate is promoted
only when all of these hold:

- The name length byte is 1 to 15, and every name byte is printable ASCII.
- Every ability score is 3 to 25.
- Race, class, gender, alignment and status hold a value the game writes.
- At least one class level is non-zero, and none exceeds 40.
- Maximum hit points is non-zero, and current hit points does not exceed it.
  Current may be negative, because a dying character is.

Current hit points is read as a signed byte, which is what makes a dying
character read as -3 rather than 253.

The arrangement check was not implemented. The measured gaps are stable but
they are one party on one machine, and a wrong arrangement rule would reject a
real party. Where a name appears twice, the lowest address wins. Tested in
`squire-core/tests/record.rs` and `tests/scan.rs`.
