# 002 — Research: reading another process's memory on Linux from Rust

Type: `wayfinder:research` (AFK)
Status: CLOSED 2026-08-22
Blocked by: none

## Question

Establish the facts the core design depends on:

- `process_vm_readv` compared with reading `/proc/<pid>/mem`. Which is right
  here, and why.
- Parsing `/proc/<pid>/maps`. Which regions hold a DOSBox emulated memory block,
  and which must be skipped.
- How `kernel.yama.ptrace_scope` gates each method, and what error surfaces when
  it blocks a read.
- Which Rust crates are worth using, and which add a dependency for no gain.
- How a process is discovered and matched by name, including a process running
  under Wine, where the command line reads `C:\DOSBox\DOSBox.exe`.

## Answer

Full write-up: [`.scratch/squire-v1/research/002-linux-process-memory.md`](../research/002-linux-process-memory.md)

Gist. Every number below was measured against a live DOSBox, not quoted.

- **Read method.** Use `process_vm_readv(2)`. Keep `/proc/<pid>/mem` as a thin
  fallback. A 16 MiB scan runs at 11500 MB/s against 3900 MB/s for `pread`. For
  scattered 285-byte records, 128 remote ranges batched into one call cost
  0.47 us each, against 1.37 us for one call per record. Neither method stops the
  target process.
- **Scanning.** Read in 1 MiB chunks and continue past an error. One large read
  that starts on an inaccessible page fails as a whole. Skip regions that are
  not readable, file-backed, shared, or larger than 512 MiB. `/proc/<pid>/mem`
  returns zeros for a `PROT_NONE` region. `process_vm_readv` returns `EFAULT`,
  which is the honest answer.
- **Guest RAM found in both builds.** Wine DOSBox 0.74-3 puts it at
  `0x04a00020`. Native dosbox-staging puts it at `0x7f508bd68010`. Both sit at a
  non-zero offset inside a larger anonymous heap region. A scanner must never
  assume the region start is the base.
- **Yama.** It gates only `PTRACE_MODE_ATTACH`. At scope 1 the tool still reads
  `maps`, `cmdline`, `environ`, and `exe`. Only the memory read fails.
  `process_vm_readv` returns `EPERM`. Opening `/proc/<pid>/mem` returns `EACCES`.
- **Crates.** Take `nix` (features `uio` and `process`) and `libc`. Nothing else.
  Reject `procfs`, `proc-maps`, `read-process-memory`, `sysinfo`,
  `process_vm_io`, and `remoteprocess`. Hand-write the maps parser, about 40
  lines.
- **Wine discovery.** `comm` reads `DOSBox.exe`. `cmdline` reads the Windows
  path plus padding. `exe` points at `wine-preloader` and is useless. The
  reliable identifier is `/proc/<pid>/maps`, which holds the real host path of
  the executable and therefore also names the Wine prefix.
