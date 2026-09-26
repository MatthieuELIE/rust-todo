use crate::todo::Todo;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Load the todo list from a file along with its raw text, both empty if the file is missing.
pub fn load(path: &Path) -> io::Result<(String, Vec<Todo>)> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };

    let todos = content.lines().filter(|line| !line.trim().is_empty()).map(Todo::from_line).collect();
    Ok((content, todos))
}

/// Save the todo list to a file, overwriting any existing content, and return the text written.
pub fn save(path: &Path, todos: &[Todo]) -> io::Result<String> {
    let body: String = todos.iter().map(|todo| todo.to_line() + "\n").collect();

    // Write to a sibling file and rename over the target so a crash can't leave a half-written todo file.
    let mut tmp = PathBuf::from(path).into_os_string();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);

    fs::write(&tmp, &body)?;
    fs::rename(&tmp, path)?;
    Ok(body)
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
}
