use super::*;
use crate::guard::Folder;
use std::path::PathBuf;

fn folder(name: &str) -> Folder {
    Folder::new(&format!("repository-{name}"))
}

#[test]
fn read_refuses_a_file_over_ten_mebibytes_and_accepts_one_of_exactly_ten() {
    let dir = folder("limit");
    let path = dir.join("limit");
    std::fs::write(&path, vec![b'a'; 10 * 1024 * 1024]).unwrap();
    assert_eq!(read(&path).unwrap().len(), 10 * 1024 * 1024);

    std::fs::write(&path, vec![b'a'; 10 * 1024 * 1024 + 1]).unwrap();
    assert_eq!(read(&path).unwrap_err().to_string(), "larger than 10 MiB");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn save_then_load_round_trips_the_list() {
    let dir = folder("roundtrip");
    let path = dir.join("roundtrip");
    let todos = vec![Todo::from_line("(A) Lorem ipsum dolor"), Todo::from_line("x Consectetur adipiscing")];

    let written = save(&path, &todos).unwrap();
    let (text, loaded) = load(&path).unwrap();

    let lines: Vec<String> = loaded.iter().map(Todo::to_line).collect();
    assert_eq!(lines, ["(A) Lorem ipsum dolor", "x Consectetur adipiscing"]);
    assert_eq!(text, written);
}

#[test]
fn save_writes_through_a_symlink_and_keeps_the_file_mode() {
    let dir = folder("target");
    let (target, link) = (dir.join("target"), dir.join("link"));
    std::fs::write(&target, "Lorem ipsum\n").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&target, &link).unwrap();

    save(&link, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "Consectetur adipiscing\n");
    assert_eq!(std::fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn save_drops_the_group_and_other_bits_of_an_open_file() {
    let dir = folder("open-save");
    let path = dir.join("open-save");
    std::fs::write(&path, "Lorem ipsum\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    save(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn append_drops_the_group_and_other_bits_of_an_open_file() {
    let dir = folder("open-append");
    let path = dir.join("open-append");
    std::fs::write(&path, "Lorem ipsum\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    append(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn append_keeps_the_bytes_of_a_file_that_is_not_utf8() {
    let dir = folder("non-utf8-append");
    let path = dir.join("non-utf8-append");
    std::fs::write(&path, b"Lorem \xff ipsum\n").unwrap();

    append(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), b"Lorem \xff ipsum\nConsectetur adipiscing\n");
}

#[test]
fn append_starts_a_new_line_after_a_file_without_a_final_newline() {
    let dir = folder("no-newline-append");
    let path = dir.join("no-newline-append");
    std::fs::write(&path, "Lorem ipsum").unwrap();

    append(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Lorem ipsum\nConsectetur adipiscing\n");
}

#[test]
fn append_adds_no_newline_to_an_empty_or_absent_file() {
    let dir = folder("empty-append");
    let (empty, absent) = (dir.join("empty-append"), dir.join("absent-append"));
    std::fs::write(&empty, "").unwrap();
    let _ = std::fs::remove_file(&absent);

    append(&empty, &[Todo::from_line("Lorem ipsum")]).unwrap();
    append(&absent, &[Todo::from_line("Lorem ipsum")]).unwrap();

    assert_eq!(std::fs::read_to_string(&empty).unwrap(), "Lorem ipsum\n");
    assert_eq!(std::fs::read_to_string(&absent).unwrap(), "Lorem ipsum\n");
}

#[test]
fn save_creates_the_target_of_a_symlink_that_has_none_yet() {
    let dir = folder("absent");
    let (target, link) = (dir.join("absent"), dir.join("dangling"));
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(target.file_name().unwrap(), &link).unwrap();

    save(&link, &[Todo::from_line("Lorem ipsum")]).unwrap();

    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "Lorem ipsum\n");
}

#[test]
fn save_refuses_a_file_that_is_read_only() {
    let dir = folder("readonly");
    let path = dir.join("readonly");
    let _ = std::fs::remove_file(&path);
    std::fs::write(&path, "Lorem ipsum\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o464)).unwrap();

    let refused = save(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap_err();

    assert_eq!(refused.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Lorem ipsum\n");
}

#[test]
fn save_creates_a_new_file_readable_by_its_owner_only() {
    let dir = folder("new");
    let path = dir.join("new");
    let _ = std::fs::remove_file(&path);

    save(&path, &[Todo::from_line("Lorem ipsum")]).unwrap();

    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn save_does_not_write_through_a_symlink_left_at_the_temporary_name() {
    let dir = folder("planted");
    let (path, victim) = (dir.join("planted"), dir.join("victim"));
    std::fs::write(&path, "Lorem ipsum\n").unwrap();
    std::fs::write(&victim, "kept\n").unwrap();
    let mut planted = std::fs::canonicalize(&path).unwrap().into_os_string();
    planted.push(".tmp");
    let _ = std::fs::remove_file(&planted);
    std::os::unix::fs::symlink(&victim, &planted).unwrap();

    save(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

    assert_eq!(std::fs::read_to_string(&victim).unwrap(), "kept\n");
    assert!(!std::fs::symlink_metadata(&path).unwrap().is_symlink());
}

#[test]
fn a_failed_save_leaves_no_temporary_file_behind() {
    let dir = folder("directory");
    let path = dir.join("directory");
    std::fs::create_dir_all(path.join("kept")).unwrap();

    assert!(save(&path, &[Todo::from_line("Lorem ipsum")]).is_err());

    let mut tmp = std::fs::canonicalize(&path).unwrap().into_os_string();
    tmp.push(format!(".{}.tmp", std::process::id()));
    assert!(!PathBuf::from(tmp).exists());
}

#[test]
fn load_drops_blank_lines_from_the_tasks_but_not_from_the_raw_text() {
    let dir = folder("blank");
    let path = dir.join("blank");
    std::fs::write(&path, "Lorem ipsum\n\n   \nConsectetur adipiscing\n").unwrap();

    let (text, todos) = load(&path).unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(text, "Lorem ipsum\n\n   \nConsectetur adipiscing\n");
}

#[test]
fn load_returns_an_empty_list_when_the_file_is_missing() {
    let dir = folder("missing");
    let path = dir.join("missing");
    let _ = std::fs::remove_file(&path);

    let (text, todos) = load(&path).unwrap();
    assert_eq!(text, "");
    assert_eq!(todos.len(), 0);
}

#[test]
fn load_fails_on_a_file_it_cannot_decode() {
    let dir = folder("invalid");
    let path = dir.join("invalid");
    std::fs::write(&path, b"Lorem ipsum\n\xff\n").unwrap();

    assert!(load(&path).is_err());
}

#[test]
fn read_fails_on_a_path_it_cannot_open_for_another_reason_than_absence() {
    let dir = folder("not-a-folder");
    let file = dir.join("not-a-folder");
    std::fs::write(&file, "Lorem ipsum\n").unwrap();

    assert_eq!(read(&file.join("todo.txt")).unwrap_err().kind(), io::ErrorKind::NotADirectory);
}

#[test]
fn remove_task_finds_a_done_line_that_carries_a_priority() {
    let dir = folder("remove-done-priority");
    let path = dir.join("remove-done-priority");
    std::fs::write(&path, "x 2026-02-02 (C) 2026-01-01 kept\nx (A) 2026-01-01 gone\n").unwrap();
    let line = Todo::from_line("x (A) 2026-01-01 gone").to_line();

    let removed = remove_task(&path, Some(2), &line).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(removed);
    assert_eq!(text, "x 2026-02-02 (C) 2026-01-01 kept\n");
}
