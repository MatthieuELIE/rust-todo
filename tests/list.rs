mod common;

use std::process::{Command, Output};

fn todo_list(terms: &[&str]) -> Output {
    let missing = std::env::temp_dir().join(format!("todo-{}-missing.txt", std::process::id()));
    Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .arg("list")
        .args(terms)
        .env("TODO_FILE", missing)
        .output()
        .unwrap()
}

#[test]
fn a_file_over_ten_mebibytes_is_refused_with_its_limit_and_left_unchanged() {
    let folder = common::folder("too-big");
    let file = folder.join("todo.txt");
    std::fs::write(&file, vec![b'a'; 10 * 1024 * 1024 + 1]).unwrap();

    let output = common::todo(&file, &["list"]);

    assert_eq!(output.status.code(), Some(1));
    let message = format!("could not read {}: larger than 10 MiB\n", file.display());
    assert_eq!(String::from_utf8_lossy(&output.stderr), message);
    assert_eq!(std::fs::metadata(&file).unwrap().len(), 10 * 1024 * 1024 + 1);
    std::fs::remove_dir_all(&folder).unwrap();
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
    let home = common::folder("home");
    std::fs::write(home.join("todo.txt"), "Call the bank\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .arg("list")
        .env_remove("TODO_FILE")
        .env("HOME", &*home)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&home).unwrap();

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  1  Call the bank\n");
}

#[test]
fn a_piped_listing_carries_no_colour_codes_nor_the_done_tasks() {
    let dir = common::folder("colour");
    let file = dir.join("todo.txt");
    std::fs::write(&file, "(A) Call the bank\nx 2026-09-03 Buy milk\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .arg("list")
        .env("TODO_FILE", &file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  1  (A) Call the bank\n");
}

#[test]
fn a_listing_replaces_the_control_characters_of_the_file() {
    let dir = common::folder("control");
    let file = dir.join("todo.txt");
    std::fs::write(&file, "Call \u{1b}]0;title\u{7}\u{1b}[2Jthe bank\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
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
fn a_listing_replaces_the_paragraph_separator_of_the_file() {
    let dir = common::folder("separator");
    let file = dir.join("todo.txt");
    std::fs::write(&file, "Call\u{2029}the bank\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .arg("list")
        .env("TODO_FILE", &file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  1  Call\u{fffd}the bank\n");
}

#[test]
fn due_keeps_the_pending_tasks_due_today_or_before() {
    let dir = common::folder("due");
    let file = dir.join("todo.txt");
    let lines =
        "Pay rent due:2020-01-01\nBuy milk due:2999-01-01\nx 2020-01-02 Call the bank due:2020-01-01\n(A) Fix the roof +house due:2020-06-01\n";
    std::fs::write(&file, lines).unwrap();
    let list = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_todo"))
            .env("XDG_CONFIG_HOME", "/nonexistent")
            .arg("list")
            .args(args)
            .env("TODO_FILE", &file)
            .output()
            .unwrap()
    };

    let due = list(&["--due"]);
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

#[test]
fn list_leaves_out_a_pending_task_whose_wake_up_date_is_after_today() {
    let folder = common::folder("sleeping");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay rent t:2999-01-01\nCall the bank\n").unwrap();

    let output = common::todo(&file, &["list"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "  2  Call the bank\n");
}

#[test]
fn the_latest_of_several_wake_up_dates_decides() {
    let folder = common::folder("sleeping-latest");
    let file = folder.join("todo.txt");
    std::fs::write(
        &file,
        "Pay rent t:2020-01-01 t:2999-01-01\nCall the bank t:2999-01-01 t:2020-01-01\nBuy milk\n",
    )
    .unwrap();

    let output = common::todo(&file, &["list"]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  3  Buy milk\n");
}

#[test]
fn an_unreadable_wake_up_date_leaves_the_task_listed_and_is_accepted_when_added() {
    let folder = common::folder("sleeping-unreadable");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "").unwrap();

    let added = [
        common::todo(&file, &["add", "2026-11-01 Pay rent t:demain"]),
        common::todo(&file, &["add", "2026-11-01 Buy milk t:2026-99-99"]),
        common::todo(&file, &["add", "2026-11-01 Call the bank t:2999-01-01"]),
    ];
    let output = common::todo(&file, &["list"]);

    assert!(added.iter().all(|output| output.status.success()));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "  1  2026-11-01 Pay rent t:demain\n  2  2026-11-01 Buy milk t:2026-99-99\n"
    );
}

#[test]
fn list_with_terms_leaves_out_the_sleeping_tasks_and_lists_a_woken_one() {
    let folder = common::folder("sleeping-terms");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay rent +home t:2999-01-01\nBuy milk +home t:2020-01-01\nCall the bank\n").unwrap();

    let all = common::todo(&file, &["list"]);
    let narrowed = common::todo(&file, &["list", "+home"]);
    let none = common::todo(&file, &["list", "rent"]);

    assert_eq!(
        String::from_utf8_lossy(&all.stdout),
        "  2  Buy milk +home t:2020-01-01\n  3  Call the bank\n"
    );
    assert_eq!(String::from_utf8_lossy(&narrowed.stdout), "  2  Buy milk +home t:2020-01-01\n");
    assert!(none.status.success());
    assert!(none.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&none.stderr), "no matching task\n");
}

#[test]
fn due_leaves_out_a_sleeping_task_even_when_it_is_due() {
    let folder = common::folder("sleeping-due");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay rent t:2999-01-01 due:2020-01-01\nCall the bank due:2020-01-01\n").unwrap();

    let output = common::todo(&file, &["list", "--due"]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  2  Call the bank due:2020-01-01\n");
}

#[test]
fn done_lists_a_done_line_whose_wake_up_date_is_after_today() {
    let folder = common::folder("sleeping-done");
    std::fs::write(folder.join("todo.txt"), "").unwrap();
    std::fs::write(folder.join("done.txt"), "x 2026-11-01 Pay rent t:2999-01-01\n").unwrap();

    let output = common::todo(&folder.join("todo.txt"), &["list", "--done"]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "  1  x 2026-11-01 Pay rent t:2999-01-01\n");
}

#[test]
fn sleeping_lists_only_the_sleeping_tasks_narrowed_by_terms() {
    let folder = common::folder("sleeping-flag");
    let file = folder.join("todo.txt");
    let lines = "Pay rent +home t:2999-01-01\nBuy milk +home t:2020-01-01\n(A) Call the bank t:2999-01-01\nx 2026-11-01 Water plants t:2999-01-01\n";
    std::fs::write(&file, lines).unwrap();

    let all = common::todo(&file, &["list", "--sleeping"]);
    let narrowed = common::todo(&file, &["list", "--sleeping", "+home"]);
    let none = common::todo(&file, &["list", "--sleeping", "+garden"]);
    std::fs::write(&file, "Buy milk t:2020-01-01\n").unwrap();
    let awake = common::todo(&file, &["list", "--sleeping"]);

    assert_eq!(
        String::from_utf8_lossy(&all.stdout),
        "  3  (A) Call the bank t:2999-01-01\n  1  Pay rent +home t:2999-01-01\n"
    );
    assert_eq!(String::from_utf8_lossy(&narrowed.stdout), "  1  Pay rent +home t:2999-01-01\n");
    assert_eq!(String::from_utf8_lossy(&none.stderr), "no matching task\n");
    assert!(awake.status.success());
    assert!(awake.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&awake.stderr), "nothing sleeping\n");
}

#[test]
fn sleeping_is_refused_with_due_and_with_done() {
    let folder = common::folder("sleeping-refused");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "Pay rent t:2999-01-01\n").unwrap();

    let outputs = [
        common::todo(&file, &["list", "--sleeping", "--due"]),
        common::todo(&file, &["list", "--done", "--sleeping"]),
    ];

    for output in outputs {
        let error = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(error.contains("--sleeping") && error.contains("cannot be used with"), "{error}");
    }
}

#[test]
fn done_lists_done_txt_alone_in_file_order_numbered_by_position() {
    let folder = common::folder("list-done");
    std::fs::write(folder.join("todo.txt"), "Buy milk +home\n").unwrap();
    let history = "x 2026-09-01 Pay rent +home\nx 2026-09-02 Call the bank\n(A) Stray line +home\n";
    std::fs::write(folder.join("done.txt"), history).unwrap();
    let list = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_todo"))
            .env("XDG_CONFIG_HOME", "/nonexistent")
            .arg("list")
            .args(args)
            .env("TODO_FILE", folder.join("todo.txt"))
            .output()
            .unwrap()
    };

    let (all, narrowed, none) = (list(&["--done"]), list(&["--done", "+home"]), list(&["--done", "+garden"]));

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&all.stdout),
        "  1  x 2026-09-01 Pay rent +home\n  2  x 2026-09-02 Call the bank\n  3  (A) Stray line +home\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&narrowed.stdout),
        "  1  x 2026-09-01 Pay rent +home\n  3  (A) Stray line +home\n"
    );
    assert!(none.status.success());
    assert_eq!(String::from_utf8_lossy(&none.stderr), "no matching task\n");
}

#[test]
fn done_without_a_done_txt_is_an_empty_list() {
    let folder = common::folder("list-done-absent");
    std::fs::write(folder.join("todo.txt"), "Buy milk\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .args(["list", "--done"])
        .env("TODO_FILE", folder.join("todo.txt"))
        .output()
        .unwrap();

    std::fs::remove_dir_all(&folder).unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing done\n");
}

#[test]
fn done_reads_done_txt_only_and_fails_when_it_cannot_be_read() {
    let folder = common::folder("list-done-unreadable");
    std::fs::write(folder.join("todo.txt"), b"\xff\n").unwrap();
    std::fs::write(folder.join("done.txt"), "x 2026-09-01 Pay rent\n").unwrap();
    let list = || {
        Command::new(env!("CARGO_BIN_EXE_todo"))
            .env("XDG_CONFIG_HOME", "/nonexistent")
            .args(["list", "--done"])
            .env("TODO_FILE", folder.join("todo.txt"))
            .output()
            .unwrap()
    };

    let listed = list();
    std::fs::write(folder.join("done.txt"), b"\xff\n").unwrap();
    let refused = list();

    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(String::from_utf8_lossy(&listed.stdout), "  1  x 2026-09-01 Pay rent\n");
    let error = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(error.starts_with("could not read ") && error.contains("done.txt"), "{error}");
}

#[test]
fn the_former_all_flag_is_refused_rather_than_read_as_a_term() {
    let dir = common::folder("all");
    let file = dir.join("todo.txt");
    std::fs::write(&file, "Call the bank\n").unwrap();
    let list = |flag: &str| {
        Command::new(env!("CARGO_BIN_EXE_todo"))
            .env("XDG_CONFIG_HOME", "/nonexistent")
            .args(["list", flag])
            .env("TODO_FILE", &file)
            .output()
            .unwrap()
    };

    let outputs = [list("--all"), list("-a")];

    std::fs::remove_file(&file).unwrap();

    for output in outputs {
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            "--all is gone: list shows the pending tasks, list --done the done ones\n"
        );
    }
}
