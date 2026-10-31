use super::super::tests::{TODAY, app_of, history_of, key, press, shown, told};
use super::super::*;
use time::macros::date;

#[test]
fn the_panel_lists_all_then_projects_then_contexts_alphabetically_with_their_shown_count() {
    let mut app = app_of(&["Pay +rent @home", "Call +bank @phone +rent", "x Old +archive", "Read +Books"]);

    let expected = [("All tasks", 3), ("+bank", 1), ("+Books", 1), ("+rent", 2), ("@home", 1), ("@phone", 1)];
    assert_eq!(app.filters(), expected.map(|(name, count)| (name.to_string(), count)));

    press(&mut app, "/call");
    assert_eq!(app.filters()[0], ("All tasks".to_string(), 3));
}

#[test]
fn in_the_panel_j_and_k_filter_the_list_from_the_top_and_tab_goes_back_keeping_the_filter() {
    let mut app = app_of(&["Pay +rent", "Call +bank", "Buy milk +rent", "Mail +bank"]);
    let tab = KeyEvent::from(KeyCode::Tab);

    press(&mut app, "j");
    app.handle_key(tab, TODAY);
    press(&mut app, "jj");
    assert_eq!(app.filter.as_deref(), Some("+rent"));
    assert_eq!(shown(&app), ["Pay +rent", "Buy milk +rent"]);
    assert_eq!(app.cursor, 0);

    app.handle_key(tab, TODAY);
    press(&mut app, "j");
    assert_eq!(app.cursor, 1);
    app.handle_key(tab, TODAY);
    press(&mut app, "k");
    assert_eq!(app.filter.as_deref(), Some("+bank"));
    assert_eq!(app.cursor, 0);
}

#[test]
fn the_panel_takes_the_arrows_stops_at_its_last_entry_and_enter_and_q_act_as_in_the_list() {
    let mut app = app_of(&["Pay +rent", "Call +bank"]);

    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.filter.as_deref(), Some("+bank"));
    press(&mut app, "jj");
    assert_eq!(app.filter.as_deref(), Some("+rent"));
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.focus, Focus::List));

    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    press(&mut app, "q");
    assert!(app.quit);
}

#[test]
fn esc_in_the_panel_goes_back_to_all_and_esc_in_the_list_drops_filter_and_search() {
    let mut app = app_of(&["Pay +rent", "Call +bank"]);
    let (tab, esc) = (KeyEvent::from(KeyCode::Tab), KeyEvent::from(KeyCode::Esc));

    app.handle_key(tab, TODAY);
    press(&mut app, "j");
    app.handle_key(esc, TODAY);
    assert_eq!(app.filter, None);
    assert!(matches!(app.focus, Focus::Panel));

    press(&mut app, "j");
    app.handle_key(tab, TODAY);
    press(&mut app, "/pay");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app.handle_key(esc, TODAY);
    assert_eq!(shown(&app), ["Pay +rent", "Call +bank"]);
}

#[test]
fn a_task_added_under_a_panel_filter_gets_its_term_unless_it_has_that_exact_word() {
    let mut app = app_of(&["Pay +rent"]);
    let enter = KeyEvent::from(KeyCode::Enter);
    app.filter = Some("+rent".to_string());

    press(&mut app, "o(A) Call landlord");
    app.handle_key(enter, TODAY);
    press(&mut app, "oFix +rental form");
    app.handle_key(enter, TODAY);
    press(&mut app, "oPay +rent again");
    app.handle_key(enter, TODAY);
    press(&mut app, "o");
    app.handle_key(enter, TODAY);

    let lines: Vec<String> = app.todos.iter().map(Todo::to_line).collect();
    let added = [
        "(A) 2026-09-26 Call landlord +rent",
        "2026-09-26 Fix +rental form +rent",
        "2026-09-26 Pay +rent again",
    ];
    assert_eq!(lines[1..], added);
    assert_eq!(app.cursor, 3);
}

#[test]
fn waiting_follows_all_tasks_and_shows_the_tasks_with_a_wait_key_value() {
    let mut app = app_of(&["Design wait:figma +app", "Mail wait:designer", "Note wait: later", "Pay +rent"]);

    let counts = [("All tasks", 4), (WAITING, 2), ("+app", 1), ("+rent", 1)];
    assert_eq!(app.filters(), counts.map(|(term, count)| (term.to_string(), count)));

    app.filter = Some(WAITING.to_string());
    assert_eq!(shown(&app), ["Design wait:figma +app", "Mail wait:designer"]);
}

#[test]
fn due_follows_all_tasks_and_shows_the_pending_tasks_due_today_or_before() {
    let mut app = app_of(&[
        "Pay rent due:2026-09-26",
        "Call due:2026-09-01 wait:bank",
        "Buy milk due:2026-10-01",
        "x 2026-09-20 Mail due:2026-09-01",
        "Note",
    ]);

    let counts = [("All tasks", 4), (DUE_NOW, 2), (WAITING, 1)];
    assert_eq!(app.filters(), counts.map(|(term, count)| (term.to_string(), count)));

    app.filter = Some(DUE_NOW.to_string());
    assert_eq!(shown(&app), ["Pay rent due:2026-09-26", "Call due:2026-09-01 wait:bank"]);
}

#[test]
fn shift_d_shows_the_due_tasks_from_any_filter_and_drops_a_pending_d() {
    let mut app = app_of(&["Note +home", "Pay rent due:2026-09-26", "Call +home due:2026-09-01"]);
    app.filter = Some("+home".to_string());
    app.search = "e".to_string();
    press(&mut app, "j");

    assert!(!press(&mut app, "dD"));

    assert_eq!((app.filter.as_deref(), app.cursor), (Some(DUE_NOW), 0));
    assert_eq!(shown(&app), ["Pay rent due:2026-09-26", "Call +home due:2026-09-01"]);
    assert_eq!((app.todos.len(), app.search.as_str()), (3, "e"));
    assert_eq!(told(&app), None);

    press(&mut app, "jD");
    assert_eq!(app.cursor, 1);
}

#[test]
fn shift_d_says_nothing_due_and_keeps_the_filter_when_no_task_is_due() {
    let mut app = app_of(&["Note +home", "Buy milk due:2026-10-01"]);
    app.filter = Some("+home".to_string());

    press(&mut app, "D");

    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::NothingDue), false));
    assert_eq!(app.filter.as_deref(), Some("+home"));
}

#[test]
fn shift_d_says_nothing_due_matches_the_search_and_keeps_filter_and_search() {
    let mut app = app_of(&["Note +home", "Pay rent due:2026-09-26"]);
    app.filter = Some("+home".to_string());
    app.search = "Note".to_string();

    press(&mut app, "D");

    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::NothingDueMatchesSearch), false));
    assert_eq!((app.filter.as_deref(), app.search.as_str()), (Some("+home"), "Note"));
}

#[test]
fn shift_d_switches_to_due_when_the_search_keeps_a_due_task() {
    let mut app = app_of(&["Note +home", "Pay rent due:2026-09-26"]);
    app.search = "rent".to_string();

    press(&mut app, "D");

    assert_eq!((app.filter.as_deref(), told(&app)), (Some(DUE_NOW), None));
    assert_eq!(app.search, "rent");
}

#[test]
fn shift_d_is_refused_in_the_history() {
    let mut app = history_of(&["Pay rent due:2026-09-26"], &["x old"]);

    press(&mut app, "HD");

    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::HistoryReadOnly), true));
    assert_eq!((app.filter.as_deref(), app.in_history), (None, true));
}

#[test]
fn a_turn_brings_the_day_the_due_entry_is_counted_against() {
    let mut app = App::new(vec![Todo::from_line("Pay rent due:2026-09-27")]);
    let mut text = String::new();
    let due = |app: &App| app.filters().iter().any(|(term, _)| term == DUE_NOW);

    app.turn(Ok(String::new()), &mut text, None, TODAY);
    assert!(!due(&app));
    app.turn(Ok(String::new()), &mut text, None, date!(2026 - 09 - 27));
    assert!(due(&app));
}

#[test]
fn a_task_added_under_due_is_due_today_unless_it_carries_a_due_date() {
    let mut app = app_of(&["Pay rent due:2026-09-26"]);
    let enter = KeyEvent::from(KeyCode::Enter);
    app.filter = Some(DUE_NOW.to_string());

    press(&mut app, "oCall landlord");
    app.handle_key(enter, TODAY);
    press(&mut app, "oMail due:");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    app.handle_key(enter, TODAY);
    press(&mut app, "oFile taxes due:LLL");
    app.handle_key(enter, TODAY);
    app.handle_key(enter, TODAY);

    assert_eq!(app.todos[1].to_line(), "2026-09-26 Call landlord due:2026-09-26");
    assert_eq!(app.todos[2].to_line(), "2026-09-26 Mail due: due:2026-09-26");
    assert_eq!(app.todos[3].to_line(), "2026-09-26 File taxes due:2026-12-26");
    assert_eq!(app.filter.as_deref(), Some(DUE_NOW));
    assert_eq!(told(&app), Some(Notice::Added { hidden: true }));
}

#[test]
fn a_task_added_under_waiting_gets_nothing_and_brings_back_all_tasks_with_the_cursor_on_it() {
    let mut app = app_of(&["Design wait:figma", "Pay rent"]);
    app.filter = Some(WAITING.to_string());

    press(&mut app, "oCall landlord");
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.todos[2].to_line(), "2026-09-26 Call landlord");
    assert_eq!(app.filter, None);
    assert_eq!(app.cursor, 2);
    assert_eq!(told(&app), Some(Notice::Added { hidden: false }));
}

#[test]
fn a_panel_entry_counts_and_shows_the_tasks_with_that_exact_word() {
    let mut app = app_of(&[
        "Learn +rust",
        "Fix +rust-todo",
        "Call @home",
        "Mail x@homes.com",
        "Read +Books",
        "Sell +books",
    ]);

    let counts = [
        ("All tasks", 6),
        ("+Books", 1),
        ("+books", 1),
        ("+rust", 1),
        ("+rust-todo", 1),
        ("@home", 1),
    ];
    assert_eq!(app.filters(), counts.map(|(term, count)| (term.to_string(), count)));

    app.filter = Some("+rust".to_string());
    assert_eq!(shown(&app), ["Learn +rust"]);
}
