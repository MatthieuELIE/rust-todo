use super::super::tests::{TODAY, app, app_of, folder, key, lines, moved, names, press, redo, said, shown, told};
use super::super::*;

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
    let undone = (read(&file), read(&done), told(&app));
    press(&mut app, "x");
    app.save(&file, &mut text);
    std::fs::write(&done, "x 2026-09-01 zero\n").unwrap();
    press(&mut app, "u");
    app.save(&file, &mut text);
    let missing = (read(&file), read(&done), told(&app), app.refusal_shown());

    std::fs::remove_dir_all(&folder).unwrap();
    let history = "x 2026-09-01 zero\nx 2026-09-26 two\n";
    assert_eq!(completed, ("one\n".into(), format!("{history}x 2026-09-26 two\n"), "one\n".into()));
    assert_eq!(undone, ("one\n(A) two\n".into(), history.into(), Some(Notice::ChangeUndone)));
    let restored = ("one\n(A) two\n".into(), "x 2026-09-01 zero\n".into());
    assert_eq!(missing, (restored.0, restored.1, Some(Notice::GoneFromDone { reopened: false }), false));
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
    let refused = (lines(&app), app.refusal_shown(), std::fs::read_to_string(&file).unwrap());
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
        said(&app).as_deref(),
        Some("Could not save: the task file is done.txt itself (file left unchanged)")
    );
}

#[test]
fn a_reload_keeps_the_cursor_row_and_says_so_through_the_next_key() {
    let mut app = app();
    press(&mut app, "G");

    app.reload(vec![Todo::from_line("one"), Todo::from_line("four")]);

    assert_eq!(shown(&app), ["one", "four"]);
    assert_eq!(app.cursor, 1);
    assert_eq!(told(&app), Some(Notice::Reloaded));
    press(&mut app, "k");
    assert_eq!(told(&app), Some(Notice::Reloaded));
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
    assert!(app.refusal_shown());
    assert!(said(&app).unwrap().starts_with("Undone, but done.txt still holds the task: "));
}

#[test]
fn a_reload_after_a_refusal_is_not_told_as_one() {
    let mut app = app();
    app.notify(Notice::HistoryReadOnly);

    app.reload(vec![Todo::from_line("one")]);

    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::Reloaded), false));
}

#[test]
fn the_history_is_what_done_txt_holds_and_a_done_txt_that_cannot_be_read_is_told() {
    let mut app = app();

    app.set_history(Ok("x old\n\nx new\n".to_string()));
    assert_eq!(app.history.iter().map(Todo::to_line).collect::<Vec<_>>(), ["x old", "x new"]);
    assert_eq!(told(&app), None);

    app.set_history(Err(io::ErrorKind::InvalidData.into()));
    assert!(app.history.is_empty());
    assert!(app.refusal_shown());
    assert!(said(&app).unwrap().starts_with("Could not read done.txt: "));
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
    assert_eq!(said(&app).as_deref(), Some("Could not save: permission denied (file left unchanged)"));
    assert!(app.refusal_shown());
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
    assert!(app.refusal_shown());

    app.saved(Ok("one\n".to_string()), &mut text, Ok(String::new()));
    assert_eq!(text, "one\n");
    assert_eq!(lines(&app), ["four"]);
}

#[test]
fn a_turn_does_not_ask_to_save_over_a_file_it_could_not_read() {
    let mut app = app();
    let mut text = "one\ntwo\nthree\n".to_string();
    let x = Some(Event::Key(KeyEvent::from(KeyCode::Char('x'))));

    assert!(!app.turn(Err(io::ErrorKind::InvalidData.into()), &mut text, x, TODAY));

    assert!(app.refusal_shown());
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
    assert!(app.refusal_shown());
    let message = said(&app).unwrap();
    assert!(
        message.starts_with("Could not save: ") && message.ends_with(" (done.txt already holds the task)"),
        "{message}"
    );
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
    let reopened = (read(&file), read(&done), shown(&app), app.cursor, told(&app));
    press(&mut app, "Hu");
    app.save(&file, &mut text);
    let undone = (read(&file), read(&done), told(&app));
    redo(&mut app);
    app.save(&file, &mut text);
    let redone = (read(&file), read(&done), told(&app));

    std::fs::remove_dir_all(&folder).unwrap();
    let left = "x 2026-09-20 dup\r\n\nx 2026-09-21 other\nx 2026-09-20 dup\r\n";
    let listed = ["x 2026-09-20 dup", "x 2026-09-21 other", "x 2026-09-20 dup"].map(String::from).to_vec();
    assert_eq!(reopened, ("one\ndup\n".into(), left.into(), listed, 2, Some(Notice::Reopened)));
    assert_eq!(undone, ("one\n".into(), format!("{left}x 2026-09-20 dup\n"), Some(Notice::ChangeUndone)));
    assert_eq!(redone, ("one\ndup\n".into(), left.into(), Some(Notice::ChangeRedone)));
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
    let refused = (lines(&app), read(&file), read(&done), app.refusal_shown(), said(&app).unwrap_or_default());

    std::fs::remove_dir_all(&folder).unwrap();
    let message = refused.4.clone();
    assert!(
        message.starts_with("Could not save: ") && message.ends_with(" (file left unchanged)"),
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
    let last = (read(&file), read(&done), told(&app));
    app.refresh_history(&file);
    press(&mut app, "r");
    app.save(&file, &mut text);
    let first = (read(&file), read(&done), told(&app));

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(last, ("one\nnew\n".into(), "x 2026-09-20 old\r\r\n".into(), Some(Notice::Reopened)));
    assert_eq!(first, ("one\nnew\nold\n".into(), String::new(), Some(Notice::Reopened)));
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
    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::GoneFromDone { reopened: true }), false));
}

#[test]
fn a_refusal_to_take_a_line_out_of_done_txt_is_not_replaced_by_a_later_removal_of_the_same_save() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("held-then-gone");
    let (file, locked) = (folder.join("todo.txt"), folder.join("locked"));
    std::fs::write(&file, "one\ntwo\n").unwrap();
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("done.txt"), "").unwrap();
    std::os::unix::fs::symlink(locked.join("done.txt"), folder.join("done.txt")).unwrap();
    let mut app = app_of(&["one", "two"]);
    let mut text = "one\ntwo\n".to_string();
    let key = || Some(Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE)));

    press(&mut app, "x");
    app.save(&file, &mut text);
    press(&mut app, "x");
    app.save(&file, &mut text);
    let kept = std::fs::read_to_string(locked.join("done.txt"))
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .to_string();
    std::fs::write(locked.join("done.txt"), format!("{kept}\n")).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    app.turn(Err(io::ErrorKind::PermissionDenied.into()), &mut text, key(), TODAY);
    let saves = app.turn(Ok(text.clone()), &mut text.clone(), key(), TODAY);
    app.save(&file, &mut text);

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(saves);
    assert!(app.refusal_shown());
    assert!(said(&app).unwrap().starts_with("Undone, but done.txt still holds the task: "));
}

#[test]
fn a_second_refusal_to_take_a_line_out_of_done_txt_in_the_same_save_does_not_replace_the_first() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("held-twice");
    let (file, locked) = (folder.join("todo.txt"), folder.join("locked"));
    std::fs::write(&file, "one\n").unwrap();
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("done.txt"), "x 2026-09-20 old\n").unwrap();
    std::os::unix::fs::symlink(locked.join("done.txt"), folder.join("done.txt")).unwrap();
    let mut app = app_of(&["one"]);
    let mut text = "one\n".to_string();
    let key = |c| Some(Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)));

    press(&mut app, "x");
    app.save(&file, &mut text);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    app.turn(Err(io::ErrorKind::PermissionDenied.into()), &mut text, key('u'), TODAY);
    press(&mut app, "H");
    app.refresh_history(&file);
    press(&mut app, "j");
    let saves = app.turn(Ok(text.clone()), &mut text.clone(), key('r'), TODAY);
    app.save(&file, &mut text);

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(saves);
    assert!(said(&app).unwrap().starts_with("Undone, but done.txt still holds the task: "));
}

#[test]
fn copied_says_copied_in_grey_or_why_it_could_not_in_red() {
    let mut app = app();
    app.copied(Ok(()));
    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::Copied), false));
    app.copied(Err(io::Error::other("boom")));
    assert_eq!(
        (told(&app), app.refusal_shown()),
        (Some(Notice::Refused("could not copy: boom".to_string())), true)
    );
}
