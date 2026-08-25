# 020 — Repick the slot mid-watch

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 018, 019

## What to build

The user picks slot B, then loads slot A in the game. The tool hunts B's names
forever and tells a player who has loaded a save to load a save. Handle the
mistake in the open instead.

- After ten seconds with no party found, extend the waiting message to name
  the assumption: which slot is being looked for and its party names, and that
  pressing Enter chooses a different slot.
- Enter during the watch returns to the slot question, then resumes watching
  with the new slot's names. This holds while waiting and after a party was
  found, so a fat-fingered pick is recoverable at any point.
- The watch loop currently sleeps between reads. Replace the sleep with a wait
  on stdin readiness with the interval as timeout, so Enter is noticed at
  once and no thread is added. `nix` is already a dependency and has the
  needed poll call.

## Acceptance criteria

- [x] Ten seconds without a match adds the slot's letter, its names, and the
      Enter hint to the output.
- [x] Enter mid-watch re-asks the slot question and the watch continues with
      the new choice.
- [x] No new dependency and no raw terminal mode.
- [x] Polling cadence is unchanged when the user types nothing.

## Answer

The watch's sleep became `nix::poll` on stdin with the poll cadence as the
timeout, so Enter is noticed at once, the cadence is unchanged when nothing
is typed, and no thread was added. Ten seconds without a match prints the
slot letter, its party names and the Enter hint. Enter re-runs the slot
question via `wizard::repick_slot` (b backs out and keeps the current slot),
and `Session::retarget` swaps the names and drops the old anchors so the next
read scans. Repicking works while waiting and after a party was found. Stdin
at end of file turns the listening off, and the `--pid` path never listens.
