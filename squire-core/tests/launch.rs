//! Goldbox Squire starts the emulator itself. Yama's descendant rule then
//! permits the memory read with no privilege change, and without asking the
//! user to weaken their machine.

use std::time::{Duration, Instant};

use squire_core::launch::Emulator;
use squire_core::mem::Reader;

/// A stand-in for DOSBox: a process that stays alive until it is stopped.
fn sleeper() -> Emulator {
    Emulator::new("sleep").arg("30")
}

#[test]
fn starts_the_emulator_and_reports_its_pid() {
    let mut running = sleeper().start().unwrap();

    assert!(running.pid() > 0);
    assert!(running.is_running());

    running.stop().unwrap();
}

#[test]
fn the_started_process_is_a_child_of_this_one() {
    // This is what makes the read permitted under Yama scope 1.
    let mut running = sleeper().start().unwrap();

    let stat = std::fs::read_to_string(format!("/proc/{}/stat", running.pid())).unwrap();
    // The parent pid is the fourth field, and the second field can hold spaces,
    // so the fields are counted after the closing bracket of the command name.
    let after_comm = stat.rsplit_once(')').unwrap().1;
    let ppid: i32 = after_comm
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();

    assert_eq!(ppid, std::process::id() as i32, "the emulator is our child");

    running.stop().unwrap();
}

#[test]
fn the_memory_of_the_started_process_can_be_read() {
    let mut running = sleeper().start().unwrap();

    // `spawn` returns as soon as the fork happens, so the child can still be
    // between fork and exec. Its address space is not the program's yet. Wait
    // for it rather than reading whatever is there at this instant.
    let reader = running.reader();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut regions = reader.regions().unwrap();
    while regions.len() <= 3 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
        regions = reader.regions().unwrap();
    }

    assert!(
        regions.len() > 3,
        "the child still had only {} regions after five seconds",
        regions.len()
    );
    running.stop().unwrap();
}

#[test]
fn stopping_ends_the_process() {
    let mut running = sleeper().start().unwrap();
    let pid = running.pid();

    running.stop().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("process {pid} was still running five seconds after stop");
}

#[test]
fn notices_when_the_process_ends_on_its_own() {
    // The user quits DOSBox. The tool must notice rather than read stale bytes.
    let mut running = Emulator::new("true").start().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !running.is_running() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the tool did not notice that the process ended");
}

#[test]
fn a_command_that_does_not_exist_is_a_clear_error() {
    let result = Emulator::new("no-such-emulator-anywhere").start();

    let err = result.unwrap_err().to_string();

    assert!(
        err.contains("no-such-emulator-anywhere"),
        "the error names the command, got: {err}"
    );
}

#[test]
fn stopping_twice_is_not_an_error() {
    let mut running = sleeper().start().unwrap();

    running.stop().unwrap();
    running.stop().unwrap();
}

#[test]
fn dropping_the_handle_does_not_kill_the_emulator() {
    // A read-only tool must never take the game down with it. If `gbs` exits
    // or panics, the player keeps playing.
    let running = sleeper().start().unwrap();
    let pid = running.pid();

    drop(running);

    std::thread::sleep(Duration::from_millis(200));
    assert!(
        std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "the emulator was killed by the handle going out of scope"
    );

    // Clean up, since nothing else will.
    unsafe { libc::kill(pid, libc::SIGKILL) };
}

#[test]
fn passes_the_configuration_file_through_as_an_argument() {
    let e = Emulator::new("dosbox").arg("-conf").arg("/tmp/por.conf");

    assert_eq!(e.args(), &["-conf", "/tmp/por.conf"]);
}
