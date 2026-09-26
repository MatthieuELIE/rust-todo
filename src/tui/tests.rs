use super::*;
use time::macros::date;

const TODAY: Date = date!(2026 - 09 - 26);

fn app_of(lines: &[&str]) -> App {
    App::new(Store::new(lines.iter().map(|l| Todo::from_line(l)).collect()))
}

fn app() -> App {
    app_of(&["one", "two", "three"])
}

fn press(app: &mut App, keys: &str) -> bool {
    keys.chars()
        .fold(false, |write, c| app.handle_key(KeyEvent::from(KeyCode::Char(c)), TODAY) | write)
}

fn popup(app: &App) -> &Popup {
    match &app.focus {
        Focus::Popup(popup) => popup,
        _ => panic!("the popup is closed"),
    }
}

fn shown(app: &App) -> Vec<String> {
    app.tasks().into_iter().map(|(_, todo)| todo.to_line()).collect()
}

#[test]
fn x_completes_the_selected_task_and_the_next_one_takes_its_row() {
    let mut app = app();

    assert!(press(&mut app, "jx"));

    assert_eq!(shown(&app), ["one", "three"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(app.store.todos[1].to_line(), "x 2026-09-26 two");
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
    assert_eq!(app.store.todos[3].to_line(), "2026-09-26 Call the bank");
}

#[test]
fn list_keys_typed_in_the_popup_are_text() {
    let mut app = app();

    assert!(!press(&mut app, "oqxdd"));
    assert!(!app.quit);
    assert_eq!(shown(&app), ["one", "two", "three"]);
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.store.todos[3].to_line(), "2026-09-26 qxdd");
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
fn an_edited_line_is_saved_as_typed_and_one_leaving_the_list_is_said_hidden() {
    let mut app = app_of(&["one", "two"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "j");
    app.handle_key(enter, TODAY);
    press(&mut app, "ix 2026-09-20 ");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(app.store.todos[1].to_line(), "x 2026-09-20 two");
    assert_eq!(app.cursor, 0);
    assert_eq!(app.message.as_deref(), Some("edited, hidden by the filter"));
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
    assert_eq!(app.store.todos[2].to_line(), "free");
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
    assert_eq!(app.message.as_deref(), Some("a task needs a description"));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["(A) one"]);
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
    let reread = || Store::new(vec![Todo::from_line("one"), Todo::from_line("two")]);

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

    assert_eq!(app.message.as_deref(), Some("a task needs a description"));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn slash_filters_at_each_letter_from_the_top_and_enter_keeps_the_search() {
    let mut app = app_of(&["Call the bank", "Pay rent", "Call mom"]);

    press(&mut app, "jj/call");
    assert_eq!(shown(&app), ["Call the bank", "Call mom"]);
    assert_eq!(app.cursor, 0);

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

#[test]
fn the_panel_lists_all_then_projects_then_contexts_alphabetically_with_their_shown_count() {
    let mut app = app_of(&["Pay +rent @home", "Call +bank @phone +rent", "x Old +archive", "Read +Books"]);

    let expected = [("all", 3), ("+bank", 1), ("+Books", 1), ("+rent", 2), ("@home", 1), ("@phone", 1)];
    assert_eq!(app.filters(), expected.map(|(name, count)| (name.to_string(), count)));

    press(&mut app, "H/call");
    assert_eq!(app.filters()[..2], [("all".to_string(), 4), ("+archive".to_string(), 1)]);
}

#[test]
fn in_the_panel_j_and_k_filter_the_list_from_the_top_and_tab_goes_back_keeping_the_filter() {
    let mut app = app_of(&["Pay +rent", "Call +bank", "Buy milk +rent"]);
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

    let lines: Vec<String> = app.store.todos.iter().map(Todo::to_line).collect();
    let added = [
        "(A) 2026-09-26 Call landlord +rent",
        "2026-09-26 Fix +rental form +rent",
        "2026-09-26 Pay +rent again",
    ];
    assert_eq!(lines[1..], added);
    assert_eq!(app.cursor, 3);
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

    let counts = [("all", 6), ("+Books", 1), ("+books", 1), ("+rust", 1), ("+rust-todo", 1), ("@home", 1)];
    assert_eq!(app.filters(), counts.map(|(term, count)| (term.to_string(), count)));

    app.filter = Some("+rust".to_string());
    assert_eq!(shown(&app), ["Learn +rust"]);
}

#[test]
fn question_mark_opens_the_help_and_the_next_key_only_closes_it() {
    let mut app = app();

    press(&mut app, "?");
    assert!(matches!(app.focus, Focus::Help));

    assert!(!press(&mut app, "x"));
    assert!(!matches!(app.focus, Focus::Help));
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "?q");
    assert!(!app.quit);
}

#[test]
fn a_reload_keeps_the_cursor_row_and_says_so_until_the_next_key() {
    let mut app = app();
    press(&mut app, "G");

    app.reload(Store::new(vec![Todo::from_line("one"), Todo::from_line("four")]));

    assert_eq!(shown(&app), ["one", "four"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(app.message.as_deref(), Some("reloaded"));
    press(&mut app, "k");
    assert_eq!(app.message, None);
}

#[test]
fn h_shows_done_tasks_from_the_top_where_x_reopens_them() {
    let mut app = app_of(&["x 2026-09-20 one", "two"]);

    assert!(!press(&mut app, "H"));
    assert_eq!(shown(&app), ["two", "x 2026-09-20 one"]);
    assert!(press(&mut app, "jx"));
    assert_eq!(shown(&app), ["one", "two"]);

    assert_eq!(app.cursor, 1);
    press(&mut app, "H");
    assert_eq!(app.cursor, 0);
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
fn p_does_nothing_with_another_key_the_same_priority_or_a_done_task() {
    let mut app = app_of(&["(A) one", "x 2026-09-20 two"]);

    assert!(!press(&mut app, "pz"));
    assert!(!press(&mut app, "pa"));
    assert!(!press(&mut app, "Hjpb"));

    assert_eq!(shown(&app), ["(A) one", "x 2026-09-20 two"]);
}

fn redo(app: &mut App) -> bool {
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL), TODAY)
}

fn lines(app: &App) -> Vec<String> {
    app.store.todos.iter().map(Todo::to_line).collect()
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
    assert_eq!(lines(&app), ["x 2026-09-26 two", "three"]);
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
    assert!(!redo(&mut app));
    assert_eq!(app.message.as_deref(), Some("nothing to redo"));
}

#[test]
fn a_reload_forgets_the_history() {
    let mut app = app();

    press(&mut app, "xxu");
    app.reload(Store::new(vec![Todo::from_line("one")]));

    assert!(!press(&mut app, "u"));
    assert!(!redo(&mut app));
}

#[test]
fn the_groups_follow_the_priorities_on_screen_with_their_counts() {
    let mut app = app_of(&["(C) one", "two", "(A) three", "x 2026-09-20 four", "(A) five +work"]);
    use Group::*;

    assert_eq!(app.groups(), [(Priority('A'), 2), (Priority('C'), 1), (Unprioritised, 1)]);

    press(&mut app, "H");
    assert_eq!(app.groups(), [(Priority('A'), 2), (Priority('C'), 1), (Unprioritised, 1), (Done, 1)]);

    app.filter = Some("+work".to_string());
    assert_eq!(app.groups(), [(Priority('A'), 1)]);
}

#[test]
fn there_are_no_groups_when_no_task_on_screen_has_a_priority() {
    let mut app = app_of(&["one", "x 2026-09-20 two", "(A) three +work"]);

    press(&mut app, "H/-work");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert_eq!(app.groups(), []);
}

fn grouped() -> App {
    app_of(&["(A) a", "(A) b", "(B) c", "d"])
}

#[test]
fn capital_z_m_folds_every_group_into_a_row_j_and_k_step_through() {
    let mut app = grouped();

    press(&mut app, "jzM");
    assert_eq!(
        app.rows(),
        [
            Row::Group(Group::Priority('A')),
            Row::Group(Group::Priority('B')),
            Row::Group(Group::Unprioritised)
        ]
    );
    assert_eq!(app.cursor, 0);

    press(&mut app, "jjj");
    assert_eq!(app.cursor, 2);
    press(&mut app, "k");
    assert_eq!(app.cursor, 1);
}

#[test]
fn a_folded_header_ignores_the_task_keys() {
    let mut app = grouped();

    press(&mut app, "zM");
    assert!(!press(&mut app, "xddpb"));
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["(A) a", "(A) b", "(B) c", "d"]);
}

#[test]
fn capital_z_r_unfolds_back_onto_the_task_or_the_first_task_of_the_folded_group() {
    let mut app = grouped();

    press(&mut app, "jjzM");
    assert_eq!(app.cursor, 1);
    press(&mut app, "zR");
    assert_eq!(app.rows()[app.cursor], Row::Task(3));

    press(&mut app, "zMkzR");
    assert_eq!(app.rows()[app.cursor], Row::Task(1));
    assert_eq!(app.rows().len(), 4);
}

#[test]
fn z_followed_by_another_key_is_dropped_and_zm_without_groups_does_nothing() {
    let mut app = grouped();
    press(&mut app, "zj");
    assert_eq!((app.cursor, app.rows().len()), (1, 4));

    let mut app = app_of(&["one", "two"]);
    press(&mut app, "jzM");
    assert_eq!(app.rows(), [Row::Task(1), Row::Task(2)]);
    assert_eq!(app.cursor, 1);
}

#[test]
fn a_filter_keeps_the_folds_but_a_group_that_leaves_the_screen_comes_back_unfolded() {
    let mut app = grouped();
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "zM/a");
    assert_eq!(app.rows(), [Row::Group(Group::Priority('A'))]);
    app.handle_key(enter, TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert_eq!(app.rows(), [Row::Group(Group::Priority('A')), Row::Task(3), Row::Task(4)]);
}

#[test]
fn za_folds_the_group_of_the_task_onto_its_header_and_unfolds_it_onto_its_first_task() {
    let mut app = grouped();

    press(&mut app, "jza");
    assert_eq!(app.rows(), [Row::Group(Group::Priority('A')), Row::Task(3), Row::Task(4)]);
    assert_eq!(app.cursor, 0);

    press(&mut app, "jzMkza");
    assert_eq!(
        app.rows(),
        [
            Row::Task(1),
            Row::Task(2),
            Row::Group(Group::Priority('B')),
            Row::Group(Group::Unprioritised)
        ]
    );
    assert_eq!(app.cursor, 0);
}

#[test]
fn za_without_groups_does_nothing() {
    let mut app = app_of(&["one", "two"]);

    press(&mut app, "jza");

    assert_eq!(app.rows(), [Row::Task(1), Row::Task(2)]);
    assert_eq!(app.cursor, 1);
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

    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "/");
    app.paste("tw\no");
    assert_eq!(app.search, "tw o");
}

#[test]
fn a_control_key_in_the_search_types_nothing() {
    let mut app = app();

    press(&mut app, "/t");
    app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL), TODAY);

    assert_eq!(app.search, "t");
}
