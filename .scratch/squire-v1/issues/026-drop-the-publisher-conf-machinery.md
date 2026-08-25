# 026 — Drop the publisher conf machinery

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: 025

## What to build

Delete what ADR 0003 obsoleted, and fix the duplicate it exposes.

**Deletions.** Discovery stops recording conf lists and bundled emulators:
conf ordering, launch-script conf parsing, and the bundled-emulator hunt go
away. A conf with an `[autoexec]` mount stays as the structural signature of
an install, and the publisher is now read from the launch script's presence
(`start.sh` is GOG, `run-game.bat` is Steam). Discovered installs are written
to the config without `confs` or `emulator`; old config files that still
carry them load fine and the values are ignored for non-manual kinds.

**The duplicate.** A hand setup next to a real install (a conf at
`~/goldbox/por.conf` mounting into the GOG folder) makes discovery report the
same game folder twice: once as `found`, once as `gog`. Now that neither
install's confs matter, the two are the same install. Deduplicate on the
canonical game folder; a publisher-scripted install wins over a `found` one.

Update the usage text, CONTEXT.md, and tickets 016 and 023 to match.

## Acceptance criteria

- [x] Discovered installs carry no conf list and no emulator.
- [x] Publisher detection still works from the scripts' presence.
- [x] Two discovered installs sharing one game folder collapse to one, the
      publisher-scripted one.
- [x] The wizard, launch, and watch flow work end to end for a discovered
      install and for a manual one.

## Answer

`DiscoveredInstall` lost `confs` and `emulator`; `order_confs`,
`confs_named_in`, and `bundled_emulator` are gone. The install signature is
unchanged (a `.conf` with an `[autoexec]` mount, now via
`has_conf_signature`), and the publisher comes from `publisher_of`: a
`start.sh` is GOG, a `run-game.bat` is Steam. `Config::absorb` writes
discovered installs with no confs and no emulator; old config files still
load and the stale values are ignored for non-manual kinds.
`dedup_by_game_folder` collapses installs whose canonical game folder is the
same, publisher-scripted first, which removes the `found` duplicate a hand
conf above a GOG folder used to create. Code review added two fixes for
existing configs: `absorb` now replaces discovered installs wholesale, so a
rescan is the authority on them, and `needs_rediscovery` also fires when two
discovered installs reach one canonical game folder, so the pre-026
`found:`/`gog:` duplicate collapses on the next run. The registry's
`save_folder` was renamed `game_folder` to match what it names. The
end-to-end flow was verified with a stub emulator that records its
arguments.
