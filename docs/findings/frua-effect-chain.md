# Unlimited Adventures — where "Diseased" lives

**Date:** 2026-08-25
**Game:** Unlimited Adventures (FRUA) only.
**Status:** confirmed for Unlimited Adventures. Every number below was read out
of two real FRUA save files taken five minutes apart, on either side of a cure.

## Scope

Everything here is Unlimited Adventures. The other eleven games are not
checked, and there is reason to expect they differ: their character records are
already different lengths, `pool-of-radiance` being 285 bytes against FRUA's
398, and each game has its own save layout. Treat the ten-byte entry shape, the
effect codes, and the save offsets as FRUA facts until another game confirms
them. `docs/hackdocs/` documents the FRUA engine specifically, so it is a source
for this game and a hypothesis for the rest.

This was read out of save files on disk, not out of a live DOSBox process. The
chain almost certainly follows the record in memory the same way, because the
save holds the record verbatim, but that is not verified.

## Summary

Disease is not a field in the character record. It is three entries in a
variable-length chain of ten-byte effect records that hangs off the end of the
record. This is the first hard look at the structure the roadmap calls
"conditions and effects", and it is why that item needs its own read rather
than another `[[field]]` in the table.

## What the status byte does not hold

`squire-core/tables/unlimited-adventures.toml` reads a `status` byte at 0x05E.
Its enum is complete and it has no diseased value:

    okay, animated, tempgone, running, unconscious, dying, dead, stoned, gone

`docs/hackdocs/CCHFORM.TXT` offset 94 lists one more, `9 = Awaiting Summons`,
which the table omits. That is the whole set. Nothing there says diseased.

Offset 128 is the "Cure Disease Flag". It is the number of times a paladin can
cure disease, not an affliction. Do not read it as a disease indicator.

## The effect chain

Bytes 4-7 of the character record are a pointer. Zero means the character has
no effects. Non-zero means a chain exists.

The chain sits after the fixed 398 bytes and ends flush with the record. Each
entry is ten bytes:

| Byte | Meaning |
|---|---|
| 0 | effect code |
| 1 | zero |
| 2-3 | duration in rounds, little-endian. 0 is permanent |
| 4-5 | `255, 0` for a constant ability. Otherwise the level of the effect |
| 6-7 | counter, little-endian |
| 8-9 | non-zero if another entry follows, `0, 0` on the last entry |

`CCHFORM.TXT` says the link marker is `254, 104`. In this save it is `51, 90`.
Both are pointer fragments left by the engine, so read bytes 8-9 as
non-zero-means-more, never as a fixed pair.

Record length therefore varies per character. In the cured save the same party
holds a 706-byte record, a 554-byte record, and four 516-byte records. Only
the leading 398 bytes are at fixed offsets.

## The disease trio

`docs/hackdocs/SPECAB.TXT` names the codes. These numbers are the FRUA
engine's. Do not assume another game numbers its effects the same way:

| Code | Name | What it does |
|---|---|---|
| 34 | Diseased | Display only. It is what prints "Diseased" on the magic screen |
| 43 | Weakened | Loses a point of strength on a timer. At strength 3, adds effect 31 |
| 44 | Loss of hit point | Loses HP on a timer. At 10% of maximum, adds effect 31 |
| 31 | Helpless | The end state of both timers |

Mummy rot is a different pair. Effect 172 prints the "Diseased" message and
drains charisma to death. Effect 215 does not print it, but blocks all
healing. A mummy hit applies both.

## The observation

Design `BASILISK.DSN`, party of six. BEORN is a dwarf fighter and the game was
showing him as diseased.

`SAVGAME.CSV`, before the cure. His record runs 1735 to 2319, so 584 bytes:
398 fixed plus a 186-byte tail. The last six ten-byte groups of that tail read
as a clean chain:

    +524  code  47  dur   0  lvl 255,0  ctr  20  next 51,90
    +534  code  26  dur   0  lvl 255,0  ctr  30  next 51,90
    +544  code  97  dur   0  lvl 255,0  ctr 180  next 51,90
    +554  code  34  dur   0  lvl   4,1  ctr 190  next 51,90
    +564  code  43  dur  29  lvl   4,1  ctr  80  next 51,90
    +574  code  44  dur   9  lvl   4,1  ctr   0  next  0,0

The first three are 47 Dwarf AC Bonus, 26 Dwarf THAC0, and 97 Short Guy MR.
They carry `255, 0` and a zero duration, which is the constant-ability shape.
The last three are the disease trio, and they carry a real level and a
counting duration.

Two fields in the fixed record agreed before we touched anything. Beorn's
strength pair at 112-113 read `17 / 16` while every other party member's pair
read equal, so effect 43 had already taken a point. His HP read 15 of 23 while
nobody else was below maximum.

## The cure

LIZABELL is a level 1 paladin. Byte 128 of her record was 1, so she had a
charge, and a paladin's cure disease costs nothing. She cured Beorn and the
game was saved to slot F.

`SAVGAMF.CSV`, after the cure. Beorn's record is 554 bytes, exactly 30 fewer,
which is three ten-byte entries. Codes 34, 43, and 44 are gone. What remains
is his three dwarf abilities, with the chain now terminating on 97:

    +524  code  47  dur   0  lvl 255,0  ctr  20  next 51,90
    +534  code  26  dur   0  lvl 255,0  ctr  30  next 51,90
    +544  code  97  dur   0  lvl 255,0  ctr   0  next  0,0

The cure removes the effects. It does not undo their work. Beorn's strength
pair still reads `17 / 16` and his HP is still 15 of 23.

Lizabell paid for it in the same file. Byte 128 went from 1 to 0, her record
grew by exactly ten bytes, and one entry was appended to her chain:

    +686  code   8  dur     0  lvl 255,0  ctr 80  next 51,90
    +696  code 110  dur 10079  lvl   0,1  ctr  0  next  0,0

Effect 110 is "Paladin Cure used up", and its duration is the delay until the
ability comes back. A round is one minute and an AD&D paladin cures disease
once per week, which is 10080 minutes. The engine writes 10079 and counts
down, so the ability returns a week later to the round.

## What this means for Squire

The TOML table cannot express this. Every `[[field]]` is a fixed offset inside
`record_len`, and the chain starts at a position that varies per character and
repeats an unknown number of times. Reading effects needs a walker, plus a
code-to-name table that is per-game the way the record tables are. Each of the
other eleven games needs the same investigation this one got before its effects
can be read.

The save reader already survives it. `party_file_records` in
`squire-core/src/saves.rs` validates at the current position and slides one
byte on a miss rather than striding by `record_len`, so a growing or shrinking
effect tail never desynchronizes the walk. Confirmed here: the six records
moved when Lizabell grew and Beorn shrank, and Squire still read the party.

Reading effects is additive work, not a rework.

## Reproducing this

In Unlimited Adventures the party file holds whole `.cch` records back to back
from offset 1039, with the party size at 1037. Both offsets are already in
`squire-core/tables/unlimited-adventures.toml`. Other games place these
elsewhere, or do not use a party file at all. To read one character's chain, find the record start,
add `398`, and walk ten bytes at a time until bytes 8-9 are both zero. The
chain ends flush with the record, so walking backward from the next record's
start works too, and is easier when the record length is already known.

FRUA effect codes are documented in `docs/hackdocs/SPECAB.TXT`, one numbered
paragraph per code. `docs/hackdocs/SPELLEFF.TXT` maps spells to the effects
they apply, which is how the disease trio was found: spell 39, Cure Disease,
removes effects 31, 34, 43, 44, 172, and 215.
