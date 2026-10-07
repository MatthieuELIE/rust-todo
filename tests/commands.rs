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
    let file = std::env::temp_dir().join(format!("todo-{}-commands.txt", std::process::id()));
    std::fs::write(&file, "2026-09-01 Call the bank\nPay rent\n").unwrap();

    assert!(todo(&file, &["add", "(B) 2026-09-02 Buy milk"]).status.success());
    assert!(todo(&file, &["do", "1"]).status.success());
    assert!(todo(&file, &["rm", "2"]).status.success());
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

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    for (output, error) in refused {
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), error);
    }
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    let completed = lines[0].strip_prefix("x ").expect("task 1 is done");
    assert_eq!(&completed[10..], " 2026-09-01 Call the bank");
    assert_eq!(lines[1], "(B) 2026-09-02 Buy milk");
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
fn reopen_makes_a_done_task_pending_again_without_its_completion_date() {
    let file = std::env::temp_dir().join(format!("todo-{}-reopen.txt", std::process::id()));
    std::fs::write(&file, "x 2026-09-03 2026-09-02 Pay rent\nBuy milk\nx x Sell it\n").unwrap();

    assert!(todo(&file, &["reopen", "1"]).status.success());
    let refused = [
        (todo(&file, &["reopen", "2"]), "task 2 is not done\n"),
        (todo(&file, &["reopen", "3"]), "task 3 cannot be reopened: its text starts with x\n"),
        (todo(&file, &["reopen", "9"]), "no task numbered 9\n"),
    ];

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    for (output, error) in refused {
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), error);
    }
    assert_eq!(text, "2026-09-02 Pay rent\nBuy milk\nx x Sell it\n");
}
