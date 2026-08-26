# Wayfinder map: Goldbox Squire v1

Label: `wayfinder:map`

## Destination

**Changed on 2026-08-22.** A working v1 of Goldbox Squire, plus a prioritized
feature roadmap for what comes after it.

The original destination was a written specification detailed enough that Jeff
built v1 himself in Rust. He withdrew that: he does not want to learn a language
to get the tool, and prefers to read working code. The decision tickets that the
specification would have answered are answered by the code instead, and each one
records where in the code its answer lives.

v1 is built. `gbs` starts DOSBox, finds the party in its memory by character
name, and prints it as a table or as JSON. 96 tests.

## Notes

**Domain.** A native Linux tool that reads live party state out of a running
DOSBox process for the SSI AD&D Gold Box games. It replaces the memory-reading
part of Gold Box Companion (GBC), which is Windows-only and works with vanilla
DOSBox 0.74-3 only.

**Wine is not a target.** GBS exists so that Wine is unnecessary. Jeff plays with
native Linux dosbox-staging, which has good sound and working shaders. The Wine
setup at `~/goldbox/wine` is the GBC workaround, and it is reference only. No
GBS feature exists to support it.

**How Jeff works.** Changed on 2026-08-22. The agent writes the code. Jeff reads
it to understand how it works, and asks questions about it. Code must therefore
be plain and commented, and a comment must say why rather than what. Tests are
written first, at seams agreed before the work starts.

**Skills each session must consult.** `grilling` and `domain-modeling` for
decision tickets. `codebase-design` when designing the core interface.
`research` for AFK research tickets. `prototype` when the question is how
something should look or behave.

**Reference material.** All of it now lives under `docs/` in the repo. The
third-party folders (`docs/gbc/`, `docs/hackdocs/`) are gitignored, so they are
on Jeff's machines and absent from a fresh clone. `docs/README.md` says how to
rebuild them.

- `docs/findings/FINDINGS.md` — the reverse-engineering write-up. Confirmed
  offsets, the memory mechanism, the deterministic layout. Committed.
- `docs/gbc/Resources/Character file formats/` — GBC's offset tables for all
  twelve games. The source for GBS's data tables.
- `docs/hackdocs/` — the UA Hacker's Guide collection, 57 documents:
  `CCHFORM.TXT`, `SAVGAM.TXT`, `ITEM.TXT`, `SPELLEFF.TXT`, `OPCODES.TXT`,
  `PCSPKR.TXT`.
- Memory: `~/.claude/projects/-home-jeff-goldbox/memory/gbc-wine-setup-works.md`
  — records the GBC-under-Wine setup. Background on why GBS exists, not a target.

**The roadmap left this map.** Every feature GBS might grow is enumerated in
`docs/roadmap.md`, a standing document. Ticket
[004](issues/004-feature-roadmap.md) keeps only the part that is still a
decision: which three features come first.

## Decisions so far

### Settled while charting

- **Name**: Goldbox Squire, GBS. `goldbox-daemon` was rejected because v1 is not
  a background service, and `gbd` collides with `gdb`.
- **Language**: Rust. Chosen to learn, over C# which Jeff already knows.
- **Layout**: a Cargo workspace with a `core` crate and a `cli` crate from the
  first commit. The crate boundary enforces that the core knows nothing about
  any user interface.
- **v1 scope**: read-only. No writes to the game's memory.
- **Game scope**: Pool of Radiance only. Other games arrive as data tables, not
  as new code.
- **Emulator scope**: any DOSBox build. The tool finds a process and scans it.
- **The anchor**: search for the character's name, taken from the save files.
  The name never changes during play. This is why GBS works where GBC fails:
  GBC assumes a fixed offset inside DOS memory, so a different emulator breaks
  it. GBS makes no such assumption.
- **GBS launches DOSBox.** GBS starts the emulator as its own child process.
  Yama's descendant rule then permits the memory read with no privilege change
  and no machine-wide setting. Relying on the user to lower
  `kernel.yama.ptrace_scope` is rejected: it ships a security downgrade as an
  install step, and it resets on every reboot. GBS is meant for other people to
  use, so it must work out of the box without weakening their machine.
- **Process discovery**: the child process is known by construction. Whether
  `--pid` survives as an attach escape hatch is open, see
  [011](issues/011-permission-model.md).
- **Anchor source**: read `CHRDATA*.SAV` from the game folder.
- **Offsets**: a data file per game, compiled into the binary at build time.
  GBC hardcodes its offsets. GBS must not.
- **Game folder**: `--game-dir` sets it. A config file stores it. Re-run the
  argument to change it.
- **Output**: a human-readable table, plus `--json`. The JSON is the seam that
  later lets a TUI or a GUI exist without touching the core.
- **v1 fields**: name, class, level, current HP, max HP.
- **Licence**: MIT plus Apache-2.0 dual.
- **Repository**: `~/git/goldbox-squire`, public on GitHub.
- **Editor**: VS Code, rust-analyzer, GitHub Copilot Free (2000 inline
  completions a month).

### Resolved tickets

<!-- one line per closed ticket, then the link -->

- **All twelve games have tables.** Closed 2026-08-24. Eleven tables were
  authored from GBC's character-format docs and its game-by-game comparison
  table; the start commands came from GBC's own per-game configuration. Nine
  are verified against real saves. `matrix-cubed`,
  `countdown-to-doomsday`, and `gateway-to-the-savage-frontier` still say
  UNVERIFIED in their headers, because their game folder names are guesses
  until a real install confirms them.
- **Testing strategy.** Answered by the code. The seam is the core crate's
  `Session`, and the tests run against recorded record bytes rather than a live
  emulator. 96 tests at v1.
- **Config file format and location.** Answered by the code and by ADR 0004: a
  TOML file in gbs's own config folder, holding a chosen directory per game,
  `last_game`, and a global `dosbox`. v1 and v2 files migrate on load.

- [Research: what every GBC tool does](issues/003-research-gbc-full-tool-inventory.md):
  4 of 13 GBC tools read process memory, the rest edit files. `GBC_Audio.exe` is
  a host-side music player over a hand-downloaded pack, not a change to the game
  audio. Paladin and ranger already exist in the Pool of Radiance engine, so
  enabling them is a small set of byte writes, not new game logic. Detail in
  [`research/003-gbc-tool-inventory.md`](research/003-gbc-tool-inventory.md).
- [Research: reading another process's memory on Linux from Rust](issues/002-research-linux-process-memory.md):
  use `process_vm_readv` with batched remote ranges, 3x faster than `pread` and
  correct about inaccessible pages. Take only `nix` and `libc`. Identify the
  target process by the executable path inside `/proc/<pid>/maps`. Guest RAM sits
  at a non-zero offset inside a heap region, so a scanner must never assume the
  region start is the base. Detail in
  [`research/002-linux-process-memory.md`](research/002-linux-process-memory.md).
- [The character record table format](issues/005-record-table-format.md): TOML,
  checked when it loads. Another game is a table, not code.
- [The record validation invariants](issues/006-record-validation-invariants.md):
  six range checks promote a name match to a confirmed record. No arrangement
  check, because one party on one machine is not enough to rule from.
- [The anchor lifecycle](issues/007-anchor-lifecycle.md): re-read the 16-byte
  name before every read, 96 bytes for a party. Rescan when it fails. Never show
  stale numbers as live.
- [The core API surface](issues/008-core-api-surface.md): `Session`, `Party`,
  `Character`. `Anchor` is private, so no address leaves the crate. The caller
  drives the loop.
- [The CLI contract](issues/009-cli-contract.md): a table plus `--json`, with
  settings stored after the first run. The JSON is hand-written so its shape
  does not follow the internal types.
- [What follows from GBS launching DOSBox](issues/011-permission-model.md):
  `--pid` survives as an escape hatch that never tells the user to weaken their
  machine. Dropping the handle leaves the emulator running.

## Not yet specified

- **Conditions and effects.** A separate structure from the character record.
  Needs its own investigation before it can be specified. Done for Unlimited
  Adventures only: the structure is a variable-length chain of ten-byte effect
  records, written up in
  [docs/findings/frua-effect-chain.md](../../docs/findings/frua-effect-chain.md).
  The other eleven games are not checked.
- **Packaging and release.** Crates.io, GitHub releases, AUR, or none of these.
- **The Windows platform layer.** `ReadProcessMemory` instead of
  `process_vm_readv`. Cheap if planned for, painful if bolted on.

## Out of scope

- **Supporting a DOSBox running under Wine.** Jeff plays native. GBS exists to
  remove the Wine layer, not to read through it. The research in
  [002](issues/002-research-linux-process-memory.md) covers the Wine case, and
  that part of it is reference, not a requirement.
- **Writing a v1 specification.** Ruled out when the destination changed. A
  specification written for Jeff to build from has no reader now that the code
  exists. See the closed [010](issues/010-write-v1-spec.md). A README is a
  different document and is ticketed as [012](issues/012-readme.md).
- ~~**Building v1.**~~ Withdrawn on 2026-08-22. Jeff decided that he does not
  want to learn Rust to build this, and asked for the tool to be built for him
  to read. v1 is built and is on `main`. The map's destination changed from a
  specification to working code.
- **The experimental monk class.** Explicitly excluded by Jeff.
- **Implementing a TUI, a GUI, or HUD pinning.** These are roadmap entries, not
  part of this map.
