mod cli;
mod repository;
mod store;
mod todo;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use time::{Date, OffsetDateTime};

use cli::{Cli, Commands};
use store::Store;
use todo::Todo;

/// Entry point for the todo CLI application.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = todo_path();

    let mut store = match repository::load(&path) {
        Ok(todos) => Store::new(todos),
        Err(e) => {
            eprintln!("could not read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };

    match cli.command {
        Commands::List { all, terms } => {
            let tasks = store.list(all, &terms);
            if tasks.is_empty() {
                eprintln!("{}", if terms.is_empty() { "nothing to do" } else { "no matching task" });
            }
            let colour = std::io::stdout().is_terminal();
            for (number, todo) in tasks {
                let line = format!("{number:>3}  {}", todo.to_line());
                println!("{}", if colour { paint(line, todo) } else { line });
            }
            return ExitCode::SUCCESS;
        }

        Commands::Add { text, priority } => match build_task(&text, priority) {
            Ok(todo) => store.add(todo),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },

        Commands::Remove { number } => {
            if !store.remove(number) {
                eprintln!("no task numbered {number}");
                return ExitCode::FAILURE;
            }
        }

        Commands::Do { number } => {
            if !store.done(number, today()) {
                eprintln!("no task numbered {number}");
                return ExitCode::FAILURE;
            }
        }
    }

    if let Err(e) = repository::save(&path, &store.todos) {
        eprintln!("could not save {}: {e} (file left unchanged)", path.display());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Builds the task to add from its text and the `--priority` flag, refusing a priority given both ways.
/// Lives here rather than in `Todo`: only the CLI can supply a priority twice.
fn build_task(text: &str, flag_priority: Option<char>) -> Result<Todo, String> {
    let mut todo = Todo::new_from_input(text, today())?;
    match (todo.priority, flag_priority) {
        (Some(_), Some(_)) => return Err("priority given twice".to_string()),
        (None, Some(priority)) => todo.priority = Some(priority),
        _ => {}
    }
    Ok(todo)
}

/// Wraps a listed line in ANSI bold when the task has a priority, tinted for A to C as `todo.sh` does, dim when it is done.
fn paint(line: String, todo: &Todo) -> String {
    let style = match (todo.done, todo.priority) {
        (true, _) => "2",
        (false, Some('A')) => "1;33",
        (false, Some('B')) => "1;32",
        (false, Some('C')) => "1;34",
        (false, Some(_)) => "1",
        (false, None) => return line,
    };
    format!("\x1b[{style}m{line}\x1b[0m")
}

/// Returns the current date in the local timezone, or UTC if local time is unavailable.
fn today() -> Date {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()).date()
}

/// Returns the path to the todo.txt file, either from the TODO_FILE environment variable or defaulting to $HOME/todo.txt.
fn todo_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TODO_FILE") {
        return PathBuf::from(path);
    }

    let home = std::env::var_os("HOME").expect("HOME is not set");
    PathBuf::from(home).join("todo.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_tasks_are_dimmed_and_prioritised_ones_bold_with_a_to_c_tinted() {
        let paint_line = |line: &str| paint(line.to_string(), &Todo::from_line(line));

        assert_eq!(paint_line("Buy milk"), "Buy milk");
        assert_eq!(paint_line("(A) Call the bank"), "\x1b[1;33m(A) Call the bank\x1b[0m");
        assert_eq!(paint_line("(B) Pay rent"), "\x1b[1;32m(B) Pay rent\x1b[0m");
        assert_eq!(paint_line("(C) Book dentist"), "\x1b[1;34m(C) Book dentist\x1b[0m");
        assert_eq!(paint_line("(D) Read book"), "\x1b[1m(D) Read book\x1b[0m");
        assert_eq!(paint_line("x 2026-09-03 Buy milk"), "\x1b[2mx 2026-09-03 Buy milk\x1b[0m");
    }
}
