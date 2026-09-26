mod cli;
mod repository;
mod store;
mod todo;
mod tui;
mod view;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use ratatui::backend::IntoCrossterm;
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

    let Some(command) = cli.command else {
        return match tui::run(store, &path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    };

    match command {
        Commands::List { all, terms } => {
            let tasks = store.list(all, &terms);
            if tasks.is_empty() {
                eprintln!("{}", if terms.is_empty() { "nothing to do" } else { "no matching task" });
            }
            let colour = std::io::stdout().is_terminal();
            for (number, todo) in tasks {
                let line = view::line(number, todo);
                if colour {
                    println!("{}", view::style(todo).into_crossterm().apply(line));
                } else {
                    println!("{line}");
                }
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
