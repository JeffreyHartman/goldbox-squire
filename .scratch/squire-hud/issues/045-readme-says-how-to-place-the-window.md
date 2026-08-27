# 045 — The README says how to place the HUD window

Type: `wayfinder:task` (AFK)
Status: open
Triage: `ready-for-agent`
Blocked by: 012, 041, 046

## What to build

A README section explaining that Squire does not place its own window, and
giving the user the rule that does.

Wayland does not let a client position its own window, and there is no flag,
protocol or workaround for that. It is the same restriction that makes GBC's
pin-above-DOSBox trick impossible to port, and it is not going away. What the
user gets instead is the app-id from [041](041-name-the-window.md) and one
compositor rule, written once, that pins the HUD to the right edge at a chosen
size on every launch.

The point of the explanation is that a user who expected Squire to place its
own window understands why it does not, rather than filing it as a bug.

Split out of 041 on 2026-08-26: the name is code and the explanation is prose,
and the prose waits for 012 to create the README.

## Acceptance criteria

- [ ] The README states the app-id 041 settled on
- [ ] It explains in two sentences why placement is the compositor's job and
      not Squire's
- [ ] A worked KWin window rule and a worked Hyprland rule are given
- [ ] The README does not promise placement, automatic or otherwise
