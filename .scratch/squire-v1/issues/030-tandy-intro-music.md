# 030 — The Tandy intro music

Type: `wayfinder:task` (HITL)
Status: open
Blocked by: 029

## Question

With `tandy = on`, the Steam install (whose POOL.CFG names Tandy sound) has
in-game sound, but the title-screen music is reportedly missing. Hypothesis:
the game only plays the intro tune when it detects an actual Tandy machine
at startup, and `tandy = on` provides the sound chip without the machine
identity. The candidate remedy is `machine = tandy`, but that also changes
the emulated video hardware while POOL.CFG line 1 says EGA.

Needs ears at the keyboard:

- Does `machine = tandy` bring the intro music back?
- What does it do to the video, with POOL.CFG saying `E`?
- Does the GOG install (PC speaker) still sound right under whatever the
  answer is?

Record the finding and, if the remedy works, decide whether it becomes the
template default or a commented-out line in it.

## Answer

<!-- filled on resolution -->
