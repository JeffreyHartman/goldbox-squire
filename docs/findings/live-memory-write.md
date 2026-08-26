# Writing to a live DOSBox is a file write

**Date:** 2026-08-25
**Status:** confirmed once, end to end, on the setup below. One byte, one game.

The roadmap's whole "Writing to memory" section was planned as if writing were
a harder capability than reading. It is not. On Linux, with the permission
Squire already needs in order to read, a write is `open` and `seek` and
`write` on `/proc/<pid>/mem`. No `ptrace` attach, no stopping the emulator, no
`process_vm_writev`.

## The setup

| Thing | Value |
|---|---|
| Game | Unlimited Adventures, design `BASILISK.DSN` |
| Emulator | `dosbox-staging` 0.83.0-RC1 (15a8e), `machine = svga_s3` |
| Host | CachyOS, kernel 7.2.0 |
| `kernel.yama.ptrace_scope` | 0 |
| Emulator pid | 117102, started by Squire |

## What was written

Beorn, a dwarf fighter, had permanently lost a point of strength to the disease
effect described in [frua-effect-chain.md](frua-effect-chain.md). The record
keeps the original score at byte 112 and the current one at byte 113, so the
target was one byte holding a wrong value with the right value beside it.

Finding it was a plain substring search. Reading `/proc/117102/maps`, taking
every readable region under 512 MiB, and searching each for `BEORN\0` returned
**exactly one hit** across the whole process. The record began 0x60 bytes
earlier, and its race, class, gender, alignment, and status bytes all validated.
Its field values matched the save on disk exactly. One copy, no ambiguity, and
nothing to keep in sync.

    record  0x7f753c661d3e   anon, rw-p
    target  0x7f753c661db3   record + 113, current strength
    write   16 -> 17

The whole write:

```python
with open(f'/proc/{pid}/mem', 'r+b', 0) as f:
    f.seek(addr)
    f.write(bytes([17]))
```

## What happened

The byte held. It was re-read at 0, 0.5, 2, and 5 seconds and stayed 17 every
time, so UA does not recompute the current score from anywhere else. The value
in the record is the value the game keeps.

The game accepted it. Beorn's character sheet showed strength 17 when reopened.

The game persisted it. Saving to a fresh slot wrote 17 to byte 113 on disk, so
the change survived the round trip through the engine's own save path rather
than sitting in memory as a display-only lie.

## What this changes

Reading and writing need the same permission. Squire's existing posture already
covers writing, so the ptrace-scope conversation and the `--pid` escape hatch in
ticket 011 do not need revisiting for it.

The write path does not need to be proven separately. The roadmap called the
Fix command "the smallest useful write, so it is the one that proves the write
path". That job is done. Fix can be built as a feature rather than as a
proof.

Finding a field by searching for a character's name works, and is what Squire's
anchor scan already does. Locating a writable field is therefore the same
problem as locating a readable one.

## Limits on this result

One byte, one game, one emulator build, one host. Nothing here says that a
multi-byte write, a write during combat, or a write to a field the engine
recomputes each frame behaves as well. The roadmap's note that these features
need an "is combat running" check still stands, and is now the harder half of
the work rather than the easier one.

`ptrace_scope` was 0. A hardened default of 1 still permits a process to be
read and written by its own parent, which is the normal Squire case because
Squire launches the emulator. The `--pid` path is the one that suffers, exactly
as it already does for reads.

The write went to DOSBox's emulated RAM, found by content rather than by a
known address. Nothing was learned about where that RAM sits, and nothing here
depends on knowing.
