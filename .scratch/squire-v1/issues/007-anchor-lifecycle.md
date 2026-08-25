# 007 — The anchor lifecycle and staleness policy

Type: `wayfinder:grilling` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: 006

## Question

The anchor is found once, then read from continuously. It can go stale. A level
up, a party change, a new save loaded, or the emulator restarting all invalidate
it.

Decide:

- How staleness is detected, and how cheaply. Re-checking the name each poll is
  one option.
- What the tool does when the anchor is lost: re-scan silently, report, or stop.
- What the user sees during a re-scan. Showing stale numbers as if they were
  live is the failure mode to avoid.
- Whether the anchor is cached between runs, and if so, how it is invalidated.

## Answer

**Re-check the name before every read. Rescan when it fails.**

`Session::anchors_still_valid` in `squire-core/src/session.rs`.

Staleness is detected by re-reading the 16-byte name field at each anchor before
every read. That is 96 bytes for a full party, so polling several times a second
costs almost nothing. A level up, a wound, an aged character or a magic item all
leave the name untouched, which is why the name is the anchor and the wider
record is not.

When any anchor fails the check, the session rescans silently and reports the
result. It never reports old numbers as if they were live: an empty result is
`PartyState::NotFound`, a short one is `PartyState::Partial`, and the front end
says so in words.

The anchor is not cached between runs. A cached address would need the same
invalidation work at startup, and the scan takes well under a second.
