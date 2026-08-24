//! Finding the system emulator on PATH by its common names (ticket 024).

use std::path::{Path, PathBuf};

use squire_cli::emulator;

#[test]
fn each_common_name_is_found_when_present() {
    for name in ["dosbox", "dosbox-staging", "dosbox-x"] {
        let dir = bin_dir(&format!("each-{name}"), &[name]);
        let path = std::env::join_paths([&dir]).unwrap();

        assert_eq!(emulator::find_on_path(&path), Some(name), "for {name}");
    }
}

#[test]
fn dosbox_is_preferred_when_several_are_present() {
    // Even when the longer names sit earlier on PATH.
    let staging = bin_dir("pref-staging", &["dosbox-staging"]);
    let plain = bin_dir("pref-plain", &["dosbox"]);
    let path = std::env::join_paths([&staging, &plain]).unwrap();

    assert_eq!(emulator::find_on_path(&path), Some("dosbox"));
}

#[test]
fn nothing_on_path_is_none() {
    let dir = bin_dir("empty", &[]);
    let path = std::env::join_paths([&dir]).unwrap();

    assert_eq!(emulator::find_on_path(&path), None);
}

#[test]
fn a_file_without_the_executable_bit_does_not_count() {
    let dir = bin_dir("noexec", &[]);
    std::fs::write(dir.join("dosbox"), "").unwrap();

    let path = std::env::join_paths([&dir]).unwrap();

    assert_eq!(emulator::find_on_path(&path), None);
}

// --- helpers -----------------------------------------------------------------

fn bin_dir(tag: &str, names: &[&str]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gbs-emulator-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for name in names {
        executable(&dir.join(name));
    }
    dir
}

fn executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

// --- precedence (ticket 027) ---------------------------------------------------

#[test]
fn the_argument_beats_the_config_beats_the_path_search() {
    assert_eq!(
        emulator::command(Some("arg"), Some("cfg"), Some("dosbox")).unwrap(),
        "arg"
    );
    assert_eq!(
        emulator::command(None, Some("cfg"), Some("dosbox")).unwrap(),
        "cfg"
    );
    assert_eq!(
        emulator::command(None, None, Some("dosbox")).unwrap(),
        "dosbox"
    );
}

#[test]
fn nothing_anywhere_is_an_error_naming_the_names_and_the_flag() {
    let err = emulator::command(None, None, None).unwrap_err();

    for expected in ["dosbox", "dosbox-staging", "dosbox-x", "--dosbox"] {
        assert!(err.contains(expected), "missing {expected} in: {err}");
    }
}
