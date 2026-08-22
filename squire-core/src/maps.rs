//! Parses `/proc/<pid>/maps` into the regions a scanner can search.

/// One mapped region of a process's address space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub start: usize,
    pub end: usize,
    pub readable: bool,
    pub writable: bool,
    /// `true` when the mapping is shared rather than private.
    pub shared: bool,
    /// The backing file, when the region has one.
    pub path: Option<String>,
}

impl Region {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Parses the whole contents of `/proc/<pid>/maps`.
///
/// A line that does not parse is skipped rather than fatal. The kernel can
/// change this file while we read it, and one odd line must not stop a scan.
pub fn parse(text: &str) -> Vec<Region> {
    text.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Region> {
    let mut fields = line.split_whitespace();
    let range = fields.next()?;
    let perms = fields.next()?;

    let (start, end) = range.split_once('-')?;
    let start = usize::from_str_radix(start, 16).ok()?;
    let end = usize::from_str_radix(end, 16).ok()?;
    if end < start {
        return None;
    }

    let perms = perms.as_bytes();
    if perms.len() < 4 {
        return None;
    }

    // Fields 3, 4 and 5 are the offset, the device and the inode. The path is
    // everything after them, and it can contain spaces.
    let path = line
        .split_whitespace()
        .nth(5)
        .map(|_| {
            let mut rest = line.split_whitespace();
            for _ in 0..5 {
                rest.next();
            }
            rest.collect::<Vec<_>>().join(" ")
        })
        .filter(|p| !p.is_empty());

    Some(Region {
        start,
        end,
        readable: perms[0] == b'r',
        writable: perms[1] == b'w',
        shared: perms[3] == b's',
        path,
    })
}
