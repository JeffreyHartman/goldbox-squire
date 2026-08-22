//! Reads the memory of another process.
//!
//! `process_vm_readv(2)` is the primary path. It is roughly three times faster
//! than seeking `/proc/<pid>/mem` for a large scan, it does not stop the target
//! process, and it reports an unmapped page honestly with `EFAULT` where
//! `/proc/<pid>/mem` quietly returns zeroes.

use std::fs;
use std::io::{IoSliceMut, Read, Seek, SeekFrom};

use nix::sys::uio::{process_vm_readv, RemoteIoVec};
use nix::unistd::Pid;

use crate::maps::{self, Region};
use crate::Error;

/// Regions larger than this are reservations rather than real memory. DOSBox
/// allocates 16 to 32 MiB for the emulated machine, so nothing we want is
/// bigger, and scanning a multi-gigabyte reservation wastes the whole scan.
pub const MAX_REGION_LEN: usize = 512 * 1024 * 1024;

/// Reads bytes out of somewhere. A trait so that a test can supply memory
/// without a live emulator.
pub trait Reader {
    /// Fills `buf` from `addr`. Returns how many bytes were read.
    fn read(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error>;

    /// The regions of the target's address space.
    fn regions(&self) -> Result<Vec<Region>, Error>;
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
    fn read_via_proc_mem(&self, addr: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let mut f = fs::File::open(format!("/proc/{}/mem", self.pid))
            .map_err(|e| Error::from_io_reading(self.pid, e))?;
        f.seek(SeekFrom::Start(addr as u64))
            .map_err(|e| Error::from_io_reading(self.pid, e))?;
        f.read_exact(buf)
            .map_err(|e| Error::from_io_reading(self.pid, e))?;
        Ok(buf.len())
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
            Ok(n) => Ok(n),
            Err(nix::errno::Errno::ENOSYS) => self.read_via_proc_mem(addr, buf),
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
