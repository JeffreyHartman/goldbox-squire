# 002 — Reading another process's memory on Linux from Rust

Research note for ticket `002-research-linux-process-memory`.
Date: 2026-08-22.
Scope: Linux only. Read-only access. DOSBox native and DOSBox under Wine.

All measurements in this note come from this machine. The test targets were a
Windows DOSBox 0.74-3 under Wine in the prefix `/home/jeff/goldbox/wine`, and a
native `dosbox` (a dosbox-staging build) that ran Pool of Radiance.

---

## Recommendations

1. **Use `process_vm_readv(2)` as the primary read path. Keep `/proc/<pid>/mem`
   as a fallback.** `process_vm_readv` is about 3x faster on large scans. It is
   also faster for scattered reads when the tool batches many ranges into one
   call. The two calls need the same permission, so the fallback helps only in
   rare cases.
2. **Batch the scattered reads.** One `process_vm_readv` call that carries 128
   remote ranges costs 0.47 us per record. One call per record costs 1.37 us.
   A `pread` on `/proc/<pid>/mem` costs 0.93 us.
3. **Read `/proc/<pid>/mem` with `pread`, not `lseek` plus `read`.** In Rust,
   use `std::os::unix::fs::FileExt::read_at`. This removes a syscall and makes
   the handle safe to share.
4. **Read regions in chunks of 1 MiB, and continue after an error.** A single
   large read that starts on an inaccessible page fails for the whole request.
   This is not a rare case. Stack regions carry a guard page at the start.
5. **Skip these regions in a scan:** file-backed regions, regions without the
   `r` permission, regions larger than 512 MiB, and the `[vvar]`, `[vdso]`,
   and `[vsyscall]` pseudo-regions. The guest RAM block is always an anonymous
   private read/write region.
6. **Do not assume that the region start is the guest RAM base.** The base sits
   at a non-zero offset inside a larger heap region. This note gives the
   measured offsets.
7. **Take `nix` (features `uio` and `process`) and `libc`. Reject the rest.**
   Write the `/proc` parsing by hand. It is about 40 lines. `procfs`,
   `proc-maps`, `read-process-memory`, and `sysinfo` each add a dependency for
   little gain.
8. **Do not require a change to `kernel.yama.ptrace_scope`.** Detect the
   failure and print a clear message. `EPERM` from `process_vm_readv` and
   `EACCES` from `open` on `/proc/<pid>/mem` both mean the same thing.
   Offer the user three named remedies, in the order given in section 3.
9. **Identify the Wine DOSBox process through `/proc/<pid>/maps`, not through
   `/proc/<pid>/exe`.** Under Wine, `/proc/<pid>/exe` points at
   `wine-preloader`. The `maps` file holds the real host path of the mapped
   `DOSBox.exe`, and that path also names the Wine prefix.

---

## 1. `process_vm_readv(2)` compared with `/proc/<pid>/mem`

### 1.1 What each call is

`process_vm_readv(2)` moves data directly between two address spaces. The
kernel does not copy the data through kernel space. The signature is:

```c
ssize_t process_vm_readv(pid_t pid,
                         const struct iovec *local_iov, unsigned long liovcnt,
                         const struct iovec *remote_iov, unsigned long riovcnt,
                         unsigned long flags);
```

`flags` must be 0. `liovcnt` and `riovcnt` must not exceed `IOV_MAX`.
[man 2 process_vm_readv]

`/proc/<pid>/mem` is a file. The tool opens it, then reads at an offset that
equals the virtual address in the target. [man 5 proc_pid_mem]

### 1.2 Measured performance

The benchmark read the emulated RAM of a live DOSBox. Record size 285 bytes,
which is the character record size in this project. The scan size was 16 MiB in
1 MiB chunks.

Wine DOSBox 0.74-3, guest RAM at `0x04a00020`:

| Operation | Time | Per read |
|---|---|---|
| 20000 small reads, `process_vm_readv`, one range per call | 27.5 ms | 1.37 us |
| 20000 small reads, `pread` on `/proc/<pid>/mem` | 18.6 ms | 0.93 us |
| 20000 small reads, `process_vm_readv`, 128 ranges per call | 9.3 ms | 0.47 us |
| 16 MiB scan, `process_vm_readv` | 1.4 ms | 11540 MB/s |
| 16 MiB scan, `pread` | 4.1 ms | 3907 MB/s |

Native dosbox-staging, guest RAM at `0x7f508bd68010`:

| Operation | Time | Per read |
|---|---|---|
| 20000 small reads, `process_vm_readv`, one range per call | 22.7 ms | 1.14 us |
| 20000 small reads, `pread` on `/proc/<pid>/mem` | 20.9 ms | 1.05 us |
| 20000 small reads, `process_vm_readv`, 128 ranges per call | 10.4 ms | 0.52 us |
| 16 MiB scan, `process_vm_readv` | 1.5 ms | 10924 MB/s |
| 16 MiB scan, `pread` | 4.3 ms | 3764 MB/s |

Result: `process_vm_readv` wins the large scan by about 3x. For single small
reads, `pread` wins by a small margin. When the tool batches the small reads
into one vectored call, `process_vm_readv` wins by about 2x. Both methods are
fast enough for a full 800 MB address-space scan in about one second.

### 1.3 Partial reads

`process_vm_readv` returns the number of bytes that it read. That number can be
less than the requested total. The kernel checks the remote ranges only just
before each transfer. A partial read happens when a remote range covers an
invalid area. The kernel then stops and does not process any later range.
Partial transfers apply at the granularity of one `iovec` element. The kernel
never splits a single element. [man 2 process_vm_readv]

Measured: a `process_vm_readv` of 64 bytes that started 8 bytes before the end
of a mapped region returned 8, with `errno` unchanged. The call reported a
partial read, not an error.

The man page gives a direct rule for the tool. When a read crosses a page
boundary into an area that can be invalid, split the remote side into two
`iovec` elements at the page boundary, and merge them into one local element.

`read(2)` on `/proc/<pid>/mem` can also return a short count. The scanner must
loop until it reads the full amount, or until it gets 0 or an error.

### 1.4 Error codes

From `man 2 process_vm_readv`:

| errno | Cause |
|---|---|
| `EFAULT` | The remote or local memory is outside the accessible address space |
| `EINVAL` | `flags` is not 0, a length sum overflows `ssize_t`, or a count is too large |
| `ENOMEM` | The kernel cannot allocate the internal `iovec` copies |
| `EPERM` | The caller has no permission to access the address space of `pid` |
| `ESRCH` | No process with that PID exists |

Measured behavior of the two methods on regions that a scanner meets:

| Target | `process_vm_readv` | read on `/proc/<pid>/mem` |
|---|---|---|
| `PROT_NONE` region (`---p` in maps) | `-1`, `EFAULT` | Succeeds. Returns zero bytes of data |
| Unmapped address | `-1`, `EFAULT` | `-1`, `EIO` |
| First page of a `MAP_GROWSDOWN` stack region | `-1`, `EFAULT` | `-1`, `EIO` |
| Read that starts inside a region and runs past its end | Partial count, no error | Short count |

The `PROT_NONE` result is important. `/proc/<pid>/mem` reads a `PROT_NONE`
region and returns zeros. The kernel uses `FOLL_FORCE`, so the protection bits
do not block the read. `process_vm_readv` refuses the same region with
`EFAULT`. A scanner that uses `/proc/<pid>/mem` therefore wastes time on large
reserved regions unless it filters them out from the `maps` file first. See
section 2.

### 1.5 Does either method need the target to stop?

No. Neither method stops the target. `process_vm_readv` does not attach to the
target. The man page states that the data transfers are not atomic in any way.
[man 2 process_vm_readv, NOTES]

`/proc/<pid>/mem` is a plain file read and also does not stop the target.

This matters for correctness, not for permission. The tool reads a live,
changing address space. A 285-byte record can tear if DOSBox writes it during
the read. For a read-only v1 that displays party state, a torn read is a small
risk. Ticket 006 covers the validation invariants that catch it.

### 1.6 Permission difference

The two methods take different ptrace access mode checks:

| Path | Check |
|---|---|
| `process_vm_readv` | `PTRACE_MODE_ATTACH_REALCREDS` |
| `/proc/<pid>/mem` | `PTRACE_MODE_ATTACH_FSCREDS` |
| `/proc/<pid>/maps` | `PTRACE_MODE_READ_FSCREDS` |
| `/proc/<pid>/exe`, `cwd`, `environ` | `PTRACE_MODE_READ_FSCREDS` |

[man 5 proc_pid_mem, man 5 proc_pid_maps, man 5 proc_pid_exe, man 2 ptrace]

`REALCREDS` uses the real UID, GID, and the permitted capability set.
`FSCREDS` uses the filesystem UID, GID, and the effective capability set.
[man 2 ptrace, "Ptrace access mode checking"]

For a normal desktop tool that a user starts from a shell, the real and
effective IDs match. The difference has no practical effect. It matters only
for a setuid or file-capability binary. See section 3.4.

Both memory paths take an `ATTACH` check. Yama gates `ATTACH`. So Yama blocks
both. The `/proc/<pid>/mem` fallback does not defeat Yama.

### 1.7 Conclusion for this tool

Use `process_vm_readv` as the primary path. It gives:

- 3x the throughput on the anchor scan.
- Vectored reads. One call can gather many scattered 285-byte records.
- A hard `EFAULT` on `PROT_NONE` and unmapped memory. This is a correct signal,
  not a silent block of zeros.

Keep an optional `/proc/<pid>/mem` path. It costs about 20 lines. It helps on a
kernel older than Linux 3.2, where `process_vm_readv` returns `ENOSYS`.
[man 2 process_vm_readv, HISTORY: Linux 3.2, glibc 2.15]

---

## 2. Parsing `/proc/<pid>/maps`

### 2.1 Field meanings

The format is:

```
address           perms offset   dev   inode      pathname
00400000-00452000 r-xp  00000000 08:02 173521     /usr/bin/dbus-daemon
```

| Field | Meaning |
|---|---|
| `address` | Start and end virtual address, hexadecimal, no `0x` prefix. End is exclusive |
| `perms` | `r` read, `w` write, `x` execute, and `s` shared or `p` private copy-on-write |
| `offset` | Offset into the backing file |
| `dev` | Device, major:minor |
| `inode` | Inode on that device. `0` means no file backs the region |
| `pathname` | The backing file, a pseudo-path, or blank for an anonymous mapping |

Pseudo-paths include `[heap]`, `[stack]`, `[vdso]`, `[vvar]`, and
`[anon:name]`. A deleted backing file gets ` (deleted)` appended. The kernel
escapes a newline in a path as `\012`. [man 5 proc_pid_maps]

Parsing notes for the Rust code:

- Split on whitespace with a limit of 6 fields. The path can contain spaces.
- The path field can be absent. Handle a 5-field line.
- Addresses are `u64` even on a 32-bit target process.

### 2.2 Which region holds the emulated guest RAM

DOSBox allocates the emulated RAM on the heap. dosbox-staging uses
`memory.pages.resize(num_pages)` on a `std::vector<page_t>`, then sets
`MemBase = &(memory.pages[0].bytes[0])`. That is a plain heap allocation, not
`mmap`. [dosbox-staging `src/hardware/memory.cpp`]

Empirical result, Wine DOSBox 0.74-3, PID 835979:

```
04a00000-06030000 rwxp 00000000 00:00 0
```

- Region size 23265280 bytes, 22.19 MiB.
- Anonymous. No path. Inode 0.
- Guest RAM base measured at `0x04a00020`, that is offset `0x20` into the
  region.

Verification at that base: the interrupt vector table reads
`60 10 00 f0 08 00 70 00 ...`, which is `F000:1060` for INT 0 and `0070:0008`
for INT 1 to INT 3. The BIOS data area at guest `0x413` reads 640, the
conventional memory size in KB. Guest `0x449` reads `0x03`, the video mode.
Guest `0x44a` reads 80, the column count. The string
`COMSPEC=Z:\COMMAND.COM` sits at guest offset `0x12b9`.

Empirical result, native dosbox-staging, PID 837974:

```
7f508bbe5000-7f508cd69000 rw-p 00000000 00:00 0
```

- Region size 18366464 bytes, 17.52 MiB.
- Anonymous. No path. Inode 0.
- Guest RAM base measured at `0x7f508bd68010`, that is offset `0x183010` into
  the region.

Verification at that base: guest `0x413` reads 640, guest `0x449` reads `0x0d`
(the EGA 320x200 mode that Pool of Radiance uses), and `COMSPEC=Z:` sits at
guest offset `0x12b0`.

Facts that the scanner design must respect:

1. The guest RAM block is always an **anonymous, private, read/write** region.
   Wine adds the `x` bit, so the perms read `rwxp` there and `rw-p` natively.
   Test for `r` and `w`, not for the exact string.
2. The block does **not** start at the region start. The heap allocator merges
   the block with other allocations into one VMA. The measured offsets were
   `0x20` and `0x183010`.
3. The region is **larger** than the RAM block. 22.19 MiB and 17.52 MiB held a
   16 MiB block.
4. The **address range differs by build**. Wine's 32-bit DOSBox sits at
   `0x04a00000`, low in the 32-bit space. Native 64-bit dosbox-staging sits at
   `0x7f50...`, high in the 64-bit space. Do not hardcode a range.
5. The emulated **VGA memory is a separate allocation**. A read at guest
   `0xB8000` returned zeros in both targets, while the video mode byte was
   valid. DOSBox holds the VGA planes outside `MemBase`. Do not use the text
   screen as an anchor.

### 2.3 Which regions to skip

| Skip | Reason |
|---|---|
| Any region without `r` in `perms` | `process_vm_readv` returns `EFAULT`. `/proc/<pid>/mem` returns zeros and wastes time |
| File-backed regions, that is inode not 0 | Code and read-only data. The guest RAM is anonymous |
| Regions larger than 512 MiB | Reserved address space. Measured example: one `06730000-67c00000 ---p` region of 1.63 GB in the Wine target |
| `[vvar]`, `[vdso]`, `[vsyscall]` | Kernel pseudo-mappings. Small, and a read can fail |
| Shared regions, that is `s` in `perms` | Graphics and audio buffers. Measured: 98 `rw-s` regions in the Wine target |

Measured region counts in the Wine DOSBox target: 1174 lines in `maps`. Of
those, 60 were `---p` (PROT_NONE). After the filters above, the scan read
803.9 MB and found the guest RAM in a few seconds.

Keep the file-backed `DOSBox.exe` mapping in the process-identity check, but
skip it in the memory scan. Ticket 001 covers whether the anchor also appears
in the executable image.

### 2.4 The stack guard page trap

Measured: several anonymous `rw-p` regions returned `EIO` on a read at the
region start, and read correctly one page later. `smaps` showed `VmFlags: ...
gu ...` for those regions. The `gu` flag marks `MAP_GROWSDOWN`. The kernel
keeps the first page of such a region inaccessible as a stack guard, even
though `maps` reports `rw-p`.

Consequence: a single `read` that covers the whole region fails with `EIO`,
because the first page fails. The scanner must read in chunks and continue
past a failed chunk. A 1 MiB chunk size worked and cost nothing measurable.

---

## 3. `kernel.yama.ptrace_scope`

### 3.1 The four values

Yama is a Linux Security Module. The kernel needs `CONFIG_SECURITY_YAMA`.
The control file is `/proc/sys/kernel/yama/ptrace_scope`, available since
Linux 3.4. [man 2 ptrace]

| Value | Name | Effect |
|---|---|---|
| 0 | classic ptrace permissions | No extra restriction beyond the commoncap and other LSM checks |
| 1 | restricted ptrace (**default**) | The caller must hold `CAP_SYS_PTRACE` in the target's user namespace, or the target must be a descendant of the caller, or the target must name the caller through `prctl(PR_SET_PTRACER, ...)` |
| 2 | admin-only attach | Only a caller with `CAP_SYS_PTRACE` in the target's user namespace can attach |
| 3 | no attach | No process can attach. **Once written, this value cannot change** |

[man 2 ptrace, `/proc/sys/kernel/yama/ptrace_scope`. Also the kernel file
`Documentation/admin-guide/LSM/Yama.rst`]

The kernel source confirms the gate. `yama_ptrace_access_check()` runs its
switch only inside `if (mode & PTRACE_MODE_ATTACH)`, and every denial path
sets `rc = -EPERM`. [`security/yama/yama_lsm.c`]

### 3.2 Which read method each value gates

Yama gates only `PTRACE_MODE_ATTACH`. From the table in section 1.6:

| Path | Access mode | Yama gates it? |
|---|---|---|
| `process_vm_readv` | `PTRACE_MODE_ATTACH_REALCREDS` | **Yes** |
| `/proc/<pid>/mem` | `PTRACE_MODE_ATTACH_FSCREDS` | **Yes** |
| `/proc/<pid>/maps` | `PTRACE_MODE_READ_FSCREDS` | No |
| `/proc/<pid>/exe`, `cwd`, `environ`, `cmdline`, `comm` | `PTRACE_MODE_READ_FSCREDS` or none | No |

This gives a clean design property. **Under `ptrace_scope=1`, the tool can
still discover the process, read its `maps`, read its `cmdline`, and read its
`environ`. Only the memory read fails.** The tool can therefore print a precise
message that names the process it found and the exact reason for the failure.

Note the value on this machine. It resets to 1 on every reboot. During this
research it was 0.

### 3.3 The exact errno

Both denials pass through `mm_access()` in `kernel/fork.c`, which discards the
specific LSM error and returns `ERR_PTR(-EACCES)`:

```c
} else if (!may_access_mm(mm, task, mode)) {
        mmput(mm);
        mm = ERR_PTR(-EACCES);
}
```

`process_vm_rw()` in `mm/process_vm_access.c` then remaps that value:

```c
mm = mm_access(task, PTRACE_MODE_ATTACH_REALCREDS);
if (IS_ERR(mm)) {
        rc = PTR_ERR(mm);
        /*
         * Explicitly map EACCES to EPERM as EPERM is a more
         * appropriate error code for process_vw_readv/writev
         */
        if (rc == -EACCES)
                rc = -EPERM;
        goto put_task_struct;
}
```

So:

| Call | errno on denial | Value |
|---|---|---|
| `process_vm_readv` | `EPERM` | 1 |
| `open("/proc/<pid>/mem", O_RDONLY)` | `EACCES` | 13 |

Measured against PID 1, which runs as root while the test ran as UID 1000:

```
process_vm_readv(pid=1): ret=-1 errno=1 Operation not permitted
open /proc/1/mem: errno=13 Permission denied
```

The errno does not tell the tool **why** the check failed. `EPERM` covers a
Yama denial, a UID mismatch, and a non-dumpable target alike. The tool must
read `/proc/sys/kernel/yama/ptrace_scope` and compare UIDs itself, then build
the message. A suggested check order:

1. Does `/proc/<pid>` exist? If not, the process exited.
2. Does the target UID in `/proc/<pid>/status` match our UID? If not, say so.
3. Read `/proc/sys/kernel/yama/ptrace_scope`. If the value is not 0, name it
   and give the remedies below.
4. Otherwise report the raw errno.

Note that `open` on `/proc/<pid>/mem` succeeds even when a later `read` fails.
The kernel repeats the access check at read time. Do not treat a successful
`open` as proof of access.

### 3.4 Alternatives to lowering `ptrace_scope`

| Option | How | Security trade-off |
|---|---|---|
| **A. Start DOSBox as a child of the tool** | Yama scope 1 allows attach to a descendant. The tool forks and execs DOSBox, then reads it | **Best.** No privilege change at all. No system setting change. The cost is a change to the user's launch habit. The tool must own the launch |
| **B. `prctl(PR_SET_PTRACER, pid)` in the target** | The target declares which PID can attach | Not usable. It needs a code change inside DOSBox. A `LD_PRELOAD` shim can do it for native DOSBox, but not for a Windows DOSBox under Wine |
| **C. `sudo sysctl kernel.yama.ptrace_scope=0`** | Lowers the setting for the whole machine, until reboot | **Worst.** Any process of the user can then read the memory of any other process of the user. That includes an SSH agent, a GPG agent, a password manager, and a browser. This is the exact escalation that Yama exists to stop |
| **D. `sudo setcap cap_sys_ptrace+ep ./squire`** | Gives the binary the capability. Yama scope 1 and 2 both allow it | **Medium.** The capability applies only to this binary, not to the whole machine. But `CAP_SYS_PTRACE` lets the binary read **any** process, including root processes. Anyone who can overwrite the binary gains that power. Rebuilding with `cargo build` clears the capability every time. A file-capability binary also runs with `AT_SECURE`, which strips `LD_LIBRARY_PATH` and marks the process non-dumpable |
| **E. Run the tool as root** | `sudo squire` | Poor. Wider than option D, and it makes any output file root-owned |
| **F. Same-UID rule alone** | Not sufficient | Same UID satisfies step 3 of the ptrace access check, but Yama scope 1 still denies a non-descendant. Same UID is necessary, not sufficient |

Recommendation: build option A into the tool as a `squire launch` mode, and
document option D as the fallback for a DOSBox that already runs. Never make
option C the documented default. The tool must not run `sysctl` on its own.

Hypothesis, not yet tested: option A works for the Wine case as well, because
the Wine `DOSBox.exe` process is a descendant of the `wine` command that the
tool starts. Wine reparents its processes to the `wineserver` (measured PPID
843 for every Wine process in the prefix). `task_is_descendant()` walks the
real parent chain, so the reparenting can break the descendant rule. This needs
a test at `ptrace_scope=1`.

---

## 4. Rust crates

Version and maintenance data below come from crates.io and GitHub, read on
2026-08-22.

### 4.1 `libc` — **take it**

- Version: `0.2.189` stable. `1.0.0-alpha.4` published 2026-07-21.
- Maintained: yes. `rust-lang/libc` pushed 2026-08-20.
- License: MIT OR Apache-2.0.
- Relevant API:

```rust
pub unsafe extern "C" fn process_vm_readv(
    pid: pid_t,
    local_iov: *const iovec, liovcnt: c_ulong,
    remote_iov: *const iovec, riovcnt: c_ulong,
    flags: c_ulong,
) -> isize
```

  The binding lives in `unix/linux_like/`, which glibc, musl, Android, and
  L4Re share. No Cargo feature gates it on `x86_64-unknown-linux-gnu`.
- What it adds: the extern declaration and the C types. Nothing else.
- Verdict: take it. It arrives through `nix` anyway, and the tool needs
  `libc::pid_t` and the errno constants for the error messages.

### 4.2 `nix` — **take it**

- Version: `0.31.3`, published 2026-05-11.
- Maintained: yes. `nix-rust/nix` pushed 2026-05-19.
- License: MIT.
- Relevant API, behind Cargo features `uio` and `process`:

```rust
pub fn process_vm_readv(
    pid: Pid,
    local_iov: &mut [IoSliceMut<'_>],
    remote_iov: &[RemoteIoVec],
) -> Result<usize>

#[repr(C)]
pub struct RemoteIoVec { pub base: usize, pub len: usize }
```

- What it adds over `libc`: a safe wrapper, a typed `Pid`, an errno mapped into
  `nix::Result`, and `IoSliceMut` plus `RemoteIoVec` in place of raw `iovec`
  pointers. The slice signature matches the batched-read plan in
  recommendation 2 exactly.
- Windows: none. `nix` is Unix only. The Windows port will use
  `ReadProcessMemory` behind the same internal trait.
- Verdict: take it. Enable only `features = ["uio", "process"]` with
  `default-features = false`. This removes the unsafe FFI block from the tool
  and gives the exact vectored API the design needs.

### 4.3 `procfs` — **reject**

- Version: `0.18.0`, released 2025-08-30. Maintained. MIT OR Apache-2.0.
- Relevant API: `Process::maps() -> ProcResult<MemoryMaps>` with a typed
  `MemoryMap { address: (u64,u64), perms: MMPermissions, offset, dev, inode,
  pathname: MMapPath, extension }`. Also `Process::smaps()` and
  `Process::mem() -> ProcResult<File>`.
- What it adds: a typed `maps` and `smaps` parser. It does **not** wrap
  `process_vm_readv`. `Process::mem()` returns a plain `File`.
- Verdict: reject. The crate covers the whole of `/proc`. The tool needs one
  line format. A hand-written parser is about 40 lines and gives exact control
  over the skip rules in section 2.3. `MMapPath` also loses the raw path
  string that section 5 needs. Linux only, so it does not help the later
  Windows port.

### 4.4 `proc-maps` — **reject, but reconsider for Windows**

- Version: `0.5.0`, created 2026-06-29. `rbspy/proc-maps` pushed 2026-07-12.
  Low activity, 0 open issues.
- License: crates.io reports `NOASSERTION`. The repository carries an MIT
  license file. The `Cargo.toml` field is not machine-readable. **Check the
  license text before you take this crate.**
- Relevant API: `get_process_maps(pid: Pid) -> Result<Vec<MapRange>>`.
- What it adds: one maps API across Linux, macOS, and Windows.
- Verdict: reject for v1. The Linux backend does what 40 lines of hand-written
  code do. Reconsider at the Windows port, where the cross-platform API earns
  its keep. Note the license question then.

### 4.5 `read-process-memory` — **reject for v1**

- Version: `0.2.0`, created 2026-07-19. `rbspy/read-process-memory` pushed
  2026-07-19. MIT.
- Relevant API: a `CopyAddress` trait with
  `copy_address(&self, addr: usize, buf: &mut [u8]) -> io::Result<()>`, plus a
  `ProcessHandle` type.
- On Linux it tries `process_vm_readv` first, then falls back to
  `/proc/<pid>/mem` when the syscall returns `ENOSYS` or `EPERM`. That is
  exactly the dual strategy in section 1.7.
- Windows and macOS backends exist. The crate depends on `windows-sys` and
  `mach2`.
- Verdict: reject for v1, on one technical ground. `copy_address` reads **one
  contiguous range per call**. It cannot express the batched vectored read that
  measured 3x faster than the one-call-per-record path. The fallback logic that
  the crate provides is about 20 lines. Revisit at the Windows port, where the
  cross-platform trait has real value and the record count per frame is small.

### 4.6 `sysinfo` — **reject**

- Version: `0.39.6`, published 2026-07-09. Very active. MIT.
- Relevant API: `Process::memory()` and `Process::virtual_memory()` return
  usage figures only. The crate has **no API that reads the bytes of another
  process**.
- Verdict: reject. It cannot do the job. It can list processes, but
  `std::fs::read_dir("/proc")` does that in 20 lines and does not pull in a
  large cross-platform crate. `sysinfo` also refreshes a full system snapshot,
  which costs far more than the tool needs.

### 4.7 Other candidates found

| Crate | Notes | Verdict |
|---|---|---|
| `process_vm_io` `1.0.14` (2025-08-09) | `ProcessVirtualMemoryIO` implements `std::io::Read`, `Write`, and `Seek` over `process_vm_readv`. MIT. Linux only | Reject. The `Read` interface hides the vectored call that the tool needs |
| `remoteprocess` `0.5.3` (2026-07-24) | The engine behind `py-spy`. Wraps `read-process-memory` and `proc-maps`, adds ptrace suspend and stack unwinding. Cross-platform. MIT | Reject. Far more machinery than a read-only scanner needs. It does confirm that `read-process-memory` plus `proc-maps` is the standard ecosystem pair |
| `process-memory`, `process-reader`, `procmaps`, `libprocmem`, `proc_mem` | Small user bases. Not verified beyond their crates.io descriptions | Reject |

### 4.8 Recommended dependency set

```toml
[dependencies]
nix = { version = "0.31", default-features = false, features = ["uio", "process"] }
libc = "0.2"
```

Everything else in this ticket is `std::fs` and `std::os::unix::fs::FileExt`.

---

## 5. Process discovery

### 5.1 The three `/proc` files

| File | Contents | Access check | Limit |
|---|---|---|---|
| `/proc/<pid>/comm` | The `comm` value. Usually the executable basename | None beyond normal file permissions | **Truncated to 15 characters.** `TASK_COMM_LEN` is 16, including the terminating null |
| `/proc/<pid>/cmdline` | The argument vector, null-separated, with a trailing null | None | The process can rewrite it. The man page calls it "the command line that the process wants you to see" |
| `/proc/<pid>/exe` | A symbolic link to the executable path. ` (deleted)` appended if unlinked | `PTRACE_MODE_READ_FSCREDS` | Unreadable if the main thread already exited |

[man 5 proc_pid_comm, man 5 proc_pid_cmdline, man 5 proc_pid_exe]

The 15-character limit on `comm` is the trap. `dosbox-staging` is 14
characters and survives. A longer name does not.

In Rust, enumerate `/proc` with `std::fs::read_dir`, keep the entries whose
name parses as a `u32`, then read the three files. Ignore an `ENOENT`. A
process can exit between the `read_dir` and the read.

### 5.2 Native DOSBox, measured

PID 837974, started as `dosbox -conf por.conf`:

```
/proc/837974/comm     -> "dosbox\n"
/proc/837974/cmdline  -> "dosbox\0-conf\0por.conf\0"
/proc/837974/exe      -> /usr/bin/dosbox
```

All three work. Match on `exe` first, because a user can rename the binary but
the link still resolves. Accept a basename of `dosbox`, `dosbox-staging`,
`dosbox-x`, or `DOSBox`.

### 5.3 Wine DOSBox, measured

PID 835979, started as `wine DOSBox.exe` with
`WINEPREFIX=/home/jeff/goldbox/wine`:

```
/proc/835979/comm     -> "DOSBox.exe\n"
/proc/835979/cmdline  -> "C:\DOSBox\DOSBox.exe\0" then about 75 empty strings
/proc/835979/exe      -> /usr/lib/wine/x86_64-unix/wine-preloader
/proc/835979/cwd      -> /home/jeff/goldbox/wine/drive_c/DOSBox
```

Three facts follow.

1. **`comm` holds the Windows executable basename.** Wine sets it. `DOSBox.exe`
   is 10 characters, so the 15-character limit does not bite.
2. **`cmdline` holds the Windows path, `C:\DOSBox\DOSBox.exe`.** It carries no
   host path and no prefix. Wine also pads the vector with many empty strings.
   A naive split on the null byte yields about 76 arguments. Strip the trailing
   empty entries.
3. **`exe` is useless for identity.** It points at `wine-preloader`, which is
   the same for every Wine process in the system.

The full Wine process set in the prefix, measured:

```
835876  start.exe /exec
835879  /usr/lib/wine/../../bin/wineserver
835885  C:\windows\system32\services.exe
835901  C:\windows\system32\winedevice.exe
835910  C:\windows\system32\explorer.exe /desktop
835912  C:\windows\system32\plugplay.exe
835918  C:\windows\system32\svchost.exe -k LocalServiceNetworkRestricted
835924  C:\windows\system32\winedevice.exe
835953  C:\windows\system32\rpcss.exe
835979  C:\DOSBox\DOSBox.exe        <- the target
835981  C:\windows\system32\conhost.exe --server 0xac
```

Every one of them has `exe -> wine-preloader`.

### 5.4 The reliable Wine identification method

**Read `/proc/<pid>/maps` and look for a file-backed mapping whose basename is
`DOSBox.exe`.** Wine maps the PE image from its real host path, and `maps`
shows that host path.

Measured for PID 835979:

```
00400000-00401000 r-xp 00000000 00:20 20269558  .../drive_c/DOSBox/DOSBox.exe
00710000-0074d000 r-xp 0030d000 00:20 20269558  .../drive_c/DOSBox/DOSBox.exe
02422000-02423000 r-xp 0034a000 00:20 20269558  .../drive_c/DOSBox/DOSBox.exe
68100000-68101000 r-xp 00000000 00:20 20269562  .../drive_c/DOSBox/SDL.dll
```

The paths above are shortened for width. The real path in the file is
`/home/jeff/goldbox/wine/drive_c/DOSBox/DOSBox.exe`. Note that the image
appears on more than one line. Deduplicate by inode.

Measured for PID 835910, the `explorer.exe` in the same prefix:

```
/usr/lib/wine/x86_64-windows/explorer.exe
```

This method gives four things at once:

1. It proves the process runs DOSBox, whatever `comm` or `cmdline` say.
2. It gives the **host path** of `DOSBox.exe`.
3. The parent directory names the **Wine prefix**. Take the path, strip the
   trailing `/drive_c/...` component, and the remainder is `WINEPREFIX`. This
   separates two DOSBox instances in two different prefixes.
4. It works under `ptrace_scope=1`, because `maps` takes only a
   `PTRACE_MODE_READ` check. See section 3.2.

Two supporting signals, both weaker:

- **`/proc/<pid>/environ`** held `WINEPREFIX=/home/jeff/goldbox/wine` in the
  test. It is not reliable. The man page states that the file holds the
  **initial** environment from `execve(2)` only. A user who does not export
  `WINEPREFIX` leaves it absent, and Wine then defaults to `~/.wine`.
- **`/proc/<pid>/cwd`** pointed at `/home/jeff/goldbox/wine/drive_c/DOSBox`.
  This is useful confirmation, but DOSBox can change its own directory.

### 5.5 Recommended matching algorithm

```
for each numeric entry in /proc:
    read comm
    if comm is one of: dosbox, dosbox-staging, dosbox-x, DOSBox.exe, DOSBOX.EXE
    or readlink(exe) basename is one of: dosbox, dosbox-staging, dosbox-x
    then:
        read maps
        find a file-backed mapping whose basename matches (?i)^dosbox([-.].*)?$
        if the mapped path ends in .exe:
            kind = Wine
            image = that path
            prefix = the path up to and including the component before drive_c
        else:
            kind = Native
            image = readlink(exe)
        record the candidate
```

Present every candidate to the user when more than one matches. Do not guess.
Two DOSBox instances in two prefixes are a real case for this project.

---

## Sources

Manual pages, from `man-pages 6.18` on this machine:

- `man 2 process_vm_readv` — signature, partial-read semantics, error list,
  `IOV_MAX` limit, the `PTRACE_MODE_ATTACH_REALCREDS` check, the atomicity
  note, and the Linux 3.2 / glibc 2.15 history.
- `man 2 ptrace` — the "Ptrace access mode checking" algorithm, the
  `PTRACE_MODE_*` constants, and the `/proc/sys/kernel/yama/ptrace_scope`
  section that defines values 0 to 3.
- `man 5 proc_pid_maps` — the field format and the pseudo-paths.
- `man 5 proc_pid_mem` — the `PTRACE_MODE_ATTACH_FSCREDS` check.
- `man 5 proc_pid_comm` — the `TASK_COMM_LEN` truncation at 16 bytes.
- `man 5 proc_pid_cmdline` — the null-separated format and the warning that the
  process controls the contents.
- `man 5 proc_pid_exe` — the `PTRACE_MODE_READ_FSCREDS` check.
- `man 5 proc_pid_environ` — the note that the file holds the initial
  environment only.

Kernel documentation and source:

- `https://www.kernel.org/doc/html/latest/admin-guide/LSM/Yama.html` — the
  four `ptrace_scope` values, `PR_SET_PTRACER`, and the descendant rule.
- `security/yama/yama_lsm.c`, `yama_ptrace_access_check()` — the
  `PTRACE_MODE_ATTACH` gate and the `-EPERM` returns.
- `kernel/fork.c`, `mm_access()` — the `ERR_PTR(-EACCES)` return on denial.
- `mm/process_vm_access.c`, `process_vm_rw()` — the explicit `EACCES` to
  `EPERM` remap.

Crate documentation:

- `https://docs.rs/libc/latest/libc/fn.process_vm_readv.html`
- `https://docs.rs/nix/latest/nix/sys/uio/fn.process_vm_readv.html`
- `https://docs.rs/nix/latest/nix/sys/uio/struct.RemoteIoVec.html`
- `https://docs.rs/procfs/latest/procfs/process/struct.Process.html`
- `https://docs.rs/procfs/latest/procfs/process/struct.MemoryMap.html`
- `https://docs.rs/proc-maps/latest/proc_maps/`
- `https://docs.rs/read-process-memory/latest/read_process_memory/`
- `https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html`
- `https://docs.rs/process_vm_io/latest/process_vm_io/`
- `https://docs.rs/remoteprocess/latest/remoteprocess/`
- `https://crates.io/api/v1/crates/{libc,nix,procfs,proc-maps,read-process-memory,sysinfo}`
  for versions, dates, and licenses.

DOSBox source:

- `dosbox-staging`, `src/hardware/memory.cpp` — `memory.pages.resize(num_pages)`
  and `MemBase = &(memory.pages[0].bytes[0])`, which prove the heap allocation.

Local measurements:

- Wine DOSBox 0.74-3, PID 835979, prefix `/home/jeff/goldbox/wine`.
- Native dosbox-staging, PID 837974, running Pool of Radiance.
- Benchmark source kept in the session scratchpad as `bench.c`.
