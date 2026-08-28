# Goldbox Squire

Goldbox Squire reads the live party out of a running DOSBox and shows it beside
the game. Twelve SSI AD&D Gold Box titles, from Pool of Radiance to Unlimited
Adventures. The binary is called `gbs`.

**You need your own copy of the games.** Squire ships no game data and no game
code. It knows the GOG and Steam layouts, and it will take a typed path to
anything else.

There is already a tool that does this. It is called Gold Box Companion, it is
very good, and it is Windows only, and it works with vanilla DOSBox 0.74-3 only.
I play these games on Linux under a current dosbox-staging, and I did not want
to run a memory reader under Wine to watch a DOS game. The goal, stated in
`docs/roadmap.md`, is to do everything GBC does, on Linux, without Wine.

## Status

Version 0.1.0, one developer, and the beginning of that goal rather than the end
of it.

Today Squire reads and does not write. It shows you name, class, levels, hit
points, armor class, status and ability scores for the party of one save slot,
updated while you play. Not inventory, not conditions and effects, not a map.
Writing is next, not excluded; see below.

Nine of the twelve character record tables are checked against real saves. Three
are not. Countdown to Doomsday and Matrix Cubed run on game folder names that
are community convention rather than anything I have seen on a disk. Gateway to
the Savage Frontier's folder and start command are confirmed against a Steam
install, but that install ships no saves, so its record offsets are still
unread. If you own one of those three, you are the person who can settle it.

There is no packaged release. You build it from source, which is one cargo
command.

## Why it works the way it does

### gbs starts DOSBox, it does not attach to one

The natural shape is: start the game however you like, then point the tool at
the process. Squire works the other way round. You run `gbs`, it asks a few
questions, and it launches the emulator itself.

That is a permission decision. On a distro-default Linux, `kernel.yama.ptrace_scope`
is 1, so one process may read another's memory only if it is an ancestor of it.
If `gbs` starts DOSBox, `gbs` is the parent and the read is simply allowed. No
setuid binary, no sysctl to loosen, and no paragraph in this README asking you
to weaken your machine so a party viewer can work. `--pid` attaches to an
emulator Squire did not start, needs a relaxed `ptrace_scope`, and is therefore
not the default and never will be.

### The party is found by searching for the character's name

A table of fixed addresses falls apart the moment anything shifts: a different
DOSBox, a different memory size, a save loaded mid-session. GBC could assume
vanilla 0.74-3. I cannot assume anything, because running under whatever
emulator you already have is half the point.

So Squire reads the character names out of the save files on disk and scans the
emulator's memory for those bytes. The name is a good needle because the player
typed it and the game never changes it. A hit is then validated field by field
against the legal ranges in the game's table, which is what separates a real
record from a copy of the same name sitting in a file buffer.

Where a record was found is called an anchor, and anchors go stale. Load a save
or restart the emulator and the record moves. Squire notices the name is gone
and searches again. That is the rule the whole reader is built on: a stale
anchor causes a fresh search, never a stale number. A byte the table has no name
for is reported as unknown with its raw value. Squire does not guess.

### A game is a table, not code

Every game has a TOML file in `squire-core/tables/` giving its record layout,
its save shape, and which numbers the engine stores inside out (armor class and
THAC0 are kept as sixty minus the real value). Twelve games sharing one engine
invites twelve special cases inside a reader, each one a place where somebody's
edit for Champions of Krynn quietly changes what Pool of Radiance shows. A table
cannot do that, and fixing an unverified game means editing a file against a
real save rather than learning the reader.

### The window is a separate process, and Squire will not place it

The HUD runs as its own process, connected to the host over a unix socket. The
host is the `gbs` that launched the emulator, and it is the only thing that ever
reads memory, because it is the only thing allowed to. Views draw what they are
sent and send your keys back. Close one and the run continues. That split is
what makes a map or a journal beside the HUD possible later, since only one
process can be DOSBox's parent. The reasoning is in `docs/adr/0005`.

Under Wayland a client cannot position its own window, so Squire does not try.
It names the window instead. The HUD reports the app id `goldbox-squire-hud`,
you write one KWin or Hyprland rule against that name, and it lands where you
want it every launch. Size is Squire's to remember, and it does.

## Where this is going

Writing to memory is the next real step, and it is a question of care rather
than of feasibility. A single byte written to `/proc/<pid>/mem` was accepted by a
running Unlimited Adventures and survived the game's own save, using the same
permission Squire already needs to read (`docs/findings/live-memory-write.md`).
The harder half is knowing when combat is running, because several of these
edits are unsafe mid-fight.

What that unlocks, in the order I intend to build it: heal the party and cure
level drain, save and restore memorised spells, level up without a training
hall, and a character editor that edits memory rather than save files, so the
change shows up in the game immediately. Reading the rest of the record comes
first, since an editor is a display with writes attached. `docs/roadmap.md` has
the full list and the reasoning, including the features I have decided against.

## Build and first run

You need Linux, cargo, a DOSBox on PATH named `dosbox`, `dosbox-staging` or
`dosbox-x`, and a terminal emulator for the HUD window.

```sh
cargo build --release   # target/release/gbs
./target/release/gbs    # then answer the wizard
```

`gbs` asks which game, which directory the first time it sees that game, and
which save slot. Unlimited Adventures gets one more question, which adventure.
Zero goes back in any menu. Then it starts the game, opens the HUD in a window
of its own, and the terminal you typed in becomes the log.

In the HUD: `q`, Escape or Ctrl-C quits the run, `a` shows ability scores, `c`
changes how many cards sit across, and `s` picks a different save slot. That
last one also covers a fresh install with no saves: start it anyway, save inside
the game, then press `s`.

`--plain` reprints the table into the terminal and `--json` prints JSON, for
scripts and for readers I have not written. `gbs --help` lists every flag.

## Reading further

- `CONTEXT.md`: the glossary. One word for one thing, and the reason "slot" and
  "character index" are never the same word.
- `docs/adr/`: the decision records, including the two that reversed earlier
  decisions after an evening of real use.
- `docs/findings/`: the reverse-engineering write-ups.
- `docs/roadmap.md`: everything this might grow, in order.
- `AGENTS.md`: how this project is worked on.

## License

Dual licensed under MIT or Apache-2.0, your choice.

## Next

Confirm the three unverified tables against real installs. Read the rest of the
character record. Then the combat check, which is what stands between here and
the write features.
