# 005 — The character record table format

Type: `wayfinder:grilling` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: none

## Question

Offsets live in a data file compiled into the binary, one file per game. Decide
what that file looks like.

- Which format: TOML, RON, or something else.
- What a field entry holds: offset, width, type, name, and what else.
- How fields that are not plain integers are described: the name is a length
  byte plus 15 bytes; spells are bit arrays; class and race are enumerations.
- Whether the table describes only the fields v1 reads, or the whole 285-byte
  record from the start.
- How a table is validated at build time, so a typo fails the build rather than
  producing wrong numbers at run time.

Source material: `~/goldbox/gbc/Resources/Character file formats/01. Pool of
Radiance.txt` and `CCHFORM.TXT` in `~/Downloads/hackdocs.zip`.

## Answer

**TOML, in `squire-core/tables/pool-of-radiance.toml`.**

A field entry holds `name`, `offset`, `len`, and `kind`. Kinds are `u8`,
`u16le`, `u32le`, `pascal_string` and `enum`. An enum field names the
enumeration it reads from, and the enumerations are defined in the same file.

The table describes the fields the tool reads or validates, not the whole
record. Adding a field is a line in the table, not a line of code.

Validation runs when the table loads, in `Table::validate` in
`squire-core/src/table.rs`. It rejects a field that runs past the record, a
duplicate name, an overlap, a zero length, a width that contradicts its kind,
and an enum field naming an enumeration that does not exist. Offsets use
`checked_add`, because a table is untrusted input.

Build-time validation was not implemented. The built-in table is checked by the
test suite instead, in `squire-core/tests/table.rs`. A typo therefore fails the
tests rather than the build. Reviewed and accepted as a smaller thing that
catches the same mistake.
