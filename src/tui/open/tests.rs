use super::super::tests::{TODAY, app_of, history_of, lines, press, told};
use super::super::*;

fn set(app: App, keys: &[&str]) -> App {
    let programs = keys.iter().map(|key| (key.to_string(), PathBuf::from(format!("/bin/open-{key}"))));

    App {
        programs: programs.collect(),
        ..app
    }
}

#[test]
fn gx_leaves_the_program_set_for_a_key_of_the_task_to_open_with_its_value_projects_and_contexts() {
    let line = "(A) review +rust-todo @review ticket:open/01 +dots @mac +dots";
    let mut app = set(app_of(&[line]), &["ticket"]);

    assert!(!press(&mut app, "gx"));
    let opening = Opening {
        program: PathBuf::from("/bin/open-ticket"),
        value: "open/01".to_string(),
        projects: "rust-todo dots".to_string(),
        contexts: "review mac".to_string(),
    };
    assert_eq!(app.open, Some(opening));
    assert_eq!((lines(&app), told(&app)), (vec![line.to_string()], None));
}

#[test]
fn gx_opens_the_first_key_value_of_the_line_whose_key_is_set_skipping_an_empty_value_and_a_double_slash() {
    let mut app = set(app_of(&["fix url:https://x ticket: pr://x/1 pr:7 ticket:a/1"]), &["ticket", "pr"]);

    press(&mut app, "gx");
    let opened = app.open.map(|opening| (opening.program, opening.value));
    assert_eq!(opened, Some((PathBuf::from("/bin/open-pr"), "7".to_string())));
}

#[test]
fn a_program_that_could_not_be_opened_is_named_with_the_system_error_and_the_list_stays_usable() {
    let mut app = set(app_of(&["one ticket:a/1", "two"]), &["ticket"]);

    press(&mut app, "gx");
    let opening = app.open.take().unwrap();
    app.opened(&opening, Err(io::Error::from_raw_os_error(libc::ENOENT)));
    let said = "could not open /bin/open-ticket: No such file or directory (os error 2)";
    assert_eq!(told(&app), Some(Notice::Refused(said.to_string())));
    assert!(app.refusal_shown());

    assert!(press(&mut app, "x"));
    assert_eq!(lines(&app), vec!["two".to_string()]);
}

#[test]
fn a_program_opened_says_nothing() {
    let mut app = set(app_of(&["one ticket:a/1"]), &["ticket"]);

    press(&mut app, "gx");
    let opening = app.open.take().unwrap();
    app.opened(&opening, Ok(()));
    assert_eq!(told(&app), None);
}

#[test]
fn gx_on_a_task_with_no_key_value_set_opens_nothing_and_says_so_and_x_alone_still_completes() {
    let mut app = set(app_of(&["one url:x ticket: ticket://x/1", "two"]), &["ticket"]);

    assert!(!press(&mut app, "gx"));
    assert_eq!((app.open.clone(), told(&app), lines(&app).len()), (None, Some(Notice::NothingToOpen), 2));
    assert!(app.refusal_shown());

    assert!(press(&mut app, "x"));
    assert_eq!(lines(&app), vec!["two".to_string()]);
}

#[test]
fn gx_with_an_empty_open_table_opens_nothing_and_says_so() {
    let mut app = set(app_of(&["one ticket:a/1"]), &[]);

    press(&mut app, "gx");
    assert_eq!((app.open.clone(), told(&app)), (None, Some(Notice::NothingToOpen)));
}

#[test]
fn gx_on_a_folded_group_or_an_empty_list_opens_nothing_and_says_so() {
    let mut folded = set(app_of(&["(A) one ticket:a"]), &["ticket"]);
    press(&mut folded, "zMgx");
    assert_eq!((folded.open.clone(), told(&folded)), (None, Some(Notice::NoTaskUnderCursor)));
    assert!(folded.refusal_shown());

    let mut empty = set(app_of(&[]), &["ticket"]);
    press(&mut empty, "gx");
    assert_eq!((empty.open.clone(), told(&empty)), (None, Some(Notice::NoTaskUnderCursor)));
}

#[test]
fn gx_in_the_history_leaves_the_program_set_for_a_key_of_the_done_task_to_open_and_writes_nothing() {
    let mut app = set(history_of(&["one"], &["x 2026-09-25 old ticket:open/01 +rust-todo"]), &["ticket"]);

    assert!(!press(&mut app, "Hgx"));
    let opened = app.open.take().map(|opening| (opening.program, opening.value, opening.projects));
    let program = PathBuf::from("/bin/open-ticket");
    assert_eq!(opened, Some((program, "open/01".to_string(), "rust-todo".to_string())));
    assert_eq!((told(&app), app.history.len(), lines(&app)), (None, 1, vec!["one".to_string()]));
}

#[test]
fn gx_leaves_the_program_to_open_when_the_task_file_could_not_be_read() {
    let mut app = set(app_of(&["one ticket:a/1"]), &["ticket"]);
    let mut text = "one ticket:a/1\n".to_string();

    for c in ['g', 'x'] {
        let key = Some(Event::Key(KeyEvent::from(KeyCode::Char(c))));
        assert!(!app.turn(Err(io::ErrorKind::PermissionDenied.into()), &mut text, key, TODAY));
    }

    assert_eq!(app.open.take().map(|opening| opening.value), Some("a/1".to_string()));
    assert_eq!((told(&app), lines(&app)), (None, vec!["one ticket:a/1".to_string()]));
}
