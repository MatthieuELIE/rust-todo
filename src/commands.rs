use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ratatui::backend::IntoCrossterm;
use ratatui::text::Line;
use time::Date;

use crate::cli::Commands;
use crate::filter;
use crate::repository;
use crate::theme::Theme;
use crate::today;
use crate::todo::Todo;
use crate::view;

/// `text` with each control character replaced, so that a line of the file cannot drive the terminal.
fn clean(text: &str) -> String {
    text.replace(char::is_control, "\u{fffd}")
}

/// A task as `list` prints it, without its number: styled on a terminal, its plain line otherwise.
fn shown(theme: &Theme, todo: &Todo, today: Date) -> String {
    if std::io::stdout().is_terminal() {
        styled(&view::line(theme, 0, todo, today))
    } else {
        clean(&todo.to_line())
    }
}

/// A listed line, without its number, as text carrying its styles for a terminal.
fn styled(line: &Line) -> String {
    line.iter()
        .skip(1)
        .map(|span| span.style.into_crossterm().apply(clean(&span.content)).to_string())
        .collect()
}

/// Runs `command` on the tasks of the file at `path`, writes what it changed and returns the exit code.
pub fn run(command: Commands, path: PathBuf, mut todos: Vec<Todo>, theme: Theme) -> ExitCode {
    let done_path = path.with_file_name("done.txt");
    if matches!(command, Commands::Archive | Commands::Reopen { .. } | Commands::Do { .. }) && repository::same_file(&path, &done_path) {
        eprintln!("the task file is done.txt itself: {}", path.display());
        return ExitCode::FAILURE;
    }

    let mut archived = None;
    let mut history = None;
    let mut moved = Vec::new();
    let mut confirmed = None;
    match command {
        Commands::List { terms, .. } if terms.first().is_some_and(|term| term == "--all" || term == "-a") => {
            eprintln!("--all is gone: list shows the pending tasks, list --done the done ones");
            return ExitCode::from(2);
        }

        Commands::List { due, done, terms } => {
            let today = today();
            let mut tasks = filter::list(&todos, &terms);
            tasks.retain(|(_, todo)| done || (!todo.done && (!due || todo.is_due(today))));
            if done {
                tasks.sort_by_key(|(number, _)| *number);
            }
            if tasks.is_empty() {
                let nothing = if due {
                    "nothing due"
                } else if done {
                    "nothing done"
                } else {
                    "nothing to do"
                };
                eprintln!("{}", if terms.is_empty() { nothing } else { "no matching task" });
            }
            for (number, todo) in tasks {
                println!("{}{}", view::number(number), shown(&theme, todo, today));
            }
            return ExitCode::SUCCESS;
        }

        Commands::Add { text } => match Todo::new_from_input(&text, today()) {
            Ok(todo) => todos.push(todo),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },

        Commands::Remove { number } | Commands::Do { number } | Commands::Edit { number, .. } if !(1..=todos.len()).contains(&number) => {
            eprintln!("no task numbered {number}");
            return ExitCode::FAILURE;
        }

        Commands::Remove { number } => confirmed = Some(("removed", todos.remove(number - 1))),

        Commands::Do { number } if todos[number - 1].done => {
            eprintln!("task {number} is already done");
            return ExitCode::FAILURE;
        }

        Commands::Do { number } => {
            let mut todo = todos.remove(number - 1);
            confirmed = Some(("done", todo.clone()));
            todo.complete(today());
            moved.push(todo);
        }

        Commands::Edit { number, .. } if todos[number - 1].done => {
            eprintln!("task {number} is done");
            return ExitCode::FAILURE;
        }

        Commands::Edit { number, text } => match Todo::from_input(&text) {
            Ok(todo) if todo.done => {
                eprintln!("cannot edit a task into a done one");
                return ExitCode::FAILURE;
            }
            Ok(mut todo) => {
                todo.created = todo.created.or(todos[number - 1].created);
                if todo.to_line() == todos[number - 1].to_line() {
                    return ExitCode::SUCCESS;
                }
                if let Err(e) = todo.check_text() {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
                todos[number - 1] = todo;
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },

        Commands::Reopen { number } => {
            let text = match repository::read(&done_path) {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("could not read {}: {e}", done_path.display());
                    return ExitCode::FAILURE;
                }
            };
            let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
            let mut tasks = (0..lines.len()).filter(|&i| !lines[i].trim().is_empty());
            let Some(index) = tasks.nth(number.wrapping_sub(1)) else {
                eprintln!("no task numbered {number}");
                return ExitCode::FAILURE;
            };
            let mut todo = Todo::from_line(lines.remove(index).trim_end_matches(['\n', '\r']));
            if let Err(reason) = todo.reopen() {
                eprintln!("task {number} {reason}");
                return ExitCode::FAILURE;
            }
            if let Err(e) = std::fs::OpenOptions::new().write(true).open(&done_path) {
                eprintln!("could not save {}: {e} (file left unchanged)", done_path.display());
                return ExitCode::FAILURE;
            }
            confirmed = Some(("reopened", todo.clone()));
            todos.push(todo);
            history = Some(lines.concat());
        }

        Commands::Archive => {
            let (done, pending): (Vec<Todo>, Vec<Todo>) = todos.into_iter().partition(|todo| todo.done);
            if done.is_empty() {
                eprintln!("nothing to archive");
                return ExitCode::SUCCESS;
            }
            archived = Some(done.len());
            moved = done;
            todos = pending;
        }
    }

    if !moved.is_empty()
        && let Err(e) = std::fs::OpenOptions::new().write(true).open(&path)
    {
        eprintln!("could not save {}: {e} (file left unchanged)", path.display());
        return ExitCode::FAILURE;
    }
    if !moved.is_empty()
        && let Err(e) = repository::append(&done_path, &moved)
    {
        eprintln!("could not save {}: {e} (file left unchanged)", done_path.display());
        return ExitCode::FAILURE;
    }
    if let Err(e) = repository::save(&path, &todos) {
        eprintln!("could not save {}: {e} (file left unchanged)", path.display());
        return ExitCode::FAILURE;
    }
    if let Some(body) = history
        && let Err(e) = repository::write(&done_path, body)
    {
        eprintln!("could not save {}: {e} (file left unchanged)", done_path.display());
        return ExitCode::FAILURE;
    }
    if let Some((verb, todo)) = confirmed {
        let _ = writeln!(io::stdout(), "{verb}: {}", shown(&theme, &todo, today()));
    }
    if let Some(count) = archived {
        let _ = writeln!(io::stdout(), "{count} task{} archived", if count == 1 { "" } else { "s" });
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests;
