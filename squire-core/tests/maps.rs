use squire_core::maps::{self, Region};

/// Four lines taken verbatim from a live DOSBox process on this machine.
const SAMPLE: &str = "\
55d3f7a00000-55d3f7a5c000 r-xp 00002000 08:03 1443212                    /usr/bin/dosbox
7f508bbe5000-7f508cd69000 rw-p 00000000 00:00 0 
7ffd1a3c1000-7ffd1a3e2000 rw-p 00000000 00:00 0                          [stack]
7f508cd69000-7f508cd6a000 ---p 00000000 00:00 0 
";

#[test]
fn reads_the_address_range_of_each_line() {
    let regions = maps::parse(SAMPLE);

    assert_eq!(regions.len(), 4);
    assert_eq!(regions[0].start, 0x55d3f7a00000);
    assert_eq!(regions[0].end, 0x55d3f7a5c000);
    assert_eq!(regions[1].len(), 0x1184000);
}

#[test]
fn reads_the_permission_flags() {
    let regions = maps::parse(SAMPLE);

    assert!(regions[0].readable);
    assert!(!regions[0].writable);
    assert!(regions[1].readable && regions[1].writable);
    assert!(!regions[3].readable, "a ---p region is not readable");
}

#[test]
fn keeps_the_backing_path_and_leaves_anonymous_regions_empty() {
    let regions = maps::parse(SAMPLE);

    assert_eq!(regions[0].path.as_deref(), Some("/usr/bin/dosbox"));
    assert_eq!(regions[1].path, None, "an anonymous region has no path");
    assert_eq!(regions[2].path.as_deref(), Some("[stack]"));
}

#[test]
fn keeps_a_path_that_contains_spaces() {
    let line = "400000-401000 r-xp 00000000 08:03 99 /home/jeff/My Games/dosbox.exe\n";

    let regions = maps::parse(line);

    assert_eq!(
        regions[0].path.as_deref(),
        Some("/home/jeff/My Games/dosbox.exe")
    );
}

#[test]
fn skips_a_line_it_cannot_parse_rather_than_failing() {
    let text = format!("this is not a maps line\n{SAMPLE}");

    let regions = maps::parse(&text);

    assert_eq!(regions.len(), 4, "the four good lines still parse");
}

#[test]
fn parses_the_real_maps_file_of_this_process() {
    let pid = std::process::id();
    let text = std::fs::read_to_string(format!("/proc/{pid}/maps")).unwrap();

    let regions: Vec<Region> = maps::parse(&text);

    assert!(regions.len() > 5, "a live process has many regions");
    assert!(
        regions.iter().all(|r| r.end > r.start),
        "every region has a positive length"
    );
    assert!(
        regions.iter().any(|r| r.readable && r.writable && r.path.is_none()),
        "a live process has an anonymous read-write region, which is where guest RAM lives"
    );
}
