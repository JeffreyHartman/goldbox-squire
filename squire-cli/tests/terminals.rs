//! The terminal table: the compiled-in defaults, and the user's file over them.

use squire_cli::terminals::{self, Terminal};

fn named<'a>(list: &'a [Terminal], name: &str) -> &'a Terminal {
    list.iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("no entry for {name}"))
}

#[test]
fn every_compiled_in_terminal_can_name_a_window_and_ask_for_cells() {
    let built_in = terminals::built_in();

    assert!(!built_in.is_empty());
    for t in &built_in {
        assert!(!t.name.is_empty());
        assert!(
            t.app_id.iter().any(|a| a.contains("{id}")),
            "{} cannot name its window",
            t.name
        );
        assert!(
            t.size.iter().any(|a| a.contains("{cols}"))
                && t.size.iter().any(|a| a.contains("{rows}")),
            "{} cannot be sized in cells",
            t.name
        );
    }
}

#[test]
fn the_command_line_carries_the_name_the_size_and_the_command() {
    let list = terminals::built_in();
    let kitty = named(&list, "kitty");

    let argv = kitty.command_line(110, 50, &["gbs".into(), "--hud".into()]);

    assert_eq!(argv[0], "kitty");
    assert!(
        argv.contains(&"--app-id=goldbox-squire".to_string()),
        "{argv:?}"
    );
    assert!(
        argv.contains(&"initial_window_width=110c".to_string()),
        "{argv:?}"
    );
    assert!(
        argv.contains(&"initial_window_height=50c".to_string()),
        "{argv:?}"
    );
    assert_eq!(
        &argv[argv.len() - 2..],
        &["gbs".to_string(), "--hud".to_string()]
    );
}

#[test]
fn the_command_is_last_even_when_the_terminal_needs_a_flag_before_it() {
    let list = terminals::built_in();
    let alacritty = named(&list, "alacritty");

    let argv = alacritty.command_line(80, 24, &["gbs".into()]);

    let e = argv
        .iter()
        .position(|a| a == "-e")
        .expect("alacritty needs -e");
    assert_eq!(argv[e + 1], "gbs");
    assert_eq!(e + 2, argv.len(), "nothing may follow the command");
}

#[test]
fn a_user_entry_replaces_the_compiled_in_one_of_the_same_name() {
    let user = r#"
        [[terminal]]
        name = "kitty"
        app_id = ["--app-id={id}"]
        size = ["--mine={cols}x{rows}"]
        exec = []
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "terminals.toml");

    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(named(&list, "kitty").size, vec!["--mine={cols}x{rows}"]);
    assert_eq!(
        list.iter().filter(|t| t.name == "kitty").count(),
        1,
        "the user's entry replaces, it does not add a second"
    );
    // Everything else is still there.
    assert!(list.iter().any(|t| t.name == "foot"));
}

#[test]
fn a_terminal_squire_never_heard_of_is_added_by_the_user_file() {
    let user = r#"
        [[terminal]]
        name = "some-terminal-from-2031"
        app_id = ["--name={id}"]
        size = ["--cells={cols},{rows}"]
        exec = ["--run"]
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "terminals.toml");

    assert!(problems.is_empty(), "{problems:?}");
    let new = named(&list, "some-terminal-from-2031");
    assert_eq!(
        new.command_line(100, 40, &["gbs".into()]),
        vec![
            "some-terminal-from-2031",
            "--name=goldbox-squire",
            "--cells=100,40",
            "--run",
            "gbs"
        ]
    );
}

#[test]
fn a_file_that_is_not_toml_names_itself_and_leaves_the_defaults_standing() {
    let (list, problems) = terminals::merge(terminals::built_in(), "not toml at all", "mine.toml");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("mine.toml"), "{}", problems[0]);
    assert_eq!(list.len(), terminals::built_in().len());
}

#[test]
fn an_entry_with_no_name_is_named_by_its_position_and_skipped() {
    let user = r#"
        [[terminal]]
        name = ""
        app_id = []
        size = []
        exec = []

        [[terminal]]
        name = "foot"
        app_id = ["--app-id={id}"]
        size = ["--window-size-chars={cols}x{rows}"]
        exec = []
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("mine.toml"), "{}", problems[0]);
    assert!(
        problems[0].contains('1'),
        "the entry is named: {}",
        problems[0]
    );
    // The good entry in the same file still took effect.
    assert!(list.iter().any(|t| t.name == "foot"));
}

#[test]
fn a_placeholder_squire_does_not_know_is_a_complaint_and_not_a_crash() {
    let user = r#"
        [[terminal]]
        name = "foot"
        app_id = ["--app-id={identifier}"]
        size = ["--window-size-chars={cols}x{rows}"]
        exec = []
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("{identifier}"), "{}", problems[0]);
    assert!(problems[0].contains("foot"), "{}", problems[0]);
    // The entry is refused, so the compiled-in foot is what runs.
    assert_eq!(named(&list, "foot").app_id, vec!["--app-id={id}"]);
}

#[test]
fn a_terminal_that_is_not_in_the_table_is_not_an_error() {
    let list = terminals::built_in();

    assert!(terminals::find(&list, "konsole").is_none());
    assert!(terminals::find(&list, "kitty").is_some());
}

#[test]
fn the_terminal_is_recognised_by_its_program_name_and_not_its_path() {
    let list = terminals::built_in();

    assert!(terminals::find(&list, "/usr/bin/kitty").is_some());
}

#[test]
fn one_bad_entry_does_not_take_the_good_ones_in_the_same_file_with_it() {
    let user = r#"
        [[terminal]]
        name = "broken"
        size = "not a list"

        [[terminal]]
        name = "some-terminal-from-2031"
        app_id = ["--name={id}"]
        size = ["--cells={cols},{rows}"]
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("mine.toml"), "{}", problems[0]);
    assert!(
        problems[0].contains('1'),
        "the entry is named: {}",
        problems[0]
    );
    assert!(
        list.iter().any(|t| t.name == "some-terminal-from-2031"),
        "the good entry after it still took effect"
    );
    assert!(!list.iter().any(|t| t.name == "broken"));
}

#[test]
fn an_entry_that_leaves_out_a_field_gets_an_empty_one_rather_than_a_refusal() {
    // A terminal that needs no flag before the command writes no `exec`.
    let user = r#"
        [[terminal]]
        name = "plain-terminal"
        app_id = ["--name={id}"]
        size = ["--cells={cols}x{rows}"]
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");

    assert!(problems.is_empty(), "{problems:?}");
    let t = named(&list, "plain-terminal");
    assert!(t.exec.is_empty());
    assert_eq!(
        t.command_line(80, 24, &["gbs".into()]),
        vec![
            "plain-terminal",
            "--name=goldbox-squire",
            "--cells=80x24",
            "gbs"
        ]
    );
}

#[test]
fn the_app_id_is_the_one_name_a_compositor_rule_matches() {
    // A user writes their KWin or Hyprland rule once, by hand, against this
    // string. Changing it breaks every rule already written, silently, so the
    // name is pinned here rather than left to whoever calls `command_line`.
    assert_eq!(terminals::APP_ID, "goldbox-squire");
}

#[test]
fn every_window_squire_opens_carries_the_app_id() {
    for t in terminals::built_in() {
        let argv = t.command_line(80, 24, &["gbs".into()]);
        assert!(
            argv.iter().any(|a| a.contains(terminals::APP_ID)),
            "{} opens an unnamed window: {argv:?}",
            t.name
        );
    }
}

#[test]
fn a_terminal_that_cannot_name_its_window_is_still_spawned() {
    // The user's own terminal may have no way to set a name. That costs them
    // the compositor rule, and nothing else: the HUD still opens.
    let user = r#"
        [[terminal]]
        name = "nameless-terminal"
        size = ["--cells={cols}x{rows}"]
    "#;

    let (list, problems) = terminals::merge(terminals::built_in(), user, "mine.toml");

    assert!(problems.is_empty(), "{problems:?}");
    let t = named(&list, "nameless-terminal");
    assert_eq!(
        t.command_line(80, 24, &["gbs".into()]),
        vec!["nameless-terminal", "--cells=80x24", "gbs"]
    );
}
