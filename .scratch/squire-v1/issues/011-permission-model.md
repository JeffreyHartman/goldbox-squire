# 011 — What follows from GBS launching DOSBox

Type: `wayfinder:grilling` (HITL)
Status: CLOSED 2026-08-22, answered by the code
Blocked by: none

## Question

Graduated from the fog by [002](002-research-linux-process-memory.md).

**Settled 2026-08-22, by Jeff, before this ticket was worked:** GBS launches
DOSBox as its own child process. Yama's descendant rule then permits the memory
read. The alternatives are rejected. `setcap cap_sys_ptrace+ep` grants the tool
the right to read every process on the machine, which is far more than it needs.
`sudo sysctl kernel.yama.ptrace_scope=0` weakens the whole machine, resets on
every reboot, and pushes a security decision onto the user. GBS is meant to be
released and used by other people, so it must work without either.

What is left to decide follows from that.

- **Does an attach mode survive at all?** The map previously settled
  "auto-detect, with `--pid` to override". Launching makes auto-detect
  unnecessary. Is `--pid` kept as an escape hatch for a DOSBox the user started
  by hand, knowing that it fails at `ptrace_scope` 1 and needs an explanation?
  Or is launching the only supported path, which is simpler to document and to
  support?
- **What does GBS need to know to launch?** The emulator binary, the config file,
  and the arguments. Which are flags, which live in the config file, and what
  happens on a machine where dosbox-staging is not on `PATH`.
- **What owns the process lifetime?** GBS starts DOSBox. When the user quits
  DOSBox, does GBS exit? When the user stops GBS, does DOSBox die with it? A
  reader that kills the game on Ctrl-C is a bad tool.
- **The failure message.** [002](002-research-linux-process-memory.md) gives the
  exact errno for each read method: `EPERM` from `process_vm_readv`, `EACCES`
  from opening `/proc/<pid>/mem`. Under the launch model these must be rare.
  When one appears, the message must name the cause and the remedy, and it must
  not suggest a machine-wide sysctl first.

## Answer

**GBS launches DOSBox. `--pid` survives as an escape hatch. The handle never kills the emulator.**

`squire-core/src/launch.rs` and `squire-cli/src/main.rs`.

- **Attach mode survives**, as `--pid`. Its help text says it works only where
  the system already permits it. It does not tell the user to lower
  `kernel.yama.ptrace_scope`, because the map rejects shipping a security
  downgrade as an install step.
- **To launch**, GBS needs the emulator command and an optional config file.
  Both are flags, both are stored in the config file. The command defaults to
  `dosbox` and is looked up on `PATH`.
- **Process lifetime**: dropping the handle deliberately does not stop the
  emulator, and `gbs` leaves it running when it exits. A tool that reads the
  game must never take the game down with it. `Launched::stop` exists for a
  caller that wants it, and stopping twice is not an error.
- **The failure message** names the cause and the remedy. `PermissionDenied`
  says to let `gbs` start the game.
