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

    assert!(todo(&file, &["add", "2026-09-02 Buy milk", "--priority", "B"]).status.success());
    assert!(todo(&file, &["do", "1"]).status.success());
    assert!(todo(&file, &["rm", "2"]).status.success());
    let refused = [
        (todo(&file, &["add", "(A) Buy bread", "--priority", "B"]), "priority given twice\n"),
        (todo(&file, &["do", "9"]), "no task numbered 9\n"),
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
