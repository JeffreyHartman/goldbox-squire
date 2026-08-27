# 046 — One app id per view

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: none

## What to build

A prefactor, so that it lands before anything spawns a window.

041 pinned one window name, `goldbox-squire`, and took the id parameter out of
`Terminal::command_line`. Its reason was that "there was never more than one
name to pass". That is no longer true. [ADR 0005](../../../docs/adr/0005-one-host-reads-many-views-draw.md)
gives every view kind its own window, and the user writes one compositor rule
per window. One shared name would place the map wherever it places the HUD.

So the name comes back as a parameter, but a typed one: a view kind, from a
fixed list in the code. The HUD is `goldbox-squire-hud`. A caller still cannot
invent a name, which was 041's real worry.

## Acceptance criteria

- [ ] Each view kind has one owned name, and the HUD's is `goldbox-squire-hud`
- [ ] `Terminal::command_line` fills `{id}` from the view kind it is given
- [ ] No caller can pass a name that is not on the list
- [ ] `terminals.toml` says what `{id}` is filled with now
- [ ] `CONTEXT.md`'s App id entry matches
