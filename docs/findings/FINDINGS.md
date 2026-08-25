# Gold Box on Linux — findings

Working notes from reverse-engineering Pool of Radiance running under
dosbox-staging on Linux, with an eye toward whether Gold Box Companion (GBC)
could be ported or reimplemented natively.

**Status:** research spike. Everything below marked *confirmed* was verified
against a live process, not inferred. Everything marked *hypothesis* was not.

Last updated: 2026-08-21

---

## 1. Environment

| Thing | Value |
|---|---|
| Game | Pool of Radiance, GOG build 2.0.0.2, game version 1.3 |
| Installed at | `~/goldbox/pool-of-radiance/` |
| Game data | `~/goldbox/pool-of-radiance/data/POOLRAD/` (153 files) |
| Emulator | `dosbox-staging` 0.83.0-RC1 (AUR `1:0.83.0_RC1-2`) |
| Launch | `dosbox --conf ~/goldbox/por.conf` or `~/goldbox/play.sh` |
| GBC | v2.65 (10-Jun-2021), extracted from `gbc.zip`, Windows-only |
| Host | CachyOS, kernel 7.2.0, PipeWire |

The GOG `.sh` installer is a Makeself/MojoSetup wrapper with a plain zip
appended. It was extracted directly rather than run, so there are no desktop
entries or `.mojosetup` state. Raw installer scaffolding is at `~/goldbox/por-raw/`.

**The bundled GOG DOSBox does not run** on a current system — it's a 2014 SDL1
build that wants `libFLAC.so.8`, and GOG's bundled `dosbox/libs/x86_64/` doesn't
include it. Not worth patching; use dosbox-staging.

### Config notes

`~/goldbox/por.conf` layers over `~/.config/dosbox/dosbox-staging.conf`
(passing `--conf` does not replace the primary config; `--noprimaryconf` does).
Only non-default settings are in it:

- `machine = ega` — PoR is EGA, and `POOL.CFG` line 1 is `E`
- `compressor = off` — so single PC-speaker beeps aren't ducked
- `[autoexec]` mounts `data/` as `C:` and runs `POOLRAD\START.EXE`

Option names changed in 0.83: it's `cpu_cycles` (not `cycles`) and `shader`
(not `glshader`). Defaults are already correct for a 1988 title
(`cpu_cycles = 3000`, `shader = crt-auto`, `pcspeaker = impulse`, `memsize = 16`).

**Sound:** PoR is PC-speaker only. No AdLib, no MIDI — the Gold Box engine
didn't get sound card support until the Krynn-era titles. The game is nearly
silent by design: a title jingle, menu blips, combat hits. Audio config was
verified correct (`nosound = off`, `pcspeaker = impulse`), so silence during
exploration is expected behaviour, not a fault.

---

## 2. The toolkit built here

All in `~/goldbox/tools/`. Plain Python 3, no dependencies.

| File | Purpose |
|---|---|
| `gbscan.py` | Find character records in a running DOSBox by using a save file as a byte signature. Reports host addresses. |
| `gbdiff.py` | Re-read the six character records and report every byte that changed vs. a snapshot. Re-locates records by name each run. |
| `gbregion.py` | Snapshot/diff an arbitrary range of the **emulated machine's** memory, addressed as the DOS program sees it. Auto-detects MemBase. |
| `goldbox_char.hexpat` | ImHex pattern (struct definition) for the 285-byte character record. |
| `../play.sh` | Launch wrapper: checks `ptrace_scope`, backs up saves, starts DOSBox, prints the PID. |
| `../por.conf` | DOSBox config described above. |

### Typical session

```bash
sudo sysctl kernel.yama.ptrace_scope=0     # once per boot; reverts on reboot
~/goldbox/play.sh                          # prints the PID

# load a save, "begin game", then:
python3 ~/goldbox/tools/gbscan.py --pid PID \
    --save-dir ~/goldbox/pool-of-radiance/data/POOLRAD --slot A

# whole-machine before/after, for anything not in the records
python3 ~/goldbox/tools/gbregion.py --pid PID --snap /tmp/before.bin
# ...do ONE thing in the game...
python3 ~/goldbox/tools/gbregion.py --pid PID --diff /tmp/before.bin --hide-noise
```

### Permissions

`kernel.yama.ptrace_scope = 1` (the distro default) allows memory reads only
from an **ancestor** of the target process. A scanner started from a different
shell is a *sibling*, which is denied. Options:

1. `sudo sysctl kernel.yama.ptrace_scope=0` — per boot, what was used here.
2. Persist via `/etc/sysctl.d/` — standing reduction in isolation between your
   own processes. Not done.
3. **Have the tool launch DOSBox itself** — then it's an ancestor by
   construction and no privilege is needed, ever.

Option 3 is the correct design for a real tool and is a genuine improvement over
GBC's model. GBC attaches to an already-running DOSBox, which is fine on Windows
(no Yama equivalent) but fights the kernel on Linux. A launcher also removes the
need for window-title matching, since you'd own the PID and window from birth.

---

## 3. What GBC actually is

Delphi (Object Pascal), PE32 i386, native code. Not .NET — there is no IL to
decompile, so its logic is only recoverable by x86 disassembly. Its data,
however, is largely plain text.

### Tool inventory, and the split that matters

| Needs DOSBox process memory | Pure file editors |
|---|---|
| `GBC.exe`, `GBC_Audio.exe`, `ECL_Monitor.exe` | `SGE.exe` (save editor), `ECL_Tool.exe`, `FRUA_Tool.exe`, `FRUA_Module_Manager.exe`, `DAXBuilder.exe`, `FontMod.exe`, `IconMod.exe`, `ItemMod.exe`, `MonsterMod.exe` |

**Only 3 of 13 tools touch process memory.** The other 10 are file editors and
should run under plain Wine with no DOSBox-under-Wine contortion at all. This
was never tested here but is the cheapest thing to try.

### Reusable data (no reverse engineering needed)

- `Games/<NN. Game>/` × 12 — `Effects.txt`, `Levels.txt`, `XP.txt`, `Icons.txt`,
  `Items/*.itm`, `World.bmp`, `Body.bmp`/`Head.bmp` sprite sheets
- `Data/` — `ECL.dat` (90 KB), `ECL_Lengths.dat`, `Experience.dat`
- `Games/*/ECL-Tool/` — a **3931-line `ECL - Addresses.txt`** cross-referencing
  every `$XXXX` DOS address to the `ECL#.DAX` record and line that touches it,
  plus `ECL - Flags.txt` (3361 lines), `ECL - Comments.txt`,
  `ECL - Forced data offsets.txt`. 9232 lines of RE notes for PoR alone.
- `DAX-Icons-EGA/`, `DAX-Icons-VGA/`, `DAX-Fonts/`, EGA/VGA palette PNGs

### The one blocker for a port

`Games/*/Game.dat`, ~557–620 KB each, one per game. Binary Delphi record dump.
Strings show the game name, launcher path, `dosbox_por.conf`, GOG cloud-save
path, save filename patterns, then map names interleaved with nibble-packed wall
grids. **The memory offsets almost certainly live in the fixed-width header.**

Mapping this layout is the gate on any port. Conditions are favourable — 12
near-identical files differing only in values — but it was not attempted.

### Credits / prior art GBC itself cites

- Simeon Pilgrim — decrypted and reimplemented Curse of the Azure Bonds in C#:
  <https://github.com/simeonpilgrim/coab>
- Stephen S. Lee — ECL flags and addresses (his PoR/CotAB FAQs)
- Bil Simser — Gold Box Explorer: <https://github.com/bsimser/Gold-Box-Explorer>
- FRUA forums (marainein, Ishad Nha, others) for early ECL analysis

---

## 4. Pool of Radiance file map

153 files in `data/POOLRAD/`.

### Config — trivial

`POOL.CFG`, 35 bytes, **plaintext**:

```
E              <- EGA
P              <- sound device, almost certainly PC speaker (hypothesis)
C:\POOLRAD\    <- data path
C:\POOLRAD\    <- save path
F
```

### Code — hard

| File | What |
|---|---|
| `START.EXE` | 65 KB MZ DOS exe, launcher stub |
| `GAME.OVR` | **206 KB — the engine.** 8086 code in a DOS overlay file. |

`GAME.OVR` requires disassembly. Prior art exists (CoAB above).

### Assets — the DAX container, solved

113 of 153 files are `.DAX`: run-length encoded, record-indexed archives. Format
is fully solved — GBC's `DAXBuilder.exe` extracts and rebuilds them, and Gold
Box Explorer reads them. The header is a record table (id, offset, packed size,
unpacked size) followed by RLE payload.

| Family | Count | Contents |
|---|---|---|
| `ECL1-8.DAX` | 8 | **Bytecode scripts** — plot, encounters, dialog, text, treasure |
| `GEO1-8.DAX` | 8 | Map geometry / wall grids (source of GBC's automap) |
| `WALLDEF1-8.DAX` | 8 | Wall definitions, tile → graphic mapping |
| `8X8D1-8.DAX` | 8 | 8×8 tile graphics |
| `PIC1-8`, `CPIC1-8` | 16 | Full-screen art; combat backdrops |
| `SPRIT1-8`, `COMSPR`, `ICON` | 10 | Sprites, combat sprites, icons |
| `HEAD1-8`, `BODY1-8`, `CHEAD`, `CBODY` | 18 | Portrait paper-doll parts |
| `ITEM1-8.DAX` | 8 | Item definitions per area |
| `MON1-8{CHA,ITM,SPC}.DAX` | 22 | Monster stats / their items / their spells |
| `DUNGCOM`, `WILDCOM`, `RANDCOM` | 3 | Combat map templates |
| `TITLE`, `FINAL5`, `SQRPACI`, `BACPAC` | 4 | Title, endgame, misc |
| `ITEMS` (no ext) | 1 | 2050 bytes, likely master item table *(hypothesis)* |

The `1-8` suffix is the game's area partitioning, and maps 1:1 onto GBC's
`Games/01. Pool of Radiance/ECL-Tool/mon1cha.txt` … `mon8cha.txt`.

**ECL is the interesting frontier.** GBC's docs are candid that its parser is
incomplete because ECL interleaves code and data, some opcodes are one byte, and
*"in some cases it's almost impossible to distinguish code and data"* — hence the
manual `ECL - Forced data offsets.txt` overrides. That's a decidability problem,
and the kind of grind that scales well with an LLM in the loop. Probably the most
novel contribution available in this space.

### Saves — trivial

| File | Size | What |
|---|---|---|
| `SAVGAM{A..J}.DAT` | 13137 | Party + world state, one per slot |
| `CHRDAT{slot}{1..6}.SAV` | 285 | One character record |
| `CHRDAT{slot}{n}.ITM` | 63–189 | That character's inventory |
| `CHRDAT{slot}{n}.SPC` | 9–45 | Memorized spells |

GOG ships slots **A** and **J** populated. GBC's `Game.dat` contains the format
strings `CHRDAT%s%d.SAV`, `CHRDAT%s%d.ITM`, `CHRDAT%s%d.SPC`, `SAVGAM%s.DAT` —
matching exactly.

No encryption, no checksum, no obfuscation:

```
00000000: 0e 54 48 52 45 4e 44 45 52 20 47 52 4f 4e 45 00  .THRENDER GRONE.
00000010: 11 0c 0c 11 10 0f ...                            STR/INT/WIS/DEX/CON/CHA
```

Slot A party: THRENDER GRONE, BAKSHI, RHIANNON, BROTHER SEAN, DARKSTAR, PHINEAS.

---

## 5. How the memory mechanism works

GBC's approach, reconstructed from its docs and confirmed by reimplementing it:

1. Parse `CHRDAT<slot>1.SAV` from disk — 285 bytes of known layout.
2. Treat those bytes as a **signature**.
3. Scan the DOSBox process's memory for them.
4. The hit is an **anchor**. Everything else — HP, XP, effects, map position,
   combat grid — is at a fixed offset from it, and those offsets are what's
   baked into `Game.dat`.

The save file isn't the data source; it's the *key that unlocks the address
space*. This is why GBC's docs insist you "begin the game" first (the structures
don't exist until then) and why changing DOSBox's soundcard settings breaks it
(different allocation order shifts the layout).

**This design ports to Linux cleanly.** You never need absolute addresses — you
derive them at runtime. `process_vm_readv` / `/proc/<pid>/mem` replaces
`ReadProcessMemory`, and that's the whole translation.

---

## 6. Confirmed findings

### 6.1 The record exists contiguously in live memory — confirmed

Comparing the on-disk `CHRDATA1.SAV` against the same record found in the
running process: **276 of 285 bytes identical (96.8%)**.

The 9 differing bytes are three identical triplets:

| Offsets | disk | live |
|---|---|---|
| 127, 129–130 | `14`, `0x4EE5` | `8`, `0x3B5E` |
| 200, 202–203 | `0`, `0x4EDE` | `8`, `0x3E31` |
| 260, 262–263 | `2`, `0x4EE8` | `8`, `0x3E3A` |

A flag byte plus a 16-bit DOS segment, relocated on load (`0x4Exx` → `0x3Bxx`/
`0x3Exx`). Everything else — name, stats, class, HP, XP — is byte-identical.

**Consequence:** bytes 0–126 never change, making an ideal scan signature.

### 6.2 The guest-side layout is deterministic — confirmed

Across a reboot, a new PID, and a fresh load of the same save, all six records
were **byte-for-byte identical** and the inter-record distances were unchanged:

```
THRENDER GRONE  →  BAKSHI          +0x0210  (528)
BAKSHI          →  RHIANNON        +0x01b0  (432)
RHIANNON        →  BROTHER SEAN    +0x01f0  (496)
BROTHER SEAN    →  DARKSTAR        +0x01e0  (480)
DARKSTAR        →  PHINEAS         +0x0160  (352)
```

Gaps are uneven because each character's **inventory is stored immediately
behind their record**, and inventories vary in length. Confirmed: THRENDER's
items begin at `record + 0x180`, and past the last character `Dagger` and
`4 Darts` are readable as ASCII.

Determinism is the property everything else depends on. Without it, no
offset-based tool could be reliable.

### 6.3 MemBase is locatable at runtime — confirmed

Guest address 0 was found in host memory at `0x7f49808ab000` (that run) by
fingerprinting the **interrupt vector table**: DOS keeps 256 far pointers at
guest `0x000–0x3FF`, and DOSBox aims many at its own BIOS handlers in segment
`0xF000`, so a run of entries ending `00 f0` marks guest zero. MemBase is
page-aligned and the IVT is only 1 KB, so at most one page-aligned candidate can
explain a match.

Cross-validated two independent ways:

- THRENDER's record lands at guest `0x3E1A8` — inside conventional memory
  (< 640 KB), where a DOS program's data belongs.
- `0x3E1A8` is segment `0x3E1A`, and the pointers inside the record hold
  `0x3EC8`–`0x3ED0`. Same neighbourhood — the pointed-to array sits ~2.7 KB past
  the party block.

This also explains the stable low bits of the host address: the record's *guest*
address is always `0x3E1A8` and MemBase is always page-aligned, so
`addr & 0xFFF == 0x1A8` every run. Arithmetic, not coincidence.

A naive BDA sanity check (word at `0x413` should be 640 KB) returned 2, so
either dosbox-staging populates the BDA differently or that assumption was
wrong. The two checks above are stronger, so MemBase is treated as confirmed.

### 6.4 Offset 283 (`0x11B`) is current HP — confirmed

Only Phineas took damage across three observations:

| Observation | Phineas off 283 | Other five |
|---|---|---|
| baseline | 6 | unchanged |
| after being hit | 3 | unchanged |
| after being hit again, died | 0 | unchanged |

One character damaged, one byte moving monotonically to 0 at death, and **no
change at that offset for the five who weren't hit**. Subsequent read of all six
gave `11, 7, 7, 10, 5, 0` — plausible level-1 AD&D hit points with the dead
character at zero.

*Still worth doing:* heal someone and confirm it moves back up.

### 6.5 Combat position is NOT in the character record — confirmed

Two consecutive whole-record diffs, with **one character moved one square north**
between them, were byte-for-byte identical. Position lives in a separate
structure.

### 6.6 The record contains a table of 4-byte pointer slots — confirmed

The changed offsets during combat fall into a repeating 4-byte shape,
`[flag, pad, word_lo, word_hi]`, at 200, 204, 208, 212, … and again at 260, 264,
268. The three triplets in §6.1 are just three members of this table.

During combat every flag became `8`, and every high byte moved from `0x4E`/
`0x4F`/`0x50` down to `0x3E` — the same relocation signature as §6.1.

### 6.7 A per-character combat-entity array is allocated on combat — confirmed

The slot at 264/266–267 gained a value for **all six** characters when combat
started, sequentially:

| Character | word @266 | → guest |
|---|---|---|
| THRENDER | `0x3EC8` | `0x3EC80` |
| BAKSHI | `0x3ECA` | `0x3ECA0` |
| RHIANNON | `0x3ECB` | `0x3ECB0` |
| BROTHER SEAN | `0x3ECD` | `0x3ECD0` |
| DARKSTAR | `0x3ECE` | `0x3ECE0` |
| PHINEAS | `0x3ED0` | `0x3ED00` |

Following them (via MemBase) yields 16-byte structures with small values and
`ffff` terminators. Spacing alternates 32/16 bytes; **hypothesis:** the kobolds
occupy the gaps, i.e. the array holds all combatants, not just the party. This
is almost certainly where combat position lives, and is what GBC's combat view
reads.

### 6.8 Noise floor is very low — confirmed

A 640 KB snapshot of all conventional memory, diffed against itself seconds
later with nothing happening in-game, showed **3 changed runs**: the BIOS timer
tick at `0x47C` and two bytes at `0x1A5FA` (likely a random seed). Signal-to-noise
for one-action experiments is excellent.

---

## 7. Character record map (285 bytes)

| Offset | Size | Field | Status |
|---|---|---|---|
| 0 | 1 | Name length | confirmed |
| 1–15 | 15 | Name, padded to fixed width | confirmed |
| 16–21 | 6 | STR, INT, WIS, DEX, CON, CHA | confirmed |
| 22–43 | 22 | Unknown — class/race/level/alignment likely | unknown |
| 44 | 1 | Held `0x28` (40) for all six characters | unexplained |
| 45–126 | 82 | Unknown — XP, AC, saves in here | unknown |
| 127 | 4 | Pointer slot (flag + segment) | confirmed volatile |
| 131–199 | 69 | Unknown | unknown |
| 200 | 4 | Pointer slot | confirmed volatile |
| 204, 208, 212 | 12 | Pointer slots (active in combat) | confirmed |
| 216–259 | 44 | Unknown | unknown |
| 260 | 4 | Pointer slot | confirmed volatile |
| 264 | 4 | Pointer slot → combat-entity array | confirmed |
| 268–269 | 2 | Changed only on death (`0→5`, `1→0`) | hypothesis: status |
| 275 | 1 | Varies in combat; was 3 for the character who moved | unknown |
| 282 | 1 | Was 0 for all six | unknown |
| **283** | **1** | **Current HP** | **confirmed** |
| 284 | 1 | Unknown | unknown |

Roughly 250 of 285 bytes remain unmapped. All are findable with the
one-action-then-diff loop.

---

## 8. Open questions

1. **`Game.dat` record layout.** The gate on any real port. 12 near-identical
   files; favourable conditions; not attempted.
2. **Combat position.** Known to be outside the record, and the combat-entity
   array (§6.7) is the prime suspect. Next experiment: `gbregion.py --snap`,
   move one square, `--diff`.
3. **What offset 44 (`0x28`) means.** Identical across all six characters.
4. **Whether `POOL.CFG` line 2 (`P`) is the sound device.** 35-byte plaintext
   file; trivial to test by editing.
5. **The HUD overlay problem.** Never addressed. GBC polls a foreign window's
   position and floats a child on top — Win32 semantics with no Wayland
   equivalent. Options, ascending in effort/quality: detached window; X11
   override-redirect; **dosbox-staging fork rendering the HUD in-emulator**
   (best UX, also sidesteps process scanning entirely); or use staging's
   debugger interface as the read channel instead of raw scanning.
6. **Why the BDA check disagreed** with the otherwise-confirmed MemBase.

---

## 9. Difficulty ranking, for anyone picking this up

1. **Trivial** — `POOL.CFG`, save files. Plaintext or near it.
2. **Solved** — DAX extraction, graphics, items, monsters, map geometry.
   Format known, working tools exist, just reimplement.
3. **Medium, genuinely open** — ECL bytecode. Parser exists at ~95%; the
   remainder is code-vs-data discrimination.
4. **Hard, prior art exists** — `GAME.OVR` disassembly. See CoAB.
5. **The GBC port** — not game hacking at all: `Game.dat` layout, the Linux
   memory read (**done, see §5–6**), and the HUD overlay (the actual hard part).

---

## 10. The pragmatic alternative

None of the above is required to *play with GBC's features*. Two much cheaper
paths, in order:

1. **The 10 file-editor tools under plain Wine.** No DOSBox-under-Wine needed —
   they only touch files. Untested, but should be close to free. Covers save
   editing, FRUA module management, DAX packing, font/icon/item/monster modding.
2. **Full GBC under Wine**, which additionally needs a **Windows** DOSBox 0.74
   running in the *same Wine prefix*, because Wine's `ReadProcessMemory` can
   only see processes it manages — it cannot read a native Linux dosbox. GBC's
   author links a German-language Ubuntu walkthrough; community reports are
   mixed. Note GBC's docs demand vanilla 0.74 specifically, not staging.

The work in this document is a research spike, not a prerequisite. Its value is
that it establishes the Linux memory-read half of a native port is
straightforward, and that the remaining obstacles are `Game.dat` and the HUD —
not the game itself.

---

## 11. Reproducing everything here from scratch

```bash
# 1. install
paru -S dosbox-staging
sudo pacman -S scanmem gameconqueror     # Cheat-Engine-style live scanning
paru -S imhex-bin                        # hex editor with struct patterns

# 2. extract the GOG installer (do not run it)
mkdir -p ~/goldbox && cd ~/goldbox
unzip -q gog_pool_of_radiance_2.0.0.2.sh -d por-raw
mv por-raw/data/noarch pool-of-radiance

# 3. allow memory reads (per boot)
sudo sysctl kernel.yama.ptrace_scope=0

# 4. play, then scan
~/goldbox/play.sh
python3 ~/goldbox/tools/gbscan.py --pid PID \
    --save-dir ~/goldbox/pool-of-radiance/data/POOLRAD --slot A
```

Snapshots of the six records live in `~/goldbox/snapshots/`. `baseline2.json`
is a clean pre-combat capture.
