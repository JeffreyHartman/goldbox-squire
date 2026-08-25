# 017 — Launch an install, not a bare emulator

Type: `wayfinder:task` (AFK)
Status: done
Blocked by: none

## What to build

The launcher starts DOSBox with at most one `-conf` and inherits all three
standard streams. Both real installs need more, and the wizard needs the
terminal for itself.

- Accept an ordered list of conf files and pass each as `-conf`, in order.
- Set the child's working directory to the folder holding the confs. Both
  publishers' autoexecs use relative mounts (`mount c "data"`,
  `mount c .\GAME`), so without this the mount fails.
- Send the child's stdout and stderr to a log file under the config directory,
  so a failed launch stays diagnosable without the emulator printing over the
  party table. Give the child a null stdin, so `gbs` owns the keyboard.
- The emulator ending is not an error. Both publishers' autoexecs end in
  `exit`, so quitting the game closes DOSBox in every normal setup. This is
  how sessions end, not an edge case.

The current inherit behaviour was deliberate (the emulator's messages reached
the user). The log file replaces that on purpose; name the log path in the
launch message so the messages stay reachable.

## Acceptance criteria

- [x] Multiple confs are passed in order.
- [x] The working directory is the conf folder.
- [x] Child output lands in a log file whose path the user is told.
- [x] The child's stdin is null.
- [x] The emulator was left running on exit before; it still is.

## Answer

`Emulator` gained `conf` (one call per file, launch order preserved),
`current_dir` and `log_to`. `start()` gives the child a null stdin always;
with `log_to` its stdout and stderr go to that file, without it they stay
inherited. The CLI starts the emulator in the folder holding the confs (the
conf's own folder for an absolute manual conf, the install root otherwise),
logs to `emulator.log` beside the config file, and names that path in the
launch message. The emulator stays running on exit, as before; the quit path
itself is ticket 018.
