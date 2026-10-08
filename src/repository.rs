use crate::todo::Todo;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Read the raw text of the todo file, empty if the file is missing.
pub fn read(path: &Path) -> io::Result<String> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        read => read,
    }
}

/// Parse the tasks of a todo file's text, its blank lines dropped.
pub fn parse(text: &str) -> Vec<Todo> {
    text.lines().filter(|line| !line.trim().is_empty()).map(Todo::from_line).collect()
}

/// Load the todo list from a file along with its raw text, both empty if the file is missing.
pub fn load(path: &Path) -> io::Result<(String, Vec<Todo>)> {
    let text = read(path)?;
    let todos = parse(&text);
    Ok((text, todos))
}

/// Save the todo list to a file, overwriting any existing content, and return the text written.
pub fn save(path: &Path, todos: &[Todo]) -> io::Result<String> {
    let body: String = todos.iter().map(|todo| todo.to_line() + "\n").collect();
    let path = fs::canonicalize(path).unwrap_or_else(|_| match fs::read_link(path) {
        Ok(target) => path.with_file_name(target),
        Err(_) => path.to_path_buf(),
    });

    // Write to a sibling file and rename over the target so a crash can't leave a half-written todo file.
    let mut tmp = path.clone().into_os_string();
    tmp.push(format!(".{}.tmp", std::process::id()));
    let tmp = PathBuf::from(tmp);

    let permissions = match fs::OpenOptions::new().write(true).open(&path) {
        Ok(file) => Some(file.metadata()?.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let written = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.write_all(body.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written.map(|()| body)
}

/// Add the tasks to the end of a file, created if it is missing, without rewriting what it holds.
pub fn append(path: &Path, todos: &[Todo]) -> io::Result<()> {
    let mut body: String = todos.iter().map(|todo| todo.to_line() + "\n").collect();
    if read(path)?.chars().last().is_some_and(|last| last != '\n') {
        body.insert(0, '\n');
    }
    let mut file = fs::OpenOptions::new().append(true).create(true).mode(0o600).open(path)?;
    file.write_all(body.as_bytes())?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("todo-{}-{name}.txt", std::process::id()))
    }

    #[test]
    fn save_then_load_round_trips_the_list() {
        let path = temp_path("roundtrip");
        let todos = vec![Todo::from_line("(A) Lorem ipsum dolor"), Todo::from_line("x Consectetur adipiscing")];

        let written = save(&path, &todos).unwrap();
        let (text, loaded) = load(&path).unwrap();

        let lines: Vec<String> = loaded.iter().map(Todo::to_line).collect();
        assert_eq!(lines, ["(A) Lorem ipsum dolor", "x Consectetur adipiscing"]);
        assert_eq!(text, written);
    }

    #[test]
    fn save_writes_through_a_symlink_and_keeps_the_file_mode() {
        use std::os::unix::fs::PermissionsExt;
        let (target, link) = (temp_path("target"), temp_path("link"));
        std::fs::write(&target, "Lorem ipsum\n").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link).unwrap();

        save(&link, &[Todo::from_line("Consectetur adipiscing")]).unwrap();

        assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "Consectetur adipiscing\n");
        assert_eq!(std::fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o640);
    }

    #[test]
    fn save_creates_the_target_of_a_symlink_that_has_none_yet() {
        let (target, link) = (temp_path("absent"), temp_path("dangling"));
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(target.file_name().unwrap(), &link).unwrap();

        save(&link, &[Todo::from_line("Lorem ipsum")]).unwrap();

        assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "Lorem ipsum\n");
    }

    #[test]
    fn save_refuses_a_file_that_is_read_only() {
        use std::os::unix::fs::PermissionsExt;
        let path = temp_path("readonly");
        let _ = std::fs::remove_file(&path);
        std::fs::write(&path, "Lorem ipsum\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o464)).unwrap();

        let refused = save(&path, &[Todo::from_line("Consectetur adipiscing")]).unwrap_err();

        assert_eq!(refused.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "Lorem ipsum\n");
    }

    #[test]
    fn save_creates_a_new_file_readable_by_its_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let path = temp_path("new");
        let _ = std::fs::remove_file(&path);

        save(&path, &[Todo::from_line("Lorem ipsum")]).unwrap();

        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn save_does_not_write_through_a_symlink_left_at_the_temporary_name() {
        let (path, victim) = (temp_path("planted"), temp_path("victim"));
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
        let path = temp_path("directory");
        std::fs::create_dir_all(path.join("kept")).unwrap();

        assert!(save(&path, &[Todo::from_line("Lorem ipsum")]).is_err());

        let mut tmp = std::fs::canonicalize(&path).unwrap().into_os_string();
        tmp.push(format!(".{}.tmp", std::process::id()));
        assert!(!PathBuf::from(tmp).exists());
    }

    #[test]
    fn load_drops_blank_lines_from_the_tasks_but_not_from_the_raw_text() {
        let path = temp_path("blank");
        std::fs::write(&path, "Lorem ipsum\n\n   \nConsectetur adipiscing\n").unwrap();

        let (text, todos) = load(&path).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(text, "Lorem ipsum\n\n   \nConsectetur adipiscing\n");
    }

    #[test]
    fn load_returns_an_empty_list_when_the_file_is_missing() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);

        let (text, todos) = load(&path).unwrap();
        assert_eq!(text, "");
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn load_fails_on_a_file_it_cannot_decode() {
        let path = temp_path("invalid");
        std::fs::write(&path, b"Lorem ipsum\n\xff\n").unwrap();

        assert!(load(&path).is_err());
    }
}
