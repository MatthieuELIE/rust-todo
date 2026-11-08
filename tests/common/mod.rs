use std::path::Path;
use std::process::{Command, Output};

pub fn todo(file: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_todo"))
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .args(args)
        .env("TODO_FILE", file)
        .output()
        .unwrap()
}

pub fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("todo-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    folder
}
