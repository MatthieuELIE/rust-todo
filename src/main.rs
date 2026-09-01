mod cli;
mod repository;
mod store;
mod todo;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use time::{Date, OffsetDateTime};

use cli::{Cli, Commands};
use store::Store;
use todo::Todo;

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
        Commands::List { all } => {
            print_tasks(&store, all);
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
        Commands::Done { number } => {
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

// Only the CLI can supply a priority twice (flag and inline), so this reconciliation stays out of Todo.
fn build_task(text: &str, flag_priority: Option<char>) -> Result<Todo, String> {
    let mut todo = Todo::new_from_input(text, today())?;
    match (todo.priority, flag_priority) {
        (Some(_), Some(_)) => return Err("priority given twice".to_string()),
        (None, Some(priority)) => todo.priority = Some(priority),
        _ => {}
    }
    Ok(todo)
}

fn today() -> Date {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()).date()
}

fn todo_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TODO_FILE") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME").expect("HOME is not set");
    PathBuf::from(home).join("todo.txt")
}

fn print_tasks(store: &Store, all: bool) {
    if all {
        for (index, todo) in store.list_all().iter().enumerate() {
            println!("{:>3}  {}", index + 1, render(todo));
        }
    } else {
        for (number, todo) in store.list_pending() {
            println!("{number:>3}  {}", render(todo));
        }
    }
}

fn render(todo: &Todo) -> String {
    let mark = if todo.done { "x" } else { " " };
    match todo.priority {
        Some(priority) => format!("[{mark}] ({priority}) {}", todo.description),
        None => format!("[{mark}] {}", todo.description),
    }
}
