# Handoff — Gold Box Companion under Wine, with dosbox-staging

**Date:** 2026-08-21
**Next session's focus:** get the Windows-only Gold Box Companion (GBC) working
under Wine, using **dosbox-staging** rather than vanilla DOSBox 0.74, because the
user specifically wants staging's shaders/CRT filters to play with.

## Read this first

**`~/goldbox/FINDINGS.md`** — the full write-up of the previous session. Do not
re-derive anything in it. It covers the environment, the game file map, GBC's
internal structure, the save/record formats, the memory mechanism, eight
confirmed findings, and a from-scratch reproduction script. Sections most
relevant here are §3 (what GBC is, and the 3-vs-10 tool split), §5 (how GBC's
memory scan works), and §10 (the pragmatic Wine alternative).

There is also a teaching artifact the user asked for, explaining memory
addresses, page offsets, the anchor technique, and GameConqueror/ImHex
walkthroughs: <https://claude.ai/code/artifact/4f009a9f-e28c-4de2-ab3e-e88a75d1b210>

## Where things stand

The previous session was a reverse-engineering spike proving a *native Linux*
GBC port is feasible (memory reading works; the HUD overlay is the hard part).
The user then reasonably concluded that GBC already does everything, so running
the real thing under Wine is the cheaper path. That pivot is why this handoff
exists. The spike work is finished and documented — treat it as reference, not
as work in progress.

### Already installed and working

| Thing | State |
|---|---|
| `dosbox-staging` 0.83.0-RC1 | installed from AUR, works natively |
| Pool of Radiance (GOG 2.0.0.2, game v1.3) | extracted to `~/goldbox/pool-of-radiance/`, game data in `data/POOLRAD/` |
| `~/goldbox/por.conf` | working staging config; auto-mounts and launches the game |
| `~/goldbox/play.sh` | launcher; checks ptrace_scope, backs up saves, prints PID |
| GBC v2.65 (extracted, not installed) | see below for path |
| wine 11.15, winetricks | installed; **no Wine prefix exists yet** |
| innoextract 1.9 | installed |
| scanmem, gameconqueror, imhex | installed |

### Key paths

- Game data: `~/goldbox/pool-of-radiance/data/POOLRAD/` (153 files)
- Custom Python tools: `~/goldbox/tools/` (`gbscan.py`, `gbdiff.py`, `gbregion.py`, `goldbox_char.hexpat`)
- Save snapshots: `~/goldbox/snapshots/` (`baseline2.json` is a clean pre-combat capture)
- GBC v2.65 extracted to **`~/goldbox/gbc/`** (12 `.exe` files + `Games/`,
  `Data/`, `Tools/`, `Resources/`, `Tutorial/`). Source zip is
  `~/Downloads/gbc.zip` if it ever needs re-extracting.

## The central tension — resolve this before doing anything else

The user wants dosbox-staging. GBC has two hard constraints that fight it:

1. **GBC's docs demand vanilla DOSBox 0.74**, stating the DOSBox version matters
   more than the game version, and that even changing soundcard settings shifts
   the memory footprint enough to break its search.
2. **Wine's `ReadProcessMemory` can only see processes Wine manages.** A
   *native Linux* dosbox-staging is invisible to a Wine-hosted GBC. This is not
   a permissions issue and cannot be worked around with sysctl.

So "GBC under Wine + native staging" is **impossible**, not merely difficult.
Do not spend time on it.

### Three viable configurations

**Option A — Windows dosbox-staging inside the Wine prefix. Recommended; try first.**
dosbox-staging publishes Windows builds (verify on
<https://www.dosbox-staging.org/releases/windows/>). Running that *inside the same
Wine prefix* as GBC gives the user their shaders **and** keeps DOSBox
Wine-visible so GBC can read it. Risk: staging's memory layout differs from
0.74, so GBC's default search range may miss. Mitigations are built into GBC
(see diagnostics below). This is the configuration that satisfies what the user
actually asked for, so it deserves the first attempt.

**Option B — Windows vanilla DOSBox 0.74-3 inside the prefix.** GBC's documented,
supported setup. Highest chance GBC works; no shaders. Use as the fallback that
proves GBC itself is functional under Wine, which then isolates whether an
Option A failure is Wine's fault or staging's.

**Option C — native staging + the local Python tools.** Filters work, GBC does
not. Already effectively done; documented in FINDINGS.md.

A sensible sequencing is B-then-A if Option A stumbles, because B establishes a
known-good baseline and turns a vague failure into a specific one.

## Suggested plan

1. **Free win first — the ten file-editor tools.** Only 3 of GBC's 13 tools touch
   process memory (`GBC.exe`, `GBC_Audio.exe`, `ECL_Monitor.exe`). The other ten
   — `SGE.exe`, `ECL_Tool.exe`, `FRUA_Tool.exe`, `FRUA_Module_Manager.exe`,
   `DAXBuilder.exe`, `FontMod.exe`, `IconMod.exe`, `ItemMod.exe`,
   `MonsterMod.exe` — are pure file editors and need no DOSBox at all. Delphi VCL
   apps are historically well-behaved under Wine. Test these before anything
   else: high information, near-zero cost, and independently useful to the user.
2. **Create a dedicated prefix** at `~/goldbox/wine` so nothing else on the
   system is touched. GBC is PE32 i386, so confirm 32-bit execution works under
   this Wine build's WoW64 arrangement.
3. **Put a Windows DOSBox in the prefix** (staging per Option A, else 0.74-3).
4. **Point it at the existing game files.** No Windows GOG installer is needed —
   Wine exposes the already-extracted data as
   `Z:\home\jeff\goldbox\pool-of-radiance\data\POOLRAD`. Mount its parent as `C:`
   so the game's hardcoded `C:\POOLRAD\` paths resolve (see `POOL.CFG`, which is
   35 bytes of plaintext).
5. **Run GBC, load a save, "begin game", then search.** Beginning the game is
   mandatory — the party structures don't exist in memory until then.

## Diagnostics when the search fails

GBC has these built in; they turn guesswork into evidence:

- **`Ctrl+R`** — read test against several addresses. Some failures are normal;
  *all* failures mean GBC cannot see DOSBox's memory at all, which distinguishes
  a Wine visibility problem from a wrong-address-range problem.
- **`Ctrl+D`** — debug logging.
- **Widen the search range** in the wizard's ADDRESS RANGE field (increase the
  upper bound). This is the documented fix for a shifted memory footprint and is
  the most likely thing Option A needs.

Facts from the spike that make failures interpretable — full detail in
FINDINGS.md §6:

- The 285-byte character record sits **contiguously** in DOSBox's memory,
  276/285 bytes identical to the on-disk `CHRDATA1.SAV`. Bytes 0–126 never
  change and make a perfect signature.
- Under native staging, THRENDER GRONE's record is at **guest address
  `0x3E1A8`** every run (deterministic), i.e. segment `0x3E1A`.
- Slot A party: THRENDER GRONE, BAKSHI, RHIANNON, BROTHER SEAN, DARKSTAR,
  PHINEAS. Offset **283** of the record is current HP (confirmed).

So if GBC reports "characters not found", the local tools can independently
confirm whether the record is present and where — turning an opaque GBC failure
into a concrete offset discrepancy.

## Gotchas

- `kernel.yama.ptrace_scope` resets to `1` on every reboot. It is **irrelevant to
  the Wine path** (Wine's own memory reads are in-prefix), but needed if the
  local Python tools are used for cross-checking:
  `sudo sysctl kernel.yama.ptrace_scope=0`.
- The GOG-bundled Linux DOSBox is broken on this system (2014 SDL1 build wanting
  `libFLAC.so.8`). Ignore it.
- GBC's HUD floats over the DOSBox window by polling its position — a Win32
  pattern. With both processes inside Wine this should work, but expect
  roughness under Wayland/XWayland. `HUD ON TOP` and `TITLE HACK` are relevant
  GBC settings.
- PoR is **PC-speaker only** — no AdLib or MIDI. Near-silence is correct, not a
  bug. Don't chase it.
- GBC's `Setup GOG` button expects a Windows GOG install layout and will likely
  not help here; set the game folder manually.

## How the user wants to be worked with

Read `~/.claude/projects/-home-jeff-git/memory/explain-as-you-go.md`. Summary:
he is deliberately trying to rebuild his own skills after a year of over-relying
on LLMs. He asked twice to slow down. Say what each artifact is *as* it's
created, keep a running inventory when several accumulate, prefer handing him
commands to run himself over silently running them, explain *why* a tool or
approach was chosen, and mark hypotheses as hypotheses. He does not want less
work done — he wants the same work, legible. Caveman mode is off system-wide
(`~/.config/caveman/config.json`); do not re-enable it.

## Suggested skills

Call the Skill tool for:

- **`research`** — if Wine/GBC compatibility questions need answering against
  primary sources (WineHQ AppDB, gbc.zorbus.net, the dosbox-staging release
  pages, the German Ubuntu walkthrough GBC's site links). Useful for confirming
  whether Windows dosbox-staging builds exist and which Wine settings others
  needed. Captures findings as Markdown rather than leaving them in chat.
- **`diagnosing-bugs`** — when GBC launches but its character search fails. That
  is a genuine multi-hypothesis debugging problem (Wine visibility vs. address
  range vs. staging layout vs. game state), and the diagnosis loop suits it
  better than ad-hoc poking.
- **`artifact-design`** — only if producing another visual explainer. The user
  responded well to the last one; load this before writing any artifact.

Not needed: `claude-api` (no LLM API work here), the code-review skills (no
codebase under review).
