# 032 — Steam's Unlimited Adventures splits the game across an overlay

Type: `wayfinder:task`
Status: open

## Question

SNEG's Steam build of Unlimited Adventures mounts two host folders as one
DOS drive:

    mount c .\GAME
    mount c .\DESIGNS -t overlay

`GAME\UA` holds the game (CKIT.EXE, START.BAT, no designs); `DESIGNS\UA`
holds the designs and their saves (and catches every write). Discovery
looks for one folder named `UA` that holds both the start file and the
designs, so this install is invisible: each half fails one test.

Supporting it means an install whose game folder is two host directories,
with saves read from the overlay half and the computed autoexec issuing
both mounts. Decide whether that is worth a third install shape, or whether
pointing `--game-dir` at `DESIGNS\UA` manually is enough (today the watch
works that way, but launching does not).

The user's own FRUA copies (`~/goldbox/frua`, `~/goldbox/frua-mm`) merge
the two halves into one tree, so they are unaffected.
