use std::fmt::Display;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::cli::{Commands, Task};
use crate::filter;
use crate::repository;
use crate::theme::Theme;
use crate::today;
use crate::todo::Todo;

/// The number of the one pending task `term` matches, as `list` matches it, or the message saying how many match.
fn pending_match(todos: &[Todo], term: &str) -> Result<usize, String> {
    if term.starts_with('-') {
        return Err(format!("edit takes a term to match, not one to exclude: {term}"));
    }

    let numbers: Vec<usize> = filter::list(todos, &[term.to_owned()])
        .into_iter()
        .filter(|(_, todo)| !todo.done)
        .map(|(number, _)| number)
        .collect();

    match numbers[..] {
        [number] => Ok(number),
        [] => Err(format!("no pending task matches {term}")),
        _ => Err(format!("{} pending tasks match {term}", numbers.len())),
    }
}

/// What a command leaves to write and say once it has changed the tasks.
#[derive(Default)]
struct Plan {
    todos: Vec<Todo>,
    moved: Vec<Todo>,
    reopened: Option<(usize, String)>,
    confirmed: Option<(&'static str, Todo)>,
    archived: Option<usize>,
}

/// A command that ends without anything to write: its exit code and what it says on stderr.
struct Stop {
    code: ExitCode,
    message: Option<String>,
}

impl Stop {
    fn fail(message: impl Display) -> Self {
        Self {
            code: ExitCode::FAILURE,
            message: Some(message.to_string()),
        }
    }

    fn succeed(message: Option<&str>) -> Self {
        Self {
            code: ExitCode::SUCCESS,
            message: message.map(str::to_owned),
        }
    }

    fn exit(self) -> ExitCode {
        if let Some(message) = self.message {
            eprintln!("{message}");
        }

        self.code
    }
}

type Step = Result<Plan, Stop>;

fn refuse_done_itself(path: &Path) -> Stop {
    Stop::fail(format!("the task file is done.txt itself: {}", path.display()))
}

fn not_saved(path: &Path, error: impl Display) -> Stop {
    Stop::fail(format!("could not save {}: {error} (file left unchanged)", path.display()))
}

fn in_range(todos: &[Todo], number: usize) -> Result<(), Stop> {
    if !(1..=todos.len()).contains(&number) {
        return Err(Stop::fail(format!("no task numbered {number}")));
    }

    Ok(())
}

fn add(mut todos: Vec<Todo>, text: &str) -> Step {
    let todo = Todo::new_from_input(text, today()).map_err(Stop::fail)?;
    todos.push(todo);

    Ok(Plan { todos, ..Plan::default() })
}

fn remove(mut todos: Vec<Todo>, number: usize) -> Step {
    in_range(&todos, number)?;
    let todo = todos.remove(number - 1);

    Ok(Plan {
        todos,
        confirmed: Some(("removed", todo)),
        ..Plan::default()
    })
}

fn do_task(mut todos: Vec<Todo>, number: usize) -> Step {
    in_range(&todos, number)?;
    if todos[number - 1].done {
        return Err(Stop::fail(format!("task {number} is already done")));
    }

    let mut todo = todos.remove(number - 1);
    let confirmed = Some(("done", todo.clone()));
    todo.complete(today());

    Ok(Plan {
        todos,
        moved: vec![todo],
        confirmed,
        ..Plan::default()
    })
}

fn edit(mut todos: Vec<Todo>, task: Task, text: &str) -> Step {
    let number = match task {
        Task::Number(number) => {
            in_range(&todos, number)?;
            if todos[number - 1].done {
                return Err(Stop::fail(format!("task {number} is done")));
            }

            number
        }
        Task::Term(term) => pending_match(&todos, &term).map_err(Stop::fail)?,
    };

    let mut todo = Todo::from_input(text).map_err(Stop::fail)?;
    if todo.done {
        return Err(Stop::fail("cannot edit a task into a done one"));
    }

    todo.created = todo.created.or(todos[number - 1].created);
    if todo.to_line() == todos[number - 1].to_line() {
        return Err(Stop::succeed(None));
    }

    todo.check_text().map_err(Stop::fail)?;
    todos[number - 1] = todo;

    Ok(Plan { todos, ..Plan::default() })
}

/// Reopens the task numbered `number` in `done_text`, the content of done.txt, onto the pending ones.
fn reopen(mut todos: Vec<Todo>, number: usize, done_text: &str) -> Step {
    let lines: Vec<&str> = done_text.split_inclusive('\n').collect();
    let mut tasks = (0..lines.len()).filter(|&i| !lines[i].trim().is_empty());
    let Some(index) = tasks.nth(number.wrapping_sub(1)) else {
        return Err(Stop::fail(format!("no task numbered {number}")));
    };
    let mut todo = Todo::from_line(lines[index].trim_end_matches(['\n', '\r']));
    let line = todo.to_line();

    todo.reopen().map_err(|reason| Stop::fail(format!("task {number} {reason}")))?;

    let confirmed = Some(("reopened", todo.clone()));
    todos.push(todo);

    Ok(Plan {
        todos,
        reopened: Some((number, line)),
        confirmed,
        ..Plan::default()
    })
}

fn archive(todos: Vec<Todo>) -> Step {
    let (done, pending): (Vec<Todo>, Vec<Todo>) = todos.into_iter().partition(|todo| todo.done);

    if done.is_empty() {
        return Err(Stop::succeed(Some("nothing to archive")));
    }

    Ok(Plan {
        todos: pending,
        archived: Some(done.len()),
        moved: done,
        ..Plan::default()
    })
}

/// Writes what `plan` changed, `done.txt` before the removal of a reopened task.
fn save(plan: &Plan, path: &Path, done_path: &Path) -> Result<(), Stop> {
    let reopens = plan.reopened.is_some();

    repository::move_to_done(path, done_path, &plan.todos, &plan.moved, reopens, reopens).map_err(|e| match e {
        repository::MoveError::SameFile => refuse_done_itself(path),
        repository::MoveError::Failed { path, error, .. } => not_saved(&path, error),
    })?;

    if let Some((number, line)) = &plan.reopened {
        repository::remove_task(done_path, Some(*number), line).map_err(|e| not_saved(done_path, e))?;
    }

    Ok(())
}

/// Writes what `plan` changed, then says what it did.
fn apply(plan: Plan, path: &Path, done_path: &Path, theme: &Theme) -> ExitCode {
    if let Err(stop) = save(&plan, path, done_path) {
        return stop.exit();
    }

    if let Some((verb, todo)) = plan.confirmed {
        let _ = writeln!(io::stdout(), "{verb}: {}", list::shown(theme, &todo, today()));
    }
    if let Some(count) = plan.archived {
        let _ = writeln!(io::stdout(), "{count} task{} archived", if count == 1 { "" } else { "s" });
    }

    ExitCode::SUCCESS
}

/// Runs `command` on the tasks of the file at `path`, writes what it changed and returns the exit code.
pub fn run(command: Commands, path: PathBuf, todos: Vec<Todo>, theme: Theme) -> ExitCode {
    let done_path = path.with_file_name("done.txt");

    if matches!(command, Commands::Archive | Commands::Reopen { .. } | Commands::Do { .. }) && repository::same_file(&path, &done_path) {
        return refuse_done_itself(&path).exit();
    }

    let step = match command {
        Commands::List { terms, .. } if terms.first().is_some_and(|term| term == "--all" || term == "-a") => {
            let message = "--all is gone: list shows the pending tasks, list --done the done ones";
            return Stop {
                code: ExitCode::from(2),
                message: Some(message.to_owned()),
            }
            .exit();
        }
        Commands::List { due, done, terms } => return list::list(&todos, due, done, &terms, &theme),
        Commands::Add { text } => add(todos, &text),
        Commands::Remove { number } => remove(todos, number),
        Commands::Do { number } => do_task(todos, number),
        Commands::Edit { task, text } => edit(todos, task, &text),
        Commands::Reopen { number } => repository::read(&done_path)
            .map_err(|e| Stop::fail(format!("could not read {}: {e}", done_path.display())))
            .and_then(|text| reopen(todos, number, &text)),
        Commands::Archive => archive(todos),
    };

    match step {
        Ok(plan) => apply(plan, &path, &done_path, &theme),
        Err(stop) => stop.exit(),
    }
}

mod list;
