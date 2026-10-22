use super::super::tests::{app, app_of, lines, moved, press, redo};

#[test]
fn u_takes_the_completed_task_back_from_done_txt_and_ctrl_r_sends_it_again() {
    let mut app = app_of(&["one", "(A) 2026-09-01 two", "three"]);

    press(&mut app, "x");
    assert_eq!(lines(&app), ["one", "three"]);
    press(&mut app, "u");
    assert_eq!(lines(&app), ["one", "(A) 2026-09-01 two", "three"]);
    assert!(redo(&mut app));
    assert_eq!(lines(&app), ["one", "three"]);

    let line = "x 2026-09-26 2026-09-01 two";
    assert_eq!(moved(&app), [format!("+{line}"), format!("-{line}"), format!("+{line}")]);
}

#[test]
fn undoing_another_change_asks_nothing_of_done_txt_even_with_a_move_still_waiting() {
    let mut app = app();

    press(&mut app, "xddu");

    assert_eq!(lines(&app), ["two", "three"]);
    assert_eq!(moved(&app), ["+x 2026-09-26 one"]);
}

#[test]
fn u_brings_back_a_completed_task_with_its_priority() {
    let mut app = app_of(&["(A) one", "two"]);

    press(&mut app, "x");
    assert!(press(&mut app, "u"));

    assert_eq!(lines(&app), ["(A) one", "two"]);
    assert_eq!(app.message.as_deref(), Some("undone"));
}

#[test]
fn u_steps_back_through_every_change_and_ctrl_r_forward_again() {
    let mut app = app();

    press(&mut app, "ddx");
    assert_eq!(lines(&app), ["three"]);
    press(&mut app, "uu");
    assert_eq!(lines(&app), ["one", "two", "three"]);

    assert!(redo(&mut app));
    assert_eq!(lines(&app), ["two", "three"]);
    assert_eq!(app.message.as_deref(), Some("redone"));
}

#[test]
fn a_change_after_u_leaves_nothing_to_redo() {
    let mut app = app();

    press(&mut app, "xux");

    assert!(!redo(&mut app));
    assert_eq!(app.message.as_deref(), Some("nothing to redo"));
}

#[test]
fn u_and_ctrl_r_with_no_history_write_nothing_and_say_so() {
    let mut app = app();

    assert!(!press(&mut app, "u"));
    assert_eq!(app.message.as_deref(), Some("nothing to undo"));
    assert!(press(&mut app, "x"));
    assert!(press(&mut app, "u"));
    assert!(!press(&mut app, "r"));
    assert_eq!(lines(&app), ["one", "two", "three"]);
    press(&mut app, "x");
    assert!(!redo(&mut app));
    assert_eq!(app.message.as_deref(), Some("nothing to redo"));
}
