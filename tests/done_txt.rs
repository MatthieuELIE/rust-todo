mod common;

use common::{folder, todo};

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
        (
            todo(&file, &["reopen", "5"]),
            "task 5 cannot be reopened: it is only +projects, @contexts and key: words\n",
        ),
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

#[test]
fn reopen_drops_the_priority_of_a_line_written_with_one() {
    let folder = folder("reopen-priority");
    let file = folder.join("todo.txt");
    std::fs::write(&file, "").unwrap();
    std::fs::write(folder.join("done.txt"), "x (A) Pay rent\n").unwrap();

    let reopened = todo(&file, &["reopen", "1"]);

    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();
    assert!(reopened.status.success());
    assert_eq!(text, "Pay rent\n");
}

#[test]
fn do_writes_the_task_to_done_txt_without_its_wait() {
    let folder = folder("do-wait");
    let (file, done) = (folder.join("todo.txt"), folder.join("done.txt"));
    std::fs::write(&file, "Mail wait:designer +work\n").unwrap();

    todo(&file, &["do", "1"]);

    let written = std::fs::read_to_string(&done).unwrap();
    assert!(written.trim_end().ends_with("Mail +work") && !written.contains("wait:"), "{written}");
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
