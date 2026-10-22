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

pub(super) fn popup(app: &App) -> &Popup {
    match &app.focus {
        Focus::Popup(popup) => popup,
        _ => panic!("the popup is closed"),
    }
}

pub(super) fn shown(app: &App) -> Vec<String> {
    app.tasks().into_iter().map(|(_, todo)| todo.to_line()).collect()
}

pub(super) fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("todo-{}-tui-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    folder
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

pub(super) fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(KeyEvent::new(code, modifiers), TODAY);
}

pub(super) fn names(app: &App) -> Vec<String> {
    app.completions().0.iter().map(|(name, count)| format!("{name} {count}")).collect()
}

pub(super) fn redo(app: &mut App) -> bool {
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL), TODAY)
}

pub(super) fn lines(app: &App) -> Vec<String> {
    app.todos.iter().map(Todo::to_line).collect()
}

#[test]
fn a_control_key_in_the_search_types_nothing() {
    let mut app = app();

    press(&mut app, "/t");
    app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL), TODAY);

    assert_eq!(app.search, "t");
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
