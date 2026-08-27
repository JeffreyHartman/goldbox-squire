//! Opening a view in a window of its own: which terminal, and what to say to
//! it. No window is opened here. Everything up to the spawn is a decision, and
//! the decisions are what can be wrong.

use std::path::Path;

use squire_cli::layout::Size;
use squire_cli::spawn;
use squire_cli::terminals::{self, ViewKind};

fn size(cols: u16, rows: u16) -> Size {
    Size { cols, rows }
}

fn nothing_installed(_: &str) -> bool {
    false
}

#[test]
fn the_terminal_the_user_named_wins() {
    let list = terminals::built_in();

    let chosen = spawn::choose(&list, Some("wezterm"), Some("kitty"), &|_| true);

    assert_eq!(chosen.as_deref(), Some("wezterm"));
}

#[test]
fn a_terminal_squire_does_not_know_is_still_the_users_choice() {
    // Naming one is the whole answer to "which terminal". Refusing it because
    // the table has no entry would be the trap that arguing about the size is
    // not.
    let list = terminals::built_in();

    let chosen = spawn::choose(&list, Some("st"), None, &nothing_installed);

    assert_eq!(chosen.as_deref(), Some("st"));
}

#[test]
fn the_environment_answers_when_the_user_did_not() {
    let list = terminals::built_in();

    let chosen = spawn::choose(&list, None, Some("foot"), &nothing_installed);

    assert_eq!(chosen.as_deref(), Some("foot"));
}

#[test]
fn otherwise_the_first_terminal_squire_knows_that_is_installed() {
    let list = terminals::built_in();

    let chosen = spawn::choose(&list, None, None, &|name| name == "kitty");

    assert_eq!(chosen.as_deref(), Some("kitty"));
}

#[test]
fn a_machine_with_none_of_them_chooses_nothing_rather_than_guessing() {
    let list = terminals::built_in();

    assert_eq!(spawn::choose(&list, None, None, &nothing_installed), None);
}

#[test]
fn an_empty_terminal_variable_is_not_an_answer() {
    let list = terminals::built_in();

    let chosen = spawn::choose(&list, None, Some(""), &|name| name == "kitty");

    assert_eq!(chosen.as_deref(), Some("kitty"));
}

#[test]
fn a_known_terminal_is_asked_for_the_name_the_size_and_the_view() {
    let list = terminals::built_in();
    let command = spawn::view_command(
        Path::new("/usr/bin/gbs"),
        ViewKind::Hud,
        Path::new("/run/user/1000/goldbox-squire/77.sock"),
    );

    let (argv, problem) = spawn::plan(&list, "kitty", ViewKind::Hud, size(110, 50), &command);

    assert_eq!(problem, None);
    assert!(
        argv.contains(&"--app-id=goldbox-squire-hud".to_string()),
        "{argv:?}"
    );
    assert!(
        argv.contains(&"initial_window_width=110c".to_string()),
        "{argv:?}"
    );
    assert_eq!(
        &argv[argv.len() - 5..],
        &[
            "/usr/bin/gbs".to_string(),
            "--view".to_string(),
            "hud".to_string(),
            "--socket".to_string(),
            "/run/user/1000/goldbox-squire/77.sock".to_string(),
        ]
    );
}

#[test]
fn an_unknown_terminal_is_still_opened_and_costs_one_sentence() {
    let list = terminals::built_in();
    let command = spawn::view_command(Path::new("gbs"), ViewKind::Hud, Path::new("/x.sock"));

    let (argv, problem) = spawn::plan(&list, "st", ViewKind::Hud, size(110, 50), &command);

    assert_eq!(argv[0], "st");
    assert_eq!(&argv[1..], &command[..], "the view is still started");
    let problem = problem.expect("an unknown terminal is worth saying once");
    assert!(problem.contains("st"), "the message names it: {problem}");
    assert!(
        problem.contains("terminals.toml"),
        "the message says how to fix it: {problem}"
    );
}

#[test]
fn a_user_entry_is_honoured_over_the_compiled_in_one() {
    // The user's own terminals.toml is what makes a terminal Squire has never
    // heard of a first-class one, with no rebuild.
    let user = r#"
        [[terminal]]
        name = "kitty"
        app_id = ["--title={id}"]
        size = ["--geometry={cols}x{rows}"]
    "#;
    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");
    assert!(problems.is_empty(), "{problems:?}");

    let (argv, problem) = spawn::plan(&list, "kitty", ViewKind::Hud, size(80, 24), &["gbs".into()]);

    assert_eq!(problem, None);
    assert_eq!(
        argv,
        vec![
            "kitty",
            "--title=goldbox-squire-hud",
            "--geometry=80x24",
            "gbs"
        ]
    );
}

#[test]
fn the_terminal_may_be_named_by_a_path() {
    // A TERMINAL variable often holds one, and the entry is found by the file
    // name at the end of it.
    let list = terminals::built_in();

    let (argv, problem) = spawn::plan(
        &list,
        "/usr/local/bin/kitty",
        ViewKind::Hud,
        size(80, 24),
        &["gbs".into()],
    );

    assert_eq!(problem, None);
    assert_eq!(argv[0], "kitty", "{argv:?}");
}
