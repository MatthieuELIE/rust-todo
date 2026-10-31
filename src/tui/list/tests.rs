use super::super::tests::{TODAY, app, app_of, history_of, key, lines, moved, popup, press, said, shown, told};
use super::super::*;

#[test]
fn x_completes_the_selected_task_and_the_next_one_takes_its_row() {
    let mut app = app();

    assert!(press(&mut app, "jx"));

    assert_eq!(lines(&app), ["one", "three"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(moved(&app), ["+x 2026-09-26 two"]);
}

#[test]
fn x_drops_the_wait_of_the_completed_task() {
    let mut app = app_of(&["Mail wait:designer +work"]);

    press(&mut app, "x");

    assert_eq!(moved(&app), ["+x 2026-09-26 Mail +work"]);
}

#[test]
fn enter_on_an_empty_list_opens_nothing() {
    let mut app = app_of(&[]);

    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn h_swaps_the_list_for_done_txt_from_its_last_line_without_groups_and_h_swaps_back() {
    let mut app = history_of(&["(A) one", "x 2026-09-20 left"], &["x (A) old", "x 2026-09-25 new", "kept as written"]);
    assert_eq!(shown(&app), ["(A) one"]);

    assert!(!press(&mut app, "H"));
    assert_eq!(shown(&app), ["kept as written", "x 2026-09-25 new", "x old"]);
    assert_eq!(app.tasks().iter().map(|(number, _)| *number).collect::<Vec<_>>(), [3, 2, 1]);
    assert!(app.groups().is_empty());
    assert_eq!(app.selected_task().map(Todo::to_line).as_deref(), Some("kept as written"));
    assert!(!press(&mut app, "zMzRzaj"));
    assert_eq!((app.cursor, told(&app)), (1, None));

    assert!(!press(&mut app, "H"));
    assert_eq!(shown(&app), ["(A) one"]);
    assert_eq!(app.cursor, 0);
}

#[test]
fn h_under_due_or_waiting_switches_to_all_tasks_and_stays_there_on_the_way_back() {
    for filter in [DUE_NOW, WAITING] {
        let mut app = history_of(&["one due:2026-09-01 wait:bob"], &["x old", "x older wait:bob"]);
        app.filter = Some(filter.to_string());

        press(&mut app, "H");
        assert_eq!((app.filter.as_deref(), shown(&app).len()), (None, 2));

        press(&mut app, "H");
        assert_eq!(app.filter, None);
    }
}

#[test]
fn h_under_a_project_keeps_it() {
    let mut app = history_of(&["one +work"], &["x old +work", "x other"]);
    app.filter = Some("+work".to_string());

    press(&mut app, "H");

    assert_eq!(app.filter.as_deref(), Some("+work"));
    assert_eq!(shown(&app), ["x old +work"]);
}

#[test]
fn the_history_refuses_every_key_that_writes() {
    let char = |c| KeyEvent::from(KeyCode::Char(c));
    let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
    let enter = KeyEvent::from(KeyCode::Enter);
    for keys in [
        vec![char('x')],
        vec![char('d'), char('d')],
        vec![char('o')],
        vec![enter],
        vec![char('p')],
        vec![char('u')],
        vec![ctrl_r],
    ] {
        let mut app = history_of(&["one", "two"], &["x old", "x older"]);
        press(&mut app, "xH");

        let wrote = keys.iter().fold(false, |wrote, key| app.handle_key(*key, TODAY) | wrote);

        assert!(!wrote, "{keys:?}");
        assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::HistoryReadOnly), true), "{keys:?}");
        assert!(matches!(app.focus, Focus::List), "{keys:?}");
        assert_eq!(lines(&app), ["two"], "{keys:?}");
        assert_eq!(moved(&app), ["+x 2026-09-26 one"], "{keys:?}");
        assert_eq!(
            app.history.iter().map(Todo::to_line).collect::<Vec<_>>(),
            ["x old", "x older"],
            "{keys:?}"
        );
    }
}

#[test]
fn dd_removes_the_selected_task_and_a_d_followed_by_another_key_does_nothing() {
    let mut app = app();

    assert!(!press(&mut app, "djd"));
    assert_eq!(app.cursor, 1);
    assert!(press(&mut app, "jdd"));

    assert_eq!(shown(&app), ["one", "two"]);
    assert_eq!(app.cursor, 1);
}

#[test]
fn p_then_a_letter_sets_the_priority_and_the_cursor_follows_the_task() {
    let mut app = app_of(&["one", "(A) two", "three"]);

    assert!(press(&mut app, "jjpb"));
    assert_eq!(shown(&app), ["(A) two", "(B) three", "one"]);
    assert_eq!(app.cursor, 1);

    assert!(press(&mut app, "kp "));
    assert_eq!(shown(&app), ["(B) three", "one", "two"]);
    assert_eq!(app.cursor, 2);
}

#[test]
fn p_does_nothing_with_another_key_or_the_same_priority() {
    let mut app = app_of(&["(A) one"]);

    assert!(!press(&mut app, "pz"));
    assert!(!press(&mut app, "pa"));

    assert_eq!(shown(&app), ["(A) one"]);
}

#[test]
fn a_key_after_p_other_than_a_to_e_or_space_says_so_and_is_swallowed() {
    let mut app = app();
    let enter = KeyEvent::from(KeyCode::Enter);

    assert!(!press(&mut app, "pj"));
    assert_eq!(told(&app), Some(Notice::NotPriority));
    assert_eq!(app.cursor, 0);
    press(&mut app, "pq");
    assert!(!app.quit);
    press(&mut app, "p");
    app.handle_key(enter, TODAY);
    assert!(matches!(app.focus, Focus::List));
    press(&mut app, "j");
    assert_eq!(app.cursor, 1);

    app.handle_key(enter, TODAY);
    press(&mut app, "pA");
    assert_eq!(told(&app), Some(Notice::NotPriority));
    press(&mut app, "p");
    app.handle_key(enter, TODAY);
    assert_eq!(popup(&app).editor.text, "two");
    press(&mut app, "pa");
    assert_eq!(popup(&app).editor.text, "(A) two");
}

#[test]
fn x_and_dd_do_nothing_on_an_empty_list() {
    let mut app = app_of(&[]);

    assert!(!press(&mut app, "xddpa"));
}

#[test]
fn j_and_k_move_the_cursor_without_leaving_the_list() {
    let mut app = app();

    press(&mut app, "k");
    assert_eq!(app.cursor, 0);
    press(&mut app, "jjj");
    assert_eq!(app.cursor, 2);
    app.handle_key(KeyEvent::from(KeyCode::Up), TODAY);
    assert_eq!(app.cursor, 1);
    app.handle_key(KeyEvent::from(KeyCode::Down), TODAY);
    assert_eq!(app.cursor, 2);
}

#[test]
fn gg_jumps_to_the_top_and_g_is_dropped_when_another_key_follows() {
    let mut app = app();

    press(&mut app, "G");
    assert_eq!(app.cursor, 2);
    press(&mut app, "gkg");
    assert_eq!(app.cursor, 1);
    press(&mut app, "gg");
    assert_eq!(app.cursor, 0);
}

#[test]
fn q_and_ctrl_c_quit() {
    let mut by_q = app();
    press(&mut by_q, "q");
    assert!(by_q.quit);

    let mut by_ctrl_c = app();
    by_ctrl_c.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), TODAY);
    assert!(by_ctrl_c.quit);

    let mut by_ctrl_alt_c = app();
    key(&mut by_ctrl_alt_c, KeyCode::Char('c'), KeyModifiers::CONTROL | KeyModifiers::ALT);
    assert!(!by_ctrl_alt_c.quit);
}

#[test]
fn a_control_key_in_the_list_is_not_its_letter() {
    let mut app = app();
    let control = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
    press(&mut app, "x");

    assert!(!app.handle_key(control('d'), TODAY));
    assert!(!app.handle_key(control('d'), TODAY));
    assert!(!app.handle_key(control('u'), TODAY));
    assert!(!app.handle_key(control('x'), TODAY));

    assert_eq!(shown(&app), ["two", "three"]);
    assert_eq!(app.todos.len(), 2);

    press(&mut app, "p");
    assert!(!app.handle_key(control('a'), TODAY));
    assert_eq!(told(&app), Some(Notice::NotPriority));

    press(&mut app, "odue:");
    app.handle_key(control('h'), TODAY);
    assert_eq!(popup(&app).picker, Some(TODAY));
}

#[test]
fn r_in_the_history_reopens_the_task_under_the_cursor_as_todo_reopen_does_and_does_nothing_in_the_list() {
    let mut app = history_of(&["one"], &["x 2026-09-20 old", "x 2026-09-25 2026-09-01 mid", "x 2026-09-26 new"]);

    assert!(!press(&mut app, "r"));
    press(&mut app, "Hj");
    assert!(!app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT), TODAY));
    assert!(press(&mut app, "r"));

    assert_eq!(lines(&app), ["one", "2026-09-01 mid"]);
    assert_eq!(shown(&app), ["x 2026-09-26 new", "x 2026-09-20 old"]);
    assert_eq!((app.cursor, told(&app), app.refusal_shown()), (1, Some(Notice::Reopened), false));
}

#[test]
fn r_refuses_a_line_todo_reopen_refuses_in_its_words_with_its_number() {
    let mut app = history_of(&["one"], &["Stray line", "x x Sell it", "x 2026-09-06 old"]);

    assert!(!press(&mut app, "Hjr"));

    let refusal = "Task 2 cannot be reopened: its text starts with x";
    assert_eq!((said(&app).as_deref(), app.refusal_shown()), (Some(refusal), true));
    assert_eq!((lines(&app), app.history.len(), moved(&app).len()), (vec!["one".to_string()], 3, 0));
}

#[test]
fn yy_leaves_the_line_of_the_task_under_the_cursor_to_copy() {
    let mut app = app_of(&["one", "(B) 2026-09-01 two +p @c k:v"]);
    assert!(!press(&mut app, "yy"));
    assert_eq!(app.copy.as_deref(), Some("(B) 2026-09-01 two +p @c k:v"));
    assert_eq!(told(&app), None);
}

#[test]
fn yy_on_a_folded_group_copies_nothing_and_says_so() {
    let mut app = app_of(&["(A) one"]);
    press(&mut app, "zMyy");
    assert_eq!(app.copy, None);
    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::NoTaskUnderCursor), true));
}

#[test]
fn yy_in_the_history_copies_the_line_of_done_txt_without_refusing() {
    let mut app = history_of(&["one"], &["x 2026-09-25 old"]);
    press(&mut app, "Hyy");
    assert_eq!(app.copy.as_deref(), Some("x 2026-09-25 old"));
    assert_eq!((told(&app), app.refusal_shown()), (None, false));
}

#[test]
fn y_then_another_key_copies_nothing() {
    let mut app = app();
    press(&mut app, "yjyg");
    assert_eq!((told(&app), app.copy), (None, None));
}
