use crate::todo::Todo;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load(path: &Path) -> io::Result<Vec<Todo>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    Ok(content.lines().filter(|line| !line.trim().is_empty()).map(Todo::from_line).collect())
}

pub fn save(path: &Path, todos: &[Todo]) -> io::Result<()> {
    let body: String = todos.iter().map(|todo| todo.to_line() + "\n").collect();

    // Write to a sibling file and rename over the target so a crash can't leave a half-written todo file.
    let mut tmp = PathBuf::from(path).into_os_string();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);

    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)
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

        save(&path, &todos).unwrap();
        let lines: Vec<String> = load(&path).unwrap().iter().map(Todo::to_line).collect();

        assert_eq!(lines, ["(A) Lorem ipsum dolor", "x Consectetur adipiscing"]);
    }

    #[test]
    fn load_drops_blank_lines() {
        let path = temp_path("blank");
        std::fs::write(&path, "Lorem ipsum\n\n   \nConsectetur adipiscing\n").unwrap();

        assert_eq!(load(&path).unwrap().len(), 2);
    }

    #[test]
    fn load_returns_an_empty_list_when_the_file_is_missing() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);

        assert_eq!(load(&path).unwrap().len(), 0);
    }
}
