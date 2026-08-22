//! These tests read real process memory. Most read this test process's own
//! memory, which the kernel always permits, so they need no special setting.

use squire_core::mem::{ProcessReader, Reader};

fn myself() -> ProcessReader {
    ProcessReader::new(std::process::id() as i32)
}

#[test]
fn reads_a_known_buffer_out_of_this_process() {
    let source: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let addr = source.as_ptr() as usize;
    let mut dest = vec![0u8; 1024];

    let n = myself().read(addr, &mut dest).unwrap();

    assert_eq!(n, 1024);
    assert_eq!(dest, source);
}

#[test]
fn reads_a_slice_from_the_middle_of_a_buffer() {
    let source: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let addr = source.as_ptr() as usize + 100;
    let mut dest = vec![0u8; 8];

    myself().read(addr, &mut dest).unwrap();

    assert_eq!(dest, source[100..108]);
}

#[test]
fn reading_an_unmapped_address_is_an_error_and_not_a_crash() {
    let mut dest = vec![0u8; 16];

    // An address the kernel never maps for a user process.
    let result = myself().read(0x10, &mut dest);

    assert!(result.is_err(), "expected an error, got {result:?}");
}

#[test]
fn reading_from_a_process_that_does_not_exist_is_an_error() {
    // A pid far above the usual maximum, which no process holds.
    let reader = ProcessReader::new(0x7FFF_FFFF);
    let mut dest = vec![0u8; 16];

    assert!(reader.read(0x1000, &mut dest).is_err());
}

#[test]
fn lists_the_regions_of_this_process() {
    let regions = myself().regions().unwrap();

    assert!(regions.len() > 5);
    assert!(regions.iter().all(|r| r.end > r.start));
}

#[test]
fn a_buffer_on_the_heap_falls_inside_a_listed_region() {
    let source = vec![0xABu8; 4096];
    let addr = source.as_ptr() as usize;

    let regions = myself().regions().unwrap();

    assert!(
        regions.iter().any(|r| r.start <= addr && addr < r.end),
        "the heap buffer at {addr:#x} is inside no listed region"
    );
}

#[test]
fn searchable_regions_exclude_the_ones_a_scan_must_skip() {
    let regions = myself().regions().unwrap();
    let searchable = squire_core::mem::searchable(&regions);

    assert!(!searchable.is_empty());
    for r in &searchable {
        assert!(r.readable, "an unreadable region cannot be searched");
        assert!(
            !r.shared,
            "a shared mapping is not the emulator's own memory"
        );
        assert!(
            r.len() <= squire_core::mem::MAX_REGION_LEN,
            "a very large reservation is not worth scanning"
        );
    }
}

#[test]
fn a_buffer_on_the_heap_is_found_by_scanning_the_searchable_regions() {
    // This is the whole read path end to end, against real memory: list the
    // regions, keep the ones worth searching, read each one, and find a known
    // pattern. Only the process differs when the target is DOSBox.
    let needle: Vec<u8> = (0u32..64)
        .map(|i| (i.wrapping_mul(37) & 0xFF) as u8)
        .collect();
    let held = needle.clone();
    let addr = held.as_ptr() as usize;

    let reader = myself();
    let regions = reader.regions().unwrap();
    let mut found_at = None;
    for r in squire_core::mem::searchable(&regions) {
        let mut buf = vec![0u8; r.len()];
        if reader.read(r.start, &mut buf).is_err() {
            continue;
        }
        if let Some(pos) = buf
            .windows(needle.len())
            .position(|w| w == needle.as_slice())
        {
            found_at = Some(r.start + pos);
            break;
        }
    }

    assert_eq!(
        found_at,
        Some(addr),
        "the known pattern was found where it lives"
    );
}

#[test]
fn a_read_that_returns_less_than_asked_for_is_an_error_not_a_short_buffer() {
    // A read that crosses out of a mapped region comes back short. Treating a
    // short read as a full one is how a wrong number reaches the user quietly,
    // so the reader refuses rather than reporting partial bytes.
    let regions = myself().regions().unwrap();
    let last = regions.iter().max_by_key(|r| r.end).unwrap();

    // Start inside the final region and ask for far more than remains.
    let addr = last.end - 64;
    let mut dest = vec![0u8; 8192];
    let result = myself().read(addr, &mut dest);

    assert!(result.is_err(), "expected an error, got {result:?}");
}

#[test]
fn reads_a_region_in_chunks_and_keeps_going_past_a_bad_page() {
    // Some regions look readable but hold an inaccessible first page, which is
    // normal for a stack. Reading the whole region at once loses all of it.
    // Chunked reading keeps what is readable.
    let reader = myself();
    let regions = reader.regions().unwrap();
    let searchable = squire_core::mem::searchable(&regions);
    assert!(!searchable.is_empty());

    let total: usize = searchable.iter().map(|r| reader.read_block(r).len()).sum();

    assert!(total > 0, "some memory was read");
}
