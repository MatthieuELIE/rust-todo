mod guard;

pub use guard::Folder;

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

pub fn folder(name: &str) -> Folder {
    Folder::new(name)
}
