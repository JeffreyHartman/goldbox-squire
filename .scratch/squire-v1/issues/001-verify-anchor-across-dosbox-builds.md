# 001 — Verify the name anchor across DOSBox builds

Type: `wayfinder:task` (AFK)
Status: open
Blocked by: none

## Question

The whole design rests on one claim: searching a DOSBox process for a character
name finds the live record, whatever the emulator build. The claim is verified
on native dosbox-staging only.

Verify it on the other native Linux builds Jeff can plausibly play with:
dosbox-staging at other versions, DOSBox-X, and vanilla DOSBox 0.74-3 built for
Linux. Wine builds are out of scope, because Wine is not a target.

For each build, start Pool of Radiance, load a save, begin the game, then scan
the process for the six names. Record whether all six are found, and what the
inter-record gaps are.

If a build fails, that is a finding, not a blocker. Record why.

## Answer

<!-- filled on resolution -->
