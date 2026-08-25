# 029 — Fold the proven settings into the template

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

The settings-conf template gains the defaults proven in `~/goldbox/por.conf`
(ADR 0004: the hand conf's job moves here):

- `machine = ega` — the faithful choice for an EGA-era engine; the CRT
  shader then picks EGA scanline behavior.
- `pcspeaker = impulse` and `pcspeaker_filter = on` — the high-accuracy
  speaker model.
- `mouse_capture = seamless` — the pointer never locks; these games are
  keyboard-driven.
- `compressor = off` and `tandy = on` stay from ticket 025's follow-up.

Settings other emulators do not know are ignored by them with a warning,
which is acceptable. The file is created once and never touched again, so
users with an existing settings conf must delete it (or copy lines) to pick
up new defaults; the template comments keep saying the file is theirs.

## Acceptance criteria

- [x] A fresh settings conf contains the five settings above.
- [x] An existing settings conf is still never touched.

## Answer

The template now sets `machine = ega`, `pcspeaker = impulse`,
`pcspeaker_filter = on`, `tandy = on`, `compressor = off`,
`mouse_capture = seamless`, and `fullscreen = false`, each with a comment
saying why. A headless probe under dosbox-staging 0.83 confirmed the
speaker's impulse model and the Tandy DAC both initialise under
`machine = ega` with `tandy = on`, so the Steam install keeps its sound.
Existing settings confs are untouched as promised; delete one to regenerate
it with the new defaults.
