use super::*;
use time::macros::date;

const TODAY: Date = date!(2026 - 09 - 26);

fn app_of(lines: &[&str]) -> App {
    App {
        today: Some(TODAY),
        ..App::new(lines.iter().map(|l| Todo::from_line(l)).collect())
    }
}

fn app() -> App {
    app_of(&["one", "two", "three"])
}

fn press(app: &mut App, keys: &str) -> bool {
    keys.chars()
        .fold(false, |write, c| app.handle_key(KeyEvent::from(KeyCode::Char(c)), TODAY) | write)
}

fn moved(app: &App) -> Vec<String> {
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

fn shown(app: &App) -> Vec<String> {
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

fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("todo-{}-tui-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    folder
}

#[test]
fn save_writes_done_txt_then_the_task_file_and_u_takes_the_line_back_out() {
    let folder = folder("save");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\n(A) two\n").unwrap();
    std::fs::write(&done, "x 2026-09-01 zero\nx 2026-09-26 two\n").unwrap();
    let mut app = app_of(&["one", "(A) two"]);
    let mut text = "one\n(A) two\n".to_string();
    let read = |path| std::fs::read_to_string(path).unwrap();

    press(&mut app, "x");
    app.save(&file, &mut text);
    let completed = (read(&file), read(&done), text.clone());
    press(&mut app, "u");
    app.save(&file, &mut text);
    let undone = (read(&file), read(&done), app.message.clone());
    press(&mut app, "x");
    app.save(&file, &mut text);
    std::fs::write(&done, "x 2026-09-01 zero\n").unwrap();
    press(&mut app, "u");
    app.save(&file, &mut text);
    let missing = (read(&file), read(&done), app.message.clone(), app.refused);

    std::fs::remove_dir_all(&folder).unwrap();
    let history = "x 2026-09-01 zero\nx 2026-09-26 two\n";
    assert_eq!(completed, ("one\n".into(), format!("{history}x 2026-09-26 two\n"), "one\n".into()));
    assert_eq!(undone, ("one\n(A) two\n".into(), history.into(), Some("undone".into())));
    let restored = ("one\n(A) two\n".into(), "x 2026-09-01 zero\n".into());
    assert_eq!(
        missing,
        (restored.0, restored.1, Some("undone, the task was no longer in done.txt".into()), false)
    );
}

#[test]
fn save_leaves_both_files_alone_when_the_move_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("save-refused");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\ntwo\n").unwrap();
    std::fs::write(&done, "").unwrap();
    std::fs::set_permissions(&done, std::fs::Permissions::from_mode(0o444)).unwrap();
    let mut app = app_of(&["one", "two"]);
    let mut text = "one\ntwo\n".to_string();

    press(&mut app, "x");
    app.save(&file, &mut text);
    let refused = (lines(&app), app.refused, std::fs::read_to_string(&file).unwrap());
    let itself = folder.join("itself");
    std::fs::create_dir(&itself).unwrap();
    std::fs::write(itself.join("done.txt"), "one\ntwo\n").unwrap();
    press(&mut app, "x");
    app.save(&itself.join("done.txt"), &mut text);
    let kept = std::fs::read_to_string(itself.join("done.txt")).unwrap();

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(refused, (vec!["one".to_string(), "two".to_string()], true, "one\ntwo\n".to_string()));
    assert_eq!(kept, "one\ntwo\n");
    assert_eq!(
        app.message.as_deref(),
        Some("could not save: the task file is done.txt itself (file left unchanged)")
    );
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
    assert_eq!(app.message.as_deref(), Some("a task needs a description"));
    assert!(!matches!(app.focus, Focus::Popup(_)));
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

    assert_eq!(app.message.as_deref(), Some("a task needs a description"));
    assert_eq!(shown(&app), ["one", "two", "three"]);
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
    assert_eq!(app.message.as_deref(), Some("added, hidden by the filter"));
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
    assert_eq!(app.message, None);
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

    press(&mut app, "?");
    key(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(!app.quit);
}

#[test]
fn a_reload_keeps_the_cursor_row_and_says_so_until_the_next_key() {
    let mut app = app();
    press(&mut app, "G");

    app.reload(vec![Todo::from_line("one"), Todo::from_line("four")]);

    assert_eq!(shown(&app), ["one", "four"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(app.message.as_deref(), Some("reloaded"));
    press(&mut app, "k");
    assert_eq!(app.message, None);
}

fn history_of(lines: &[&str], done: &[&str]) -> App {
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
fn the_history_panel_has_no_due_and_no_waiting_entry() {
    let mut app = history_of(&["one"], &["x Asked wait:bob +home", "Left pending due:2026-09-01"]);

    press(&mut app, "H");

    let entries: Vec<String> = app.filters().into_iter().map(|(term, _)| term).collect();
    assert_eq!(entries, ["All tasks", "+home"]);
}

#[test]
fn done_txt_is_read_when_the_list_opens_at_each_h_and_at_a_reload_and_not_in_between() {
    let folder = folder("history");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    let append = |line: &str| {
        use std::io::Write;
        let mut history = std::fs::OpenOptions::new().append(true).create(true).open(&done).unwrap();
        writeln!(history, "{line}").unwrap();
    };
    append("x first");
    let mut app = app();

    app.refresh_history(&file);
    let opened = app.history.len();
    press(&mut app, "H");
    app.refresh_history(&file);
    let first = shown(&app);
    append("x second");
    press(&mut app, "j");
    app.refresh_history(&file);
    let unwatched = shown(&app);
    press(&mut app, "HH");
    app.refresh_history(&file);
    let second = shown(&app);
    append("x third");
    app.reload(vec![Todo::from_line("one")]);
    app.refresh_history(&file);
    let reloaded = shown(&app);

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(opened, 1);
    assert_eq!(first, ["x first"]);
    assert_eq!(unwatched, ["x first"]);
    assert_eq!(second, ["x second", "x first"]);
    assert_eq!(reloaded, ["x third", "x second", "x first"]);
}

#[test]
fn done_txt_is_read_again_after_a_save_that_moved_a_task_so_its_names_still_complete() {
    let folder = folder("moved");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay +bank\nRead +books\n").unwrap();
    let mut app = app_of(&["Pay +bank", "Read +books"]);
    let mut text = "Pay +bank\nRead +books\n".to_string();

    app.refresh_history(&file);
    press(&mut app, "jx");
    app.save(&file, &mut text);
    app.refresh_history(&file);
    press(&mut app, "o+b");
    let completed = names(&app);
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    press(&mut app, "u");
    app.save(&file, &mut text);
    app.refresh_history(&file);
    let undone = app.history.len();

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(completed, ["+bank 1", "+books 0"]);
    assert_eq!(undone, 0);
}

#[test]
fn reading_done_txt_again_after_a_save_keeps_the_refusal_that_save_left() {
    let folder = folder("kept");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\ntwo\n").unwrap();
    let mut app = app_of(&["one", "two"]);
    let mut text = "one\ntwo\n".to_string();

    press(&mut app, "x");
    app.save(&file, &mut text);
    app.refresh_history(&file);
    std::fs::write(&done, [0xff, 0xfe, b'\n']).unwrap();
    press(&mut app, "u");
    app.save(&file, &mut text);
    app.refresh_history(&file);

    std::fs::remove_dir_all(&folder).unwrap();
    assert!(app.history.is_empty());
    assert!(app.refused);
    assert!(app.message.unwrap().starts_with("undone, but done.txt still holds the task: "));
}

#[test]
fn a_reload_after_a_refusal_is_not_told_as_one() {
    let mut app = app();
    app.refuse("history is read-only");

    app.reload(vec![Todo::from_line("one")]);

    assert_eq!((app.message.as_deref(), app.refused), (Some("reloaded"), false));
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
fn the_history_is_what_done_txt_holds_and_a_done_txt_that_cannot_be_read_is_told() {
    let mut app = app();

    app.set_history(Ok("x old\n\nx new\n".to_string()));
    assert_eq!(app.history.iter().map(Todo::to_line).collect::<Vec<_>>(), ["x old", "x new"]);
    assert_eq!(app.message, None);

    app.set_history(Err(io::ErrorKind::InvalidData.into()));
    assert!(app.history.is_empty());
    assert!(app.refused);
    assert!(app.message.unwrap().starts_with("could not read done.txt: "));
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

fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(KeyEvent::new(code, modifiers), TODAY);
}

fn names(app: &App) -> Vec<String> {
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

fn redo(app: &mut App) -> bool {
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL), TODAY)
}

fn lines(app: &App) -> Vec<String> {
    app.todos.iter().map(Todo::to_line).collect()
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

#[test]
fn a_turn_reloads_a_file_that_changed_and_ignores_the_key_of_that_turn() {
    let mut app = app();
    let mut text = "one\ntwo\nthree\n".to_string();
    let x = Some(Event::Key(KeyEvent::from(KeyCode::Char('x'))));

    assert!(!app.turn(Ok("one\nfour\n".to_string()), &mut text, x.clone(), TODAY));
    assert_eq!(shown(&app), ["one", "four"]);
    assert_eq!(text, "one\nfour\n");

    let released = KeyEvent::new_with_kind(KeyCode::Char('x'), KeyModifiers::NONE, KeyEventKind::Release);
    assert!(!app.turn(Ok("one\nfour\n".to_string()), &mut text, Some(Event::Key(released)), TODAY));
    assert_eq!(shown(&app), ["one", "four"]);

    assert!(app.turn(Ok("one\nfour\n".to_string()), &mut text, x, TODAY));
    assert_eq!(shown(&app), ["four"]);
}

#[test]
fn a_failed_save_goes_back_to_what_the_file_holds_and_says_so() {
    let mut app = app();
    let mut text = "one\ntwo\nthree\n".to_string();
    press(&mut app, "x");

    app.saved(
        Err(io::ErrorKind::PermissionDenied.into()),
        &mut text,
        Ok("one\ntwo\nthree\n".to_string()),
    );

    assert_eq!(lines(&app), ["one", "two", "three"]);
    assert_eq!(app.message.as_deref(), Some("could not save: permission denied (file left unchanged)"));
    assert!(app.refused);
    assert_eq!(text, "one\ntwo\nthree\n");

    app.saved(Err(io::ErrorKind::PermissionDenied.into()), &mut text, Ok("one\nfour\n".to_string()));
    assert_eq!(lines(&app), ["one", "four"]);
    assert_eq!(text, "one\ntwo\nthree\n");

    press(&mut app, "x");
    app.saved(
        Err(io::ErrorKind::PermissionDenied.into()),
        &mut text,
        Err(io::ErrorKind::InvalidData.into()),
    );
    assert_eq!(lines(&app), ["four"]);
    assert!(app.refused);

    app.saved(Ok("one\n".to_string()), &mut text, Ok(String::new()));
    assert_eq!(text, "one\n");
    assert_eq!(lines(&app), ["four"]);
}

#[test]
fn a_reload_drops_the_fold_of_a_group_gone_from_the_file() {
    let mut app = grouped();
    press(&mut app, "zM");

    app.reload(vec![Todo::from_line("(A) a"), Todo::from_line("d")]);
    app.reload(grouped().todos);

    assert_eq!(
        app.rows(),
        [Row::Group(Group::Priority('A')), Row::Task(3), Row::Group(Group::Unprioritised)]
    );
}

#[test]
fn a_turn_does_not_ask_to_save_over_a_file_it_could_not_read() {
    let mut app = app();
    let mut text = "one\ntwo\nthree\n".to_string();
    let x = Some(Event::Key(KeyEvent::from(KeyCode::Char('x'))));

    assert!(!app.turn(Err(io::ErrorKind::InvalidData.into()), &mut text, x, TODAY));

    assert!(app.refused);
    assert_eq!(text, "one\ntwo\nthree\n");
}

#[test]
fn a_reload_forgets_the_history() {
    let mut app = app();

    press(&mut app, "xxu");
    app.reload(vec![Todo::from_line("one")]);

    assert!(!press(&mut app, "u"));
    assert!(!redo(&mut app));
}

#[test]
fn the_groups_follow_the_priorities_on_screen_with_their_counts() {
    let mut app = app_of(&["(C) one", "two", "(A) three", "x 2026-09-20 four", "(A) five +work"]);
    use Group::*;

    assert_eq!(app.groups(), [(Priority('A'), 2), (Priority('C'), 1), (Unprioritised, 1)]);

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
fn a_paste_in_the_search_drops_the_folds_of_the_groups_it_takes_off_the_screen() {
    let mut app = app_of(&["(A) one", "two", "three"]);
    press(&mut app, "jza/");

    app.paste("t");

    assert_eq!(app.rows(), [Row::Task(2), Row::Task(3)]);
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
fn a_reload_drops_the_moves_still_waiting_with_the_history_they_belong_to() {
    let mut app = app();
    let mut text = "one\ntwo\nthree\n".to_string();
    let x = Some(Event::Key(KeyEvent::from(KeyCode::Char('x'))));

    assert!(!app.turn(Err(io::ErrorKind::PermissionDenied.into()), &mut text, x, TODAY));
    assert_eq!(moved(&app), ["+x 2026-09-26 one"]);
    app.turn(Ok("one\ntwo\nthree\nfour\n".to_string()), &mut text, None, TODAY);

    assert_eq!(lines(&app), ["one", "two", "three", "four"]);
    assert!(moved(&app).is_empty());
}

#[test]
fn a_save_failing_after_the_move_says_done_txt_holds_the_task() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("save-half");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\ntwo\n").unwrap();
    std::fs::write(&done, "").unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
    let mut app = app_of(&["one", "two"]);
    let mut text = "one\ntwo\n".to_string();

    press(&mut app, "x");
    app.save(&file, &mut text);

    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    let history = std::fs::read_to_string(&done).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(lines(&app), ["one", "two"]);
    assert_eq!(history, "x 2026-09-26 one\n");
    assert!(app.refused);
    let message = app.message.unwrap();
    assert!(
        message.starts_with("could not save: ") && message.ends_with(" (done.txt already holds the task)"),
        "{message}"
    );
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
fn save_takes_the_reopened_line_out_of_done_txt_among_lines_reading_the_same_and_u_sends_it_back_to_its_end() {
    let folder = folder("reopen");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::write(&done, "x 2026-09-20 dup\r\nx 2026-09-20 dup\n\nx 2026-09-21 other\nx 2026-09-20 dup\r\n").unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();
    let read = |path| std::fs::read_to_string(path).unwrap();

    app.refresh_history(&file);
    press(&mut app, "Hjjr");
    app.save(&file, &mut text);
    app.refresh_history(&file);
    let reopened = (read(&file), read(&done), shown(&app), app.cursor, app.message.clone());
    press(&mut app, "Hu");
    app.save(&file, &mut text);
    let undone = (read(&file), read(&done), app.message.clone());
    redo(&mut app);
    app.save(&file, &mut text);
    let redone = (read(&file), read(&done), app.message.clone());

    std::fs::remove_dir_all(&folder).unwrap();
    let left = "x 2026-09-20 dup\r\n\nx 2026-09-21 other\nx 2026-09-20 dup\r\n";
    let listed = ["x 2026-09-20 dup", "x 2026-09-21 other", "x 2026-09-20 dup"].map(String::from).to_vec();
    assert_eq!(reopened, ("one\ndup\n".into(), left.into(), listed, 2, Some("reopened".into())));
    assert_eq!(undone, ("one\n".into(), format!("{left}x 2026-09-20 dup\n"), Some("undone".into())));
    assert_eq!(redone, ("one\ndup\n".into(), left.into(), Some("redone".into())));
}

#[test]
fn a_reopen_writes_nothing_when_done_txt_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("reopen-refused");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::write(&done, "x 2026-09-20 old\n").unwrap();
    std::fs::set_permissions(&done, std::fs::Permissions::from_mode(0o444)).unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();
    let read = |path| std::fs::read_to_string(path).unwrap();

    app.refresh_history(&file);
    press(&mut app, "Hr");
    app.save(&file, &mut text);
    let refused = (
        lines(&app),
        read(&file),
        read(&done),
        app.refused,
        app.message.clone().unwrap_or_default(),
    );

    std::fs::remove_dir_all(&folder).unwrap();
    let message = refused.4.clone();
    assert!(
        message.starts_with("could not save: ") && message.ends_with(" (file left unchanged)"),
        "{message}"
    );
    assert_eq!(
        (refused.0, refused.1, refused.2, refused.3),
        (vec!["one".to_string()], "one\n".to_string(), "x 2026-09-20 old\n".to_string(), true)
    );
}

#[test]
fn a_reopened_line_ending_in_stray_carriage_returns_leaves_done_txt_and_comes_back_without_them() {
    let folder = folder("reopen-cr");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::write(&done, "x 2026-09-20 old\r\r\nx 2026-09-21 new\r").unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();
    let read = |path| std::fs::read_to_string(path).unwrap();

    app.refresh_history(&file);
    press(&mut app, "Hr");
    app.save(&file, &mut text);
    let last = (read(&file), read(&done), app.message.clone());
    app.refresh_history(&file);
    press(&mut app, "r");
    app.save(&file, &mut text);
    let first = (read(&file), read(&done), app.message.clone());

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(last, ("one\nnew\n".into(), "x 2026-09-20 old\r\r\n".into(), Some("reopened".into())));
    assert_eq!(first, ("one\nnew\nold\n".into(), String::new(), Some("reopened".into())));
}

#[test]
fn a_reopened_line_gone_from_done_txt_since_it_was_read_is_kept_and_said_so() {
    let folder = folder("reopen-gone");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::write(&done, "x 2026-09-20 old\nx 2026-09-21 new\n").unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();

    app.refresh_history(&file);
    std::fs::write(&done, "x 2026-09-20 old\n").unwrap();
    press(&mut app, "Hr");
    app.save(&file, &mut text);
    let files = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(files, ("one\nnew\n".to_string(), "x 2026-09-20 old\n".to_string()));
    assert_eq!(
        (app.message.as_deref(), app.refused),
        (Some("reopened, the task was no longer in done.txt"), false)
    );
}

#[test]
fn u_after_a_reopen_whose_line_could_not_leave_done_txt_does_not_add_it_there_again() {
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
    press(&mut app, "Hu");
    app.save(&file, &mut text);
    let undone = (read(&file), read(&locked.join("done.txt")));

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(held.2.starts_with("reopened, but done.txt still holds the task: "), "{}", held.2);
    assert_eq!((held.0, held.1), ("one\nold\n".to_string(), true));
    assert_eq!(undone, ("one\n".to_string(), "x 2026-09-20 old\n".to_string()));
}
