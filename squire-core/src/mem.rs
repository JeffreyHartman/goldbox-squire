//! Reads the memory of another process.
//!
//! `process_vm_readv(2)` is the primary path. It is roughly three times faster
//! than seeking `/proc/<pid>/mem` for a large scan, it does not stop the target
//! process, and it reports an unmapped page honestly with `EFAULT` where
//! `/proc/<pid>/mem` quietly returns zeroes.

use std::fs;
use std::io::IoSliceMut;
use std::os::unix::fs::FileExt;
use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::uio::{process_vm_readv, RemoteIoVec};
use nix::unistd::Pid;

use crate::maps::{self, Region};
use crate::Error;

/// Regions larger than this are reservations rather than real memory. DOSBox
/// allocates 16 to 32 MiB for the emulated machine, so nothing we want is
/// bigger, and scanning a multi-gigabyte reservation wastes the whole scan.
pub const MAX_REGION_LEN: usize = 512 * 1024 * 1024;

/// A one-shot latch. The first call to `first_time` returns `true`. Every call
/// after returns `false`.
///
/// This is how a warning prints once per run and not once per read. A shared
/// static holds the state, so all reads answer to the same latch.
#[derive(Debug, Default)]
pub struct OnceFlag {
    fired: AtomicBool,
}

impl OnceFlag {
    /// A fresh latch that has not fired.
    pub const fn new() -> Self {
        OnceFlag {
            fired: AtomicBool::new(false),
        }
    }

    /// Returns `true` the first time only, then `false` for ever after.
    ///
    /// `swap` sets the flag to `true` and hands back the value it held before.
    /// The first call gets back `false`, so it reports the first time. Every
    /// call after gets back `true`. `Relaxed` is enough, because the only fact
    /// that matters is which single call was first.
    pub fn first_time(&self) -> bool {
        !self.fired.swap(true, Ordering::Relaxed)
    }
}

/// Set the first time a read falls back to the file path, so the warning about
/// the slower path prints once per run.
static FALLBACK_WARNED: OnceFlag = OnceFlag::new();

/// Reads bytes out of somewhere. A trait so that a test can supply memory
/// without a live emulator.
pub trait Reader {
    /// Fills `buf` from `addr`. Returns how many bytes were read.
    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error>;

    /// The regions of the target's address space.
    fn regions(&self) -> Result<Vec<Region>, Error>;

    /// Reads a whole region, in chunks, and reports what was readable.
    ///
    /// A region that looks readable can still begin on an inaccessible page,
    /// which is normal for a stack. One large read of such a region fails as a
    /// whole and loses every readable byte behind it. Reading in chunks keeps
    /// the rest.
    ///
    /// The sweep stops at the first failure after data was found, because a
    /// scanner needs one continuous block. A gap would move every later offset
    /// and turn a found address into a wrong one.
    fn read_block(&self, region: &Region) -> RegionBytes {
        let mut bytes: Vec<u8> = Vec::new();
        let mut start = region.start;
        let mut at = region.start;

        while at < region.end {
            let want = CHUNK_LEN.min(region.end - at);
            let mut chunk = vec![0u8; want];
            match self.read(at, &mut chunk) {
                Ok(_) => {
                    if bytes.is_empty() {
                        start = at;
                    }
                    bytes.extend_from_slice(&chunk);
                    at += want;
                }
                // Nothing readable yet, so step past the bad page and retry.
                Err(_) if bytes.is_empty() => at += want,
                // Data already found, so this is the end of the block.
                Err(_) => break,
            }
        }

        RegionBytes { start, bytes }
    }
}

/// Reads the memory of a live process.
#[derive(Debug, Clone, Copy)]
pub struct ProcessReader {
    pid: i32,
}

impl ProcessReader {
    pub fn new(pid: i32) -> Self {
        ProcessReader { pid }
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// The fallback path, used when `process_vm_readv` is unavailable.
    ///
    /// `read` calls this on its own when the syscall is missing. It is public so
    /// a test can exercise the file path directly, which the syscall path hides
    /// on any kernel that has the syscall.
    ///
    /// `read_at` rather than a seek and a read, so that the file offset is not
    /// shared state and two reads cannot race with each other.
    pub fn read_via_proc_mem(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let f = fs::File::open(format!("/proc/{}/mem", self.pid))
            .map_err(|e| Error::from_io_reading(self.pid, e))?;
        let mut done = 0;
        while done < buf.len() {
            match f.read_at(&mut buf[done..], (addr + done) as u64) {
                Ok(0) => {
                    return Err(Error::Unmapped {
                        pid: self.pid,
                        addr,
                    })
                }
                Ok(n) => done += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(Error::from_io_reading(self.pid, e)),
            }
        }
        Ok(done)
    }

    /// Prints the fallback warning, but only the first time per run.
    ///
    /// The file path is slower, and it does not report every unreadable page as
    /// an error. A silent switch to it would hide those facts, so the reader
    /// says so once, the first time it happens.
    fn warn_fallback_once(&self) {
        if FALLBACK_WARNED.first_time() {
            eprintln!(
                "warning: process_vm_readv is unavailable on this kernel; \
                 falling back to /proc/{}/mem. That path is slower, and it \
                 does not report every unreadable page as an error.",
                self.pid
            );
        }
    }
}

/// How much is read in one call when sweeping a region.
pub const CHUNK_LEN: usize = 1024 * 1024;

/// The readable part of one region, and where it starts.
#[derive(Debug, Clone, Default)]
pub struct RegionBytes {
    /// The address the first byte came from.
    pub start: usize,
    pub bytes: Vec<u8>,
}

impl RegionBytes {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl Reader for ProcessReader {
    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        let len = buf.len();
        let remote = [RemoteIoVec { base: addr, len }];
        let mut local = [IoSliceMut::new(buf)];

        match process_vm_readv(Pid::from_raw(self.pid), &mut local, &remote) {
            // A read that crosses out of a mapped region comes back short.
            // Reporting that as a success leaves the rest of the buffer holding
            // whatever was in it before, which is how a wrong number reaches
            // the user quietly. This tool refuses instead.
            Ok(n) if n == len => Ok(n),
            Ok(_) => Err(Error::Unmapped {
                pid: self.pid,
                addr,
            }),
            Err(nix::errno::Errno::ENOSYS) => {
                self.warn_fallback_once();
                self.read_via_proc_mem(addr, buf)
            }
            Err(e) => Err(Error::from_errno(self.pid, addr, e)),
        }
    }

    fn regions(&self) -> Result<Vec<Region>, Error> {
        let text = fs::read_to_string(format!("/proc/{}/maps", self.pid))
            .map_err(|e| Error::from_io_reading(self.pid, e))?;
        Ok(maps::parse(&text))
    }
}

/// Keeps only the regions worth searching for a character record.
///
/// The emulated machine is an anonymous private allocation on the heap. A
/// file-backed region holds the executable and its libraries. A shared region
/// belongs to something else. Neither holds the guest's RAM.
pub fn searchable(regions: &[Region]) -> Vec<Region> {
    regions
        .iter()
        .filter(|r| r.readable && r.writable)
        .filter(|r| !r.shared)
        .filter(|r| r.path.is_none())
        .filter(|r| !r.is_empty() && r.len() <= MAX_REGION_LEN)
        .cloned()
        .collect()
}
