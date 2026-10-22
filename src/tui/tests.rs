use super::*;
use time::macros::date;

pub(super) const TODAY: Date = date!(2026 - 09 - 26);

pub(super) fn app_of(lines: &[&str]) -> App {
    App {
        today: Some(TODAY),
        ..App::new(lines.iter().map(|l| Todo::from_line(l)).collect())
    }
}

pub(super) fn app() -> App {
    app_of(&["one", "two", "three"])
}

pub(super) fn press(app: &mut App, keys: &str) -> bool {
    keys.chars()
        .fold(false, |write, c| app.handle_key(KeyEvent::from(KeyCode::Char(c)), TODAY) | write)
}

pub(super) fn moved(app: &App) -> Vec<String> {
    let line = |moved: &Move| match moved {
        Move::Append(todo) => format!("+{}", todo.to_line()),
        Move::Remove(todo, _) => format!("-{}", todo.to_line()),
    };
    app.moves.iter().map(line).collect()
}

fn popup(app: &App) -> &Popup {
    match &app.focus {
        Focus::Popup(popup) => popup,
        _ => panic!("the popup is closed"),
    }
}

pub(super) fn shown(app: &App) -> Vec<String> {
    app.tasks().into_iter().map(|(_, todo)| todo.to_line()).collect()
}

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

pub(super) fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("todo-{}-tui-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    folder
}

#[test]
fn o_types_a_task_that_enter_adds_with_the_cursor_on_it() {
    let mut app = app();

    assert!(!press(&mut app, "jjo(A) Ask for a quote"));
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(shown(&app), ["(A) 2026-09-26 Ask for a quote", "one", "two", "three"]);
    assert_eq!(app.cursor, 0);
    assert!(!app.quit);
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn backspace_edits_the_input_and_esc_twice_drops_it() {
    let mut app = app();
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "oab");
    app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
    press(&mut app, "c");
    assert_eq!(popup(&app).editor.text, "ac");

    assert!(!app.handle_key(esc, TODAY));
    assert_eq!(popup(&app).editor.mode, Mode::Normal);
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn enter_in_normal_mode_adds_the_task_too() {
    let mut app = app();

    press(&mut app, "oCall bank");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "bithe ");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));
    assert_eq!(app.todos[3].to_line(), "2026-09-26 Call the bank");
}

#[test]
fn list_keys_typed_in_the_popup_are_text() {
    let mut app = app();

    assert!(!press(&mut app, "oqxdd"));
    assert!(!app.quit);
    assert_eq!(shown(&app), ["one", "two", "three"]);
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.todos[3].to_line(), "2026-09-26 qxdd");
}

#[test]
fn enter_opens_the_task_line_in_normal_mode_and_the_cursor_follows_the_edited_task() {
    let mut app = app_of(&["one", "two", "three"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    assert!(!press(&mut app, "jj"));
    app.handle_key(enter, TODAY);
    let editor = &popup(&app).editor;
    assert_eq!((editor.text.as_str(), editor.cursor, editor.mode), ("three", 0, Mode::Normal));

    press(&mut app, "i(A) ");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(shown(&app), ["(A) three", "one", "two"]);
    assert_eq!(app.cursor, 0);
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn a_completion_date_typed_with_the_x_is_the_one_the_task_moves_to_done_txt_with() {
    let mut app = app_of(&["one", "two"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "j");
    app.handle_key(enter, TODAY);
    press(&mut app, "ix 2026-09-20 ");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(lines(&app), ["one"]);
    assert_eq!(moved(&app), ["+x 2026-09-20 two"]);
    assert_eq!(app.cursor, 0);
}

#[test]
fn a_line_holding_a_control_character_opens_for_edit_with_a_space_in_its_place() {
    let mut app = app_of(&["Call\tthe bank"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.todos[0].to_line(), "Call the bank");
}

#[test]
fn an_edit_hidden_by_the_search_leaves_the_cursor_where_it_was() {
    let mut app = app_of(&["one", "two", "three"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "/t");
    app.handle_key(enter, TODAY);
    press(&mut app, "j");
    app.handle_key(enter, TODAY);
    press(&mut app, "Cfree");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(shown(&app), ["two"]);
    assert_eq!(app.cursor, 0);
    assert_eq!(app.todos[2].to_line(), "free");
    assert_eq!(app.message.as_deref(), Some("edited, hidden by the filter"));
}

#[test]
fn an_edit_emptied_or_left_unchanged_writes_nothing() {
    let mut app = app_of(&["(A) one"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    app.handle_key(enter, TODAY);
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message, None);

    app.handle_key(enter, TODAY);
    press(&mut app, "WD");
    assert_eq!(popup(&app).editor.text, "(A) ");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message.as_deref(), Some("a task needs more than +projects, @contexts and key: words"));
    assert_eq!(popup(&app).editor.text, "(A) ");
    assert_eq!(shown(&app), ["(A) one"]);
}

#[test]
fn a_trailing_space_is_dropped_on_add_and_edit() {
    let mut app = app_of(&["one"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "oShip ");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(shown(&app), ["one", "2026-09-26 Ship"]);

    app.handle_key(enter, TODAY);
    press(&mut app, "A ");
    assert!(!app.handle_key(enter, TODAY));

    app.handle_key(enter, TODAY);
    press(&mut app, "A it ");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(shown(&app), ["one", "2026-09-26 Ship it"]);
}

#[test]
fn a_refusal_is_told_apart_from_an_information_until_the_next_key() {
    let mut app = app_of(&["one"]);

    press(&mut app, "pz");
    assert_eq!((app.message.as_deref(), app.refused), (Some(NOT_PRIORITY), true));

    press(&mut app, "u");
    assert_eq!((app.message.as_deref(), app.refused), (Some("nothing to undo"), false));
}

#[test]
fn the_cursor_is_a_bar_while_typing_in_the_popup_and_a_block_otherwise() {
    let mut app = app_of(&["one"]);
    assert_eq!(app.cursor_shape(), SetCursorStyle::SteadyBlock);

    press(&mut app, "o");
    assert_eq!(app.cursor_shape(), SetCursorStyle::SteadyBar);

    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    assert_eq!(app.cursor_shape(), SetCursorStyle::SteadyBlock);
}

#[test]
fn enter_on_an_empty_list_opens_nothing() {
    let mut app = app_of(&[]);

    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn a_reload_cancels_an_edit_but_keeps_an_add_being_typed() {
    let mut app = app();
    let reread = || vec![Todo::from_line("one"), Todo::from_line("two")];

    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app.reload(reread());
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(app.message.as_deref(), Some("reloaded, edit cancelled"));

    press(&mut app, "onew");
    app.reload(reread());
    assert_eq!(popup(&app).editor.text, "new");
    assert_eq!(app.message.as_deref(), Some("reloaded"));
}

#[test]
fn a_rejected_input_adds_nothing_and_says_why() {
    let mut app = app();

    press(&mut app, "o");
    assert!(!app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.message.as_deref(), Some("a task needs more than +projects, @contexts and key: words"));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn a_refused_line_keeps_the_popup_open_with_its_text_and_esc_closes_it_writing_nothing() {
    let mut app = app();
    let enter = KeyEvent::from(KeyCode::Enter);
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "o");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message.as_deref(), Some("a task needs more than +projects, @contexts and key: words"));
    assert_eq!(popup(&app).editor.text, "");

    press(&mut app, "x Lorem");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message.as_deref(), Some("cannot add a task that is already done"));
    assert_eq!(popup(&app).editor.text, "x Lorem");

    app.handle_key(esc, TODAY);
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn an_accepted_add_closes_the_popup() {
    let mut app = app();

    press(&mut app, "oLorem");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn slash_filters_at_each_letter_from_the_top_and_enter_keeps_the_search() {
    let mut app = app_of(&["Call the bank", "Pay rent", "Call mom"]);

    press(&mut app, "jj/call");
    assert_eq!(shown(&app), ["Call the bank", "Call mom"]);
    assert_eq!(app.cursor, 0);
    app.cursor = 1;
    app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
    assert_eq!((app.search.as_str(), app.cursor), ("cal", 0));
    press(&mut app, "l");

    press(&mut app, " -mom");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "j");
    assert_eq!(shown(&app), ["Call the bank"]);
}

#[test]
fn esc_drops_the_search_being_typed_or_the_one_kept_in_the_list() {
    let mut app = app();
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "/one");
    app.handle_key(esc, TODAY);
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "/one");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app.handle_key(esc, TODAY);
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "jj");
    app.handle_key(esc, TODAY);
    assert_eq!(app.cursor, 0);

    press(&mut app, "/one");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "/");
    assert_eq!(app.search, "");
}

#[test]
fn a_task_added_under_a_search_it_does_not_match_is_saved_but_said_hidden() {
    let mut app = app();
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "/t");
    app.handle_key(enter, TODAY);
    press(&mut app, "jofour");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(shown(&app), ["two", "three"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(app.message.as_deref(), Some("added, hidden by the filter"));
}

fn scroll(app: &App) -> u16 {
    match app.focus {
        Focus::Help(scroll) => scroll,
        _ => panic!("the help is closed"),
    }
}

fn help_app() -> App {
    let app = app();
    app.help_rows.set(26);
    app
}

#[test]
fn j_and_k_scroll_the_help_a_line_and_stop_at_the_top_and_the_bottom() {
    let mut app = help_app();
    let max = crate::view::HELP.lines().count() as u16 - 26;

    press(&mut app, "?");
    assert_eq!(scroll(&app), 0);
    press(&mut app, "k");
    assert_eq!(scroll(&app), 0);
    press(&mut app, "j");
    assert_eq!(scroll(&app), 1);
    press(&mut app, "k");
    assert_eq!(scroll(&app), 0);
    press(&mut app, &"j".repeat(max as usize + 5));
    assert_eq!(scroll(&app), max);
    key(&mut app, KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(scroll(&app), max - 1);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(scroll(&app), max);
}

#[test]
fn capital_g_scrolls_the_help_to_the_bottom_and_gg_back_to_the_top() {
    let mut app = help_app();
    let max = crate::view::HELP.lines().count() as u16 - 26;

    press(&mut app, "?G");
    assert_eq!(scroll(&app), max);
    press(&mut app, "g");
    assert_eq!(scroll(&app), max);
    press(&mut app, "g");
    assert_eq!(scroll(&app), 0);
}

#[test]
fn esc_enter_question_mark_and_q_each_close_the_help_with_no_other_effect() {
    for code in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char('?'), KeyCode::Char('q')] {
        let mut app = help_app();
        press(&mut app, "?");
        key(&mut app, code, KeyModifiers::NONE);
        assert!(matches!(app.focus, Focus::List), "{code:?}");
        assert!(!app.quit && app.moves.is_empty() && app.message.is_none(), "{code:?}");
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }
}

#[test]
fn any_other_key_leaves_the_help_open_and_changes_nothing() {
    let mut app = help_app();
    press(&mut app, "?");

    for keys in ["x", "/", "dd", "o"] {
        assert!(!press(&mut app, keys));
        assert_eq!(scroll(&app), 0);
    }
    assert_eq!(shown(&app), ["one", "two", "three"]);
    assert!(app.moves.is_empty());
}

#[test]
fn ctrl_c_quits_from_the_help() {
    let mut app = help_app();
    press(&mut app, "?");
    key(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(app.quit);
}

#[test]
fn the_help_opened_again_starts_at_the_top() {
    let mut app = help_app();
    press(&mut app, "?jjj");
    assert_eq!(scroll(&app), 3);
    press(&mut app, "q?");
    assert_eq!(scroll(&app), 0);
}

pub(super) fn history_of(lines: &[&str], done: &[&str]) -> App {
    App {
        history: done.iter().map(|l| Todo::from_line(l)).collect(),
        ..app_of(lines)
    }
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
    assert_eq!((app.cursor, app.message.as_deref()), (1, None));

    assert!(!press(&mut app, "H"));
    assert_eq!(shown(&app), ["(A) one"]);
    assert_eq!(app.cursor, 0);
}

#[test]
fn the_panel_and_the_search_work_on_the_history_and_both_are_kept_across_h() {
    let mut app = history_of(
        &["Pay +rent @web", "Call +bank"],
        &["x Paid +rent", "x Sold +car @web", "x Mailed +rent @web"],
    );
    app.filter = Some("+rent".to_string());
    app.search = "pa".to_string();
    assert_eq!(shown(&app), ["Pay +rent @web"]);

    press(&mut app, "H");
    let entries: Vec<String> = app.filters().iter().map(|(term, count)| format!("{term} {count}")).collect();
    assert_eq!(entries, ["All tasks 3", "+car 1", "+rent 2", "@web 2"]);
    assert_eq!(shown(&app), ["x Paid +rent"]);

    press(&mut app, "/mail");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    assert_eq!(shown(&app), ["x Mailed +rent @web"]);

    press(&mut app, "H");
    assert_eq!((app.filter.as_deref(), app.search.as_str()), (Some("+rent"), "mail"));
    assert!(shown(&app).is_empty());
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
fn the_history_panel_has_no_due_and_no_waiting_entry() {
    let mut app = history_of(&["one"], &["x Asked wait:bob +home", "Left pending due:2026-09-01"]);

    press(&mut app, "H");

    let entries: Vec<String> = app.filters().into_iter().map(|(term, _)| term).collect();
    assert_eq!(entries, ["All tasks", "+home"]);
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
        assert_eq!((app.message.as_deref(), app.refused), (Some("history is read-only"), true), "{keys:?}");
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
    assert_eq!(app.message.as_deref(), Some("priority is a to e, or space"));
    assert_eq!(app.cursor, 0);
    press(&mut app, "pq");
    assert!(!app.quit);
    press(&mut app, "p");
    app.handle_key(enter, TODAY);
    assert!(matches!(app.focus, Focus::List));
    press(&mut app, "j");
    assert_eq!((app.message.as_deref(), app.cursor), (None, 1));

    app.handle_key(enter, TODAY);
    press(&mut app, "pA");
    assert_eq!(app.message.as_deref(), Some("priority is a to e, or space"));
    press(&mut app, "p");
    app.handle_key(enter, TODAY);
    assert_eq!(popup(&app).editor.text, "two");
    press(&mut app, "pa");
    assert_eq!((popup(&app).editor.text.as_str(), app.message.as_deref()), ("(A) two", None));
}

pub(super) fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(KeyEvent::new(code, modifiers), TODAY);
}

pub(super) fn names(app: &App) -> Vec<String> {
    app.completions().0.iter().map(|(name, count)| format!("{name} {count}")).collect()
}

#[test]
fn a_tag_typed_in_the_popup_lists_the_names_of_the_file_starting_like_it_most_used_first() {
    let mut app = app_of(&[
        "Pay +bank",
        "Call +Bank @phone",
        "x 2026-09-20 Old +bank",
        "Read +books +bank",
        "Sell +rent",
    ]);

    press(&mut app, "oBuy +b");
    assert_eq!(names(&app), ["+bank 3", "+Bank 1", "+books 1"]);
    press(&mut app, "O");
    assert_eq!(names(&app), ["+books 1"]);
    press(&mut app, " @");
    assert_eq!(names(&app), ["@phone 1"]);
    press(&mut app, " ");
    assert!(names(&app).is_empty());

    press(&mut app, "+r");
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(names(&app), ["+bank 3", "+Bank 1", "+books 1", "+rent 1"]);
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert!(names(&app).is_empty());
    key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(names(&app).len(), 4);
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(names(&app).is_empty());
}

#[test]
fn wait_colon_typed_in_the_popup_lists_the_wait_values_of_the_file_most_used_first() {
    let mut app = app_of(&[
        "Design wait:figma",
        "x 2026-09-20 Mock wait:figma",
        "Mail wait:designer",
        "Note wait: later",
        "Pay +rent",
    ]);

    press(&mut app, "oDraw wait:");
    assert_eq!(names(&app), ["wait:figma 2", "wait:designer 1"]);
    press(&mut app, "d");
    assert_eq!(names(&app), ["wait:designer 1"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(popup(&app).editor.text, "Draw wait:designer ");
}

#[test]
fn the_names_of_done_txt_complete_a_tag_after_those_of_the_task_file_with_a_count_of_zero() {
    let mut app = history_of(
        &["Pay +bank", "Call +bank @phone", "Read +books"],
        &["x 2026-09-20 Old +bank +archive @Home wait:bob", "x 2026-09-21 Older +Attic +archive"],
    );

    press(&mut app, "o+");
    assert_eq!(names(&app), ["+bank 2", "+books 1", "+archive 0", "+Attic 0"]);
    press(&mut app, " @");
    assert_eq!(names(&app), ["@phone 1", "@Home 0"]);
    press(&mut app, " wait:");
    assert_eq!(names(&app), ["wait:bob 0"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(popup(&app).editor.text, "+ @ wait:bob ");
}

#[test]
fn arrows_and_ctrl_n_p_pick_a_name_that_tab_writes_in_place_of_the_tag() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Read +books"]);
    let pick = |app: &mut App, code: KeyCode, modifiers: KeyModifiers| {
        key(app, code, modifiers);
        app.completions().1
    };

    press(&mut app, "oCall +B");
    assert_eq!(app.completions().1, 0);
    assert_eq!(pick(&mut app, KeyCode::Down, KeyModifiers::NONE), 1);
    assert_eq!(pick(&mut app, KeyCode::Down, KeyModifiers::NONE), 1);
    assert_eq!(pick(&mut app, KeyCode::Char('p'), KeyModifiers::CONTROL), 0);
    assert_eq!(pick(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL), 1);
    assert_eq!(pick(&mut app, KeyCode::Up, KeyModifiers::NONE), 0);
    assert_eq!(pick(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL), 1);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!((popup(&app).editor.text.as_str(), popup(&app).editor.cursor), ("Call +books ", 12));
    assert!(names(&app).is_empty());

    press(&mut app, "+b now");
    (0..5).for_each(|_| key(&mut app, KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(names(&app), ["+bank 2", "+books 1"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(
        (popup(&app).editor.text.as_str(), popup(&app).editor.cursor),
        ("Call +books +bank now", 18)
    );

    press(&mut app, "+");
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(shown(&app).last().unwrap(), "2026-09-26 Call +books +bank +now");
}

#[test]
fn the_pick_goes_back_to_the_first_name_after_a_typed_key_and_after_tab() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Wash +bath"]);

    press(&mut app, "o+b");
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.completions().1, 1);
    press(&mut app, "a");
    assert_eq!(
        (names(&app), app.completions().1),
        (vec!["+bank 2".to_string(), "+bath 1".to_string()], 0)
    );

    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    app.paste("+b");
    assert_eq!((popup(&app).editor.text.as_str(), app.completions().1), ("+bath +b", 0));
}

#[test]
fn names_used_as_often_are_listed_alphabetically_whatever_their_case() {
    let mut app = app_of(&["Pay +Bank", "Eat +apple"]);

    press(&mut app, "o+");

    assert_eq!(names(&app), ["+apple 1", "+Bank 1"]);
}

#[test]
fn a_month_step_in_the_date_picker_crosses_the_year_and_keeps_to_the_month_length() {
    assert_eq!(shift(date!(2026 - 12 - 15), KeyCode::Char('L')), date!(2027 - 01 - 15));
    assert_eq!(shift(date!(2027 - 01 - 15), KeyCode::Char('H')), date!(2026 - 12 - 15));
    assert_eq!(shift(date!(2028 - 01 - 31), KeyCode::Char('L')), date!(2028 - 02 - 29));
}

pub(super) fn redo(app: &mut App) -> bool {
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL), TODAY)
}

pub(super) fn lines(app: &App) -> Vec<String> {
    app.todos.iter().map(Todo::to_line).collect()
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
fn a_paste_types_its_lines_as_one_into_the_popup_or_the_search_and_does_nothing_in_the_list() {
    let mut app = app();

    app.paste("dd");
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "oCall ");
    app.paste("the\r\nbank\n");
    assert_eq!(popup(&app).editor.text, "Call the bank");
    assert_eq!(popup(&app).editor.cursor, 13);
    app.paste(" on\rMonday");
    assert_eq!(popup(&app).editor.text, "Call the bank on Monday");
    app.paste(" at\tnoon\u{1b}");
    assert_eq!(popup(&app).editor.text, "Call the bank on Monday at noon");

    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "/");
    app.paste("tw\no");
    assert_eq!(app.search, "tw o");
}

#[test]
fn an_x_typed_in_front_of_a_task_moves_it_to_done_txt_completed_today_and_u_brings_it_back() {
    let mut app = app_of(&["(A) 2026-08-01 one", "two"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "ix ");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(lines(&app), ["two"]);
    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one"]);
    assert_eq!(app.message, None);
    assert!(press(&mut app, "u"));
    assert_eq!(lines(&app), ["(A) 2026-08-01 one", "two"]);
    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one", "-x 2026-09-26 2026-08-01 one"]);
}

#[test]
fn an_x_typed_in_front_of_a_dated_task_keeps_its_creation_date() {
    let mut app = app_of(&["2026-08-01 one"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "ix ");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one"]);
}

#[test]
fn u_in_the_popup_undoes_the_typed_text_and_says_so_without_touching_the_list() {
    let mut app = app();
    press(&mut app, "ofour");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert!(!press(&mut app, "u"));
    assert_eq!(popup(&app).editor.text, "");
    assert_eq!((app.message.as_deref(), app.refused), (Some("undone"), false));

    press(&mut app, "u");
    assert_eq!(app.message.as_deref(), Some("nothing to undo"));
    assert_eq!(shown(&app), ["one", "two", "three"]);
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
    assert_eq!(app.message.as_deref(), Some(NOT_PRIORITY));

    press(&mut app, "odue:");
    app.handle_key(control('h'), TODAY);
    assert_eq!(popup(&app).picker, Some(TODAY));
}

#[test]
fn a_control_key_in_the_search_types_nothing() {
    let mut app = app();

    press(&mut app, "/t");
    app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL), TODAY);

    assert_eq!(app.search, "t");
}

#[test]
fn due_colon_typed_in_the_popup_opens_a_date_picker_on_today_whose_keys_move_the_date_and_enter_writes_it() {
    let mut app = app();
    let picked = |app: &App| popup(app).picker;

    press(&mut app, "oShip due:");
    assert_eq!(picked(&app), Some(TODAY));
    press(&mut app, "lj");
    assert_eq!(picked(&app), Some(date!(2026 - 10 - 04)));
    press(&mut app, "hk");
    assert_eq!(picked(&app), Some(TODAY));
    key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    key(&mut app, KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(picked(&app), Some(TODAY));
    press(&mut app, "L");
    assert_eq!(picked(&app), Some(date!(2026 - 10 - 26)));
    press(&mut app, "HHlllll");
    assert_eq!(picked(&app), Some(date!(2026 - 08 - 31)));
    press(&mut app, "Lx?");
    app.paste("2026-01-01");
    assert_eq!(
        (picked(&app), popup(&app).editor.text.as_str()),
        (Some(date!(2026 - 09 - 30)), "Ship due:")
    );

    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(picked(&app), None);
    assert_eq!(
        (popup(&app).editor.text.as_str(), popup(&app).editor.mode),
        ("Ship due:2026-09-30 ", Mode::Insert)
    );
}

#[test]
fn esc_closes_the_date_picker_on_due_colon_and_a_key_leaving_due_colon_again_reopens_it() {
    let mut app = app();

    press(&mut app, "oShip due:");
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!((popup(&app).picker, popup(&app).editor.text.as_str()), (None, "Ship due:"));
    assert_eq!(popup(&app).editor.mode, Mode::Insert);

    press(&mut app, "2");
    assert_eq!(popup(&app).picker, None);
    key(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
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
    assert_eq!((app.cursor, app.message.as_deref(), app.refused), (1, Some("reopened"), false));
}

#[test]
fn r_refuses_a_line_todo_reopen_refuses_in_its_words_with_its_number() {
    let mut app = history_of(&["one"], &["Stray line", "x x Sell it", "x 2026-09-06 old"]);

    assert!(!press(&mut app, "Hjr"));

    let refusal = "task 2 cannot be reopened: its text starts with x";
    assert_eq!((app.message.as_deref(), app.refused), (Some(refusal), true));
    assert_eq!((lines(&app), app.history.len(), moved(&app).len()), (vec!["one".to_string()], 3, 0));
}

#[test]
fn u_after_a_reopen_whose_line_could_not_leave_done_txt_has_nothing_to_undo() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("reopen-held");
    let (file, locked) = (folder.join("todo.txt"), folder.join("locked"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("done.txt"), "x 2026-09-20 old\n").unwrap();
    std::os::unix::fs::symlink(locked.join("done.txt"), folder.join("done.txt")).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();
    let read = |path| std::fs::read_to_string(path).unwrap();

    app.refresh_history(&file);
    press(&mut app, "Hr");
    app.save(&file, &mut text);
    let held = (read(&file), app.refused, app.message.clone().unwrap_or_default());
    let wrote = press(&mut app, "Hu");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(held.2.starts_with("reopened, but done.txt still holds the task: "), "{}", held.2);
    assert_eq!((held.0, held.1), ("one\nold\n".to_string(), true));
    assert!(!wrote);
    assert_eq!(app.message.as_deref(), Some("nothing to undo"));
}

#[test]
fn ctrl_r_after_a_u_whose_line_could_not_leave_done_txt_does_not_add_it_there_again() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("undo-held");
    let (file, locked) = (folder.join("todo.txt"), folder.join("locked"));
    std::fs::write(&file, "one\ntwo\n").unwrap();
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("done.txt"), "").unwrap();
    std::os::unix::fs::symlink(locked.join("done.txt"), folder.join("done.txt")).unwrap();
    let mut app = app_of(&["one", "two"]);
    let mut text = "one\ntwo\n".to_string();
    let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);

    press(&mut app, "x");
    app.save(&file, &mut text);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    press(&mut app, "u");
    app.save(&file, &mut text);
    let held = (app.refused, app.message.clone().unwrap_or_default());
    let wrote = app.handle_key(ctrl_r, TODAY);
    let done = std::fs::read_to_string(locked.join("done.txt")).unwrap();

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(held.1.starts_with("undone, but done.txt still holds the task: "), "{}", held.1);
    assert!(held.0);
    assert!(!wrote);
    assert_eq!(app.message.as_deref(), Some("nothing to redo"));
    assert_eq!(done.matches("one").count(), 1);
}

#[test]
fn yy_leaves_the_line_of_the_task_under_the_cursor_to_copy() {
    let mut app = app_of(&["one", "(B) 2026-09-01 two +p @c k:v"]);
    assert!(!press(&mut app, "yy"));
    assert_eq!(app.copy.as_deref(), Some("(B) 2026-09-01 two +p @c k:v"));
    assert_eq!(app.message, None);
}

#[test]
fn yy_on_a_folded_group_copies_nothing_and_says_so() {
    let mut app = app_of(&["(A) one"]);
    press(&mut app, "zMyy");
    assert_eq!(app.copy, None);
    assert_eq!((app.message.as_deref(), app.refused), (Some("no task under the cursor"), true));
}

#[test]
fn yy_in_the_history_copies_the_line_of_done_txt_without_refusing() {
    let mut app = history_of(&["one"], &["x 2026-09-25 old"]);
    press(&mut app, "Hyy");
    assert_eq!(app.copy.as_deref(), Some("x 2026-09-25 old"));
    assert_eq!((app.message, app.refused), (None, false));
}

#[test]
fn y_then_another_key_copies_nothing() {
    let mut app = app();
    press(&mut app, "yjyg");
    assert_eq!((app.copy, app.message), (None, None));
}

#[test]
fn a_line_of_only_tags_is_refused_unchanged_ignored_and_ended_by_a_typed_x() {
    let enter = KeyEvent::from(KeyCode::Enter);
    let mut app = app_of(&["+work"]);

    app.handle_key(enter, TODAY);
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message, None);

    app.handle_key(enter, TODAY);
    press(&mut app, "A wait:a");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message.as_deref(), Some("a task needs more than +projects, @contexts and key: words"));
    assert_eq!(shown(&app), ["+work"]);

    let mut app = app_of(&["+work"]);
    app.handle_key(enter, TODAY);
    press(&mut app, "Ix ");
    assert_eq!(popup(&app).editor.text, "x +work");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(moved(&app), ["+x 2026-09-26 +work"]);
}

#[test]
fn a_line_replaced_by_a_lone_x_is_refused() {
    let enter = KeyEvent::from(KeyCode::Enter);
    let mut app = app_of(&["one"]);
    app.handle_key(enter, TODAY);
    press(&mut app, "0Dix ");
    assert_eq!(popup(&app).editor.text, "x ");

    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(app.message.as_deref(), Some("a task needs more than +projects, @contexts and key: words"));
    assert_eq!(shown(&app), ["one"]);
    assert!(moved(&app).is_empty());
}
