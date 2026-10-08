use std::path::Path;
use std::process::{Command, Output};

fn todo(file: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_todo"))
        .args(args)
        .env("TODO_FILE", file)
        .output()
        .unwrap()
}

#[test]
fn add_do_and_remove_rewrite_the_file_and_a_refused_command_leaves_it_alone() {
    let folder = folder("commands");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "2026-09-01 Call the bank\nPay rent\nx 2026-08-30 Typed by hand\n").unwrap();
    std::fs::write(&done, "x 2026-08-01 Water plants\n").unwrap();

    let said = [
        (todo(&file, &["add", "(B) 2026-09-02 Buy milk"]), ""),
        (todo(&file, &["edit", "2", "(C) Pay rent"]), ""),
        (todo(&file, &["do", "1"]), "done: 2026-09-01 Call the bank\n"),
        (todo(&file, &["rm", "1"]), "removed: (C) Pay rent\n"),
    ];
    let refused = [
        (todo(&file, &["add", "x Buy bread"]), "cannot add a task that is already done\n"),
        (
            todo(&file, &["add", "Buy bread\nx 2026-01-01 Sell it"]),
            "a task is one line, without control characters\n",
        ),
        (todo(&file, &["do", "9"]), "no task numbered 9\n"),
        (todo(&file, &["do", "0"]), "no task numbered 0\n"),
        (todo(&file, &["rm", "0"]), "no task numbered 0\n"),
        (todo(&file, &["do", "1"]), "task 1 is already done\n"),
        (todo(&file, &["rm", "9"]), "no task numbered 9\n"),
    ];

    let (text, history) = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    std::fs::remove_dir_all(&folder).unwrap();
    for (output, confirmation) in said {
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), confirmation);
    }
    for (output, error) in refused {
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&output.stderr), error);
    }
    assert_eq!(text, "x 2026-08-30 Typed by hand\n(B) 2026-09-02 Buy milk\n");
    let lines: Vec<&str> = history.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "x 2026-08-01 Water plants");
    let completed = lines[1].strip_prefix("x ").expect("the task moved is done");
    assert_eq!(&completed[10..], " 2026-09-01 Call the bank");
}

#[test]
fn a_file_that_cannot_be_read_is_left_as_it_is() {
    let file = std::env::temp_dir().join(format!("todo-{}-unreadable.txt", std::process::id()));
    std::fs::write(&file, b"Pay rent\n\xff\n").unwrap();

    let output = todo(&file, &["add", "Buy milk"]);

    let bytes = std::fs::read(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not read "));
    assert_eq!(bytes, b"Pay rent\n\xff\n");
}

#[test]
fn add_stamps_an_undated_task_with_the_day() {
    let file = std::env::temp_dir().join(format!("todo-{}-stamp.txt", std::process::id()));
    let _ = std::fs::remove_file(&file);

    assert!(todo(&file, &["add", "Buy bread"]).status.success());

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    let (date, rest) = text.split_at(10);
    assert_eq!(rest, " Buy bread\n");
    let dated = date
        .bytes()
        .enumerate()
        .all(|(i, byte)| if i == 4 || i == 7 { byte == b'-' } else { byte.is_ascii_digit() });
    assert!(dated, "{date}");
}

#[test]
fn a_file_that_cannot_be_saved_is_left_as_it_is() {
    use std::os::unix::fs::PermissionsExt;
    let file = std::env::temp_dir().join(format!("todo-{}-unsaved.txt", std::process::id()));
    let _ = std::fs::remove_file(&file);
    std::fs::write(&file, "Pay rent\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["add", "Buy milk"]);

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        error.starts_with("could not save ") && error.ends_with(" (file left unchanged)\n"),
        "{error}"
    );
    assert_eq!(text, "Pay rent\n");
}

#[test]
fn edit_replaces_a_pending_task_and_keeps_its_creation_date() {
    let file = std::env::temp_dir().join(format!("todo-{}-edit.txt", std::process::id()));
    let before = "(C) 2026-09-01 Call the bank\nx 2026-09-03 2026-09-02 Pay rent\n\nBuy milk\n(B) 2026-09-05 Water plants\n";
    std::fs::write(&file, before).unwrap();

    let unchanged = todo(&file, &["edit", "3", "Buy milk"]);
    let refused = [
        (todo(&file, &["edit", "9", "Buy bread"]), "no task numbered 9\n"),
        (todo(&file, &["edit", "2", "Pay the rent"]), "task 2 is done\n"),
        (todo(&file, &["edit", "1", "x Call the bank"]), "cannot edit a task into a done one\n"),
        (todo(&file, &["edit", "1", "(A) "]), "a task needs a description\n"),
    ];
    let untouched = std::fs::read_to_string(&file).unwrap();
    assert!(todo(&file, &["edit", "1", "(A) Call the @bank due:2026-10-09"]).status.success());
    assert!(todo(&file, &["edit", "4", "2026-10-01 Water the plants"]).status.success());

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    for (output, error) in refused {
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), error);
    }
    assert!(unchanged.status.success());
    assert!(unchanged.stderr.is_empty());
    assert_eq!(untouched, before);
    assert_eq!(
        text,
        "(A) 2026-09-01 Call the @bank due:2026-10-09\nx 2026-09-03 2026-09-02 Pay rent\nBuy milk\n2026-10-01 Water the plants\n"
    );
}

#[test]
fn reopen_moves_a_task_of_done_txt_to_the_end_of_the_task_file_without_its_completion_date() {
    let folder = folder("reopen");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Buy milk\n").unwrap();
    let history = "x 2026-09-03 2026-09-02 Pay rent\n\nStray line\nx x Sell it\nx (A) 2026-09-05 Kept as written\nx 2026-09-06 \n";
    std::fs::write(&done, history).unwrap();

    let refused = [
        (todo(&file, &["reopen", "2"]), "task 2 is not done\n"),
        (todo(&file, &["reopen", "3"]), "task 3 cannot be reopened: its text starts with x\n"),
        (todo(&file, &["reopen", "5"]), "task 5 cannot be reopened: it has no description\n"),
        (todo(&file, &["reopen", "9"]), "no task numbered 9\n"),
        (todo(&file, &["reopen", "0"]), "no task numbered 0\n"),
    ];
    let untouched = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    let reopened = todo(&file, &["reopen", "1"]);

    let (text, left) = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    std::fs::remove_dir_all(&folder).unwrap();
    for (output, error) in refused {
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), error);
    }
    assert!(reopened.status.success());
    assert_eq!(String::from_utf8_lossy(&reopened.stdout), "reopened: 2026-09-02 Pay rent\n");
    assert_eq!(untouched, ("Buy milk\n".to_string(), history.to_string()));
    assert_eq!(text, "Buy milk\n2026-09-02 Pay rent\n");
    assert_eq!(left, "\nStray line\nx x Sell it\nx (A) 2026-09-05 Kept as written\nx 2026-09-06 \n");
}

fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("todo-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    folder
}

#[test]
fn archive_moves_the_done_tasks_to_the_end_of_done_txt_in_file_order() {
    let folder = folder("archive");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "x 2026-09-03 Pay rent\nBuy milk\nx 2026-09-04 Call the bank\n").unwrap();
    std::fs::write(&done, "x 2026-09-01 Water plants").unwrap();

    let output = todo(&file, &["archive"]);

    let (text, archived) = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2 tasks archived\n");
    assert_eq!(text, "Buy milk\n");
    assert_eq!(archived, "x 2026-09-01 Water plants\nx 2026-09-03 Pay rent\nx 2026-09-04 Call the bank\n");
}

#[test]
fn archive_with_no_done_task_says_so_and_writes_no_file() {
    let folder = folder("archive-nothing");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Buy milk\n\nPay rent\n").unwrap();

    let output = todo(&file, &["archive"]);

    let text = std::fs::read_to_string(&file).unwrap();
    let created = folder.join("done.txt").exists();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing to archive\n");
    assert_eq!(text, "Buy milk\n\nPay rent\n");
    assert!(!created);
}

#[test]
fn archive_creates_a_missing_done_txt_readable_by_its_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("archive-absent");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Buy milk\nx 2026-09-03 Pay rent\n").unwrap();

    let output = todo(&file, &["archive"]);

    let (text, archived) = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    let mode = std::fs::metadata(&done).unwrap().permissions().mode() & 0o777;
    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1 task archived\n");
    assert_eq!(text, "Buy milk\n");
    assert_eq!(archived, "x 2026-09-03 Pay rent\n");
    assert_eq!(mode, 0o600);
}

#[test]
fn archive_reopen_and_do_refuse_a_task_file_that_is_done_txt_itself() {
    let folder = folder("archive-itself");
    let file = folder.join("done.txt");
    std::fs::write(&file, "x 2026-09-03 Pay rent\nBuy milk\n").unwrap();

    let outputs = [todo(&file, &["archive"]), todo(&file, &["reopen", "1"]), todo(&file, &["do", "2"])];

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    for output in outputs {
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!("the task file is done.txt itself: {}\n", file.display())
        );
    }
    assert_eq!(text, "x 2026-09-03 Pay rent\nBuy milk\n");
}

#[test]
fn archive_adds_no_blank_line_after_a_done_txt_ending_with_a_line_break() {
    let folder = folder("archive-twice");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "x 2026-09-03 Pay rent\n").unwrap();
    std::fs::write(&done, "x 2026-09-01 Water plants\n").unwrap();

    assert!(todo(&file, &["archive"]).status.success());

    let archived = std::fs::read_to_string(&done).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(archived, "x 2026-09-01 Water plants\nx 2026-09-03 Pay rent\n");
}

#[test]
fn archive_leaves_the_task_file_alone_when_done_txt_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("archive-readonly");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "x 2026-09-03 Pay rent\nBuy milk\n").unwrap();
    std::fs::write(&done, "").unwrap();
    std::fs::set_permissions(&done, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["archive"]);

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
    assert_eq!(text, "x 2026-09-03 Pay rent\nBuy milk\n");
}

#[test]
fn archive_does_not_announce_a_count_when_the_task_file_cannot_be_saved() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("archive-unsaved");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "x 2026-09-03 Pay rent\nBuy milk\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["archive"]);

    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
}

#[test]
fn reopen_leaves_done_txt_alone_when_the_task_file_cannot_be_saved() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("reopen-unsaved");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Buy milk\n").unwrap();
    std::fs::write(&done, "x 2026-09-03 Pay rent\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["reopen", "1"]);

    let left = std::fs::read_to_string(&done).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
    assert_eq!(left, "x 2026-09-03 Pay rent\n");
}

#[test]
fn reopen_writes_nothing_when_done_txt_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("reopen-readonly");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Buy milk\n").unwrap();
    std::fs::write(&done, "x 2026-09-03 Pay rent\n").unwrap();
    std::fs::set_permissions(&done, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["reopen", "1"]);

    let (text, left) = (std::fs::read_to_string(&file).unwrap(), std::fs::read_to_string(&done).unwrap());
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
    assert_eq!(text, "Buy milk\n");
    assert_eq!(left, "x 2026-09-03 Pay rent\n");
}

#[test]
fn do_leaves_the_task_file_alone_when_done_txt_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("do-readonly");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Pay rent\nBuy milk\n").unwrap();
    std::fs::write(&done, "").unwrap();
    std::fs::set_permissions(&done, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["do", "1"]);

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
    assert_eq!(text, "Pay rent\nBuy milk\n");
}

#[test]
fn do_writes_nothing_when_the_task_file_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let folder = folder("do-unsaved");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay rent\nBuy milk\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = todo(&file, &["do", "1"]);

    let created = folder.join("done.txt").exists();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("could not save "));
    assert!(!created);
}
