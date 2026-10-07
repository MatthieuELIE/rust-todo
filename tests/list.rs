use std::process::{Command, Output};

fn todo_list(terms: &[&str]) -> Output {
    let missing = std::env::temp_dir().join(format!("todo-{}-missing.txt", std::process::id()));
    Command::new(env!("CARGO_BIN_EXE_todo"))
        .arg("list")
        .args(terms)
        .env("TODO_FILE", missing)
        .output()
        .unwrap()
}

#[test]
fn an_empty_list_says_so_on_stderr_and_still_succeeds() {
    let output = todo_list(&[]);

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing to do\n");
}

#[test]
fn an_empty_filtered_list_says_nothing_matched() {
    let output = todo_list(&["+work"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stderr), "no matching task\n");
}

#[test]
fn without_todo_file_the_list_is_todo_txt_in_the_home_folder() {
    let home = std::env::temp_dir().join(format!("todo-{}-home", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("todo.txt"), "Call the bank\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .arg("list")
        .env_remove("TODO_FILE")
        .env("HOME", &home)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&home).unwrap();

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  1  Call the bank\n");
}

#[test]
fn a_piped_listing_carries_no_colour_codes() {
    let file = std::env::temp_dir().join(format!("todo-{}-colour.txt", std::process::id()));
    std::fs::write(&file, "(A) Call the bank\nx 2026-09-03 Buy milk\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .args(["list", "--all"])
        .env("TODO_FILE", &file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "  1  (A) Call the bank\n  2  x 2026-09-03 Buy milk\n"
    );
}

#[test]
fn a_listing_replaces_the_control_characters_of_the_file() {
    let file = std::env::temp_dir().join(format!("todo-{}-control.txt", std::process::id()));
    std::fs::write(&file, "Call \u{1b}]0;title\u{7}\u{1b}[2Jthe bank\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .arg("list")
        .env("TODO_FILE", &file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "  1  Call \u{fffd}]0;title\u{fffd}\u{fffd}[2Jthe bank\n"
    );
}

#[test]
fn due_keeps_the_pending_tasks_due_today_or_before() {
    let file = std::env::temp_dir().join(format!("todo-{}-due.txt", std::process::id()));
    let lines =
        "Pay rent due:2020-01-01\nBuy milk due:2999-01-01\nx 2020-01-02 Call the bank due:2020-01-01\n(A) Fix the roof +house due:2020-06-01\n";
    std::fs::write(&file, lines).unwrap();
    let list = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_todo"))
            .arg("list")
            .args(args)
            .env("TODO_FILE", &file)
            .output()
            .unwrap()
    };

    let due = list(&["--all", "--due"]);
    let narrowed = list(&["--due", "+house"]);
    let none = list(&["--due", "+garden"]);
    std::fs::write(&file, "Buy milk due:2999-01-01\n").unwrap();
    let later = list(&["--due"]);

    std::fs::remove_file(&file).unwrap();
    assert!(later.status.success());
    assert!(later.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&later.stderr), "nothing due\n");
    assert_eq!(
        String::from_utf8_lossy(&due.stdout),
        "  4  (A) Fix the roof +house due:2020-06-01\n  1  Pay rent due:2020-01-01\n"
    );
    assert_eq!(String::from_utf8_lossy(&narrowed.stdout), "  4  (A) Fix the roof +house due:2020-06-01\n");
    assert_eq!(String::from_utf8_lossy(&none.stderr), "no matching task\n");
}
