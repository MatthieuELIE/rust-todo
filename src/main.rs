mod cli;
mod editor;
mod repository;
mod todo;
mod tui;
mod view;

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use ratatui::DefaultTerminal;
use ratatui::backend::IntoCrossterm;
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::text::Line;
use ratatui::widgets::ListState;
use time::{Date, OffsetDateTime};

use cli::{Cli, Commands};
use todo::Todo;
use tui::App;

/// Entry point for the todo CLI application.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = todo_path();
    let path = if matches!(cli.command, Some(Commands::List { done: true, .. })) {
        path.with_file_name("done.txt")
    } else {
        path
    };

    let (text, mut todos) = match repository::load(&path) {
        Ok(loaded) => loaded,
        Err(e) => {
            let message = format!("could not read {}: {e}", path.display());
            eprintln!("{message}");
            if cli.command.is_none() {
                let _ = show_error(&message);
            }
            return ExitCode::FAILURE;
        }
    };

    let Some(command) = cli.command else {
        return match run(todos, text, &path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    };

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
            let mut tasks = todo::list(&todos, &terms);
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
                println!("{}{}", view::number(number), shown(todo, today));
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
            if !todo.done {
                eprintln!("task {number} is not done");
                return ExitCode::FAILURE;
            }
            if !todo.reopen() {
                eprintln!("task {number} cannot be reopened: its text starts with x");
                return ExitCode::FAILURE;
            }
            if todo.description.trim().is_empty() {
                eprintln!("task {number} cannot be reopened: it has no description");
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
        let _ = writeln!(io::stdout(), "{verb}: {}", shown(&todo, today()));
    }
    if let Some(count) = archived {
        let _ = writeln!(io::stdout(), "{count} task{} archived", if count == 1 { "" } else { "s" });
    }

    ExitCode::SUCCESS
}

/// `text` with each control character replaced, so that a line of the file cannot drive the terminal.
fn clean(text: &str) -> String {
    text.replace(char::is_control, "\u{fffd}")
}

/// A task as `list` prints it, without its number: styled on a terminal that takes colours, its plain line otherwise.
fn shown(todo: &Todo, today: Date) -> String {
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty()) {
        styled(&view::line(0, todo, today))
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

/// Returns the current date in the local timezone, or UTC if local time is unavailable.
fn today() -> Date {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()).date()
}

/// Returns the path to the todo.txt file, either from the TODO_FILE environment variable or defaulting to the home directory's.
fn todo_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TODO_FILE") {
        return PathBuf::from(path);
    }

    std::env::home_dir().unwrap_or_default().join("todo.txt")
}

/// Runs the list until the user quits, saving to `path` on each change and reloading when the file no longer matches `text`.
fn run(todos: Vec<Todo>, mut text: String, path: &Path) -> io::Result<()> {
    let mut app = App::new(todos);
    app.today = Some(today());
    let mut scroll = ListState::default();
    on_terminal(|terminal| {
        execute!(io::stdout(), EnableBracketedPaste)?;
        while !app.quit {
            app.refresh_history(path);
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll, today()))?;
            execute!(io::stdout(), app.cursor_shape())?;
            let event = if event::poll(Duration::from_millis(250))? {
                Some(event::read()?)
            } else {
                None
            };
            if app.turn(repository::read(path), &mut text, event, today()) {
                app.save(path, &mut text);
            }
        }
        execute!(io::stdout(), DisableBracketedPaste, SetCursorStyle::DefaultUserShape)
    })
}

/// Runs `f` on the terminal set up for the full screen and puts it back, with an error when it cannot be set up.
fn on_terminal(f: impl FnOnce(&mut DefaultTerminal) -> io::Result<()>) -> io::Result<()> {
    let mut terminal = ratatui::try_init().map_err(|e| io::Error::other(format!("could not set up the terminal: {e}")))?;
    let result = f(&mut terminal);
    ratatui::restore();
    result
}

/// Shows `message` full screen until a key is pressed, so a popup that closes when the program exits does not swallow it.
fn show_error(message: &str) -> io::Result<()> {
    on_terminal(|terminal| {
        terminal.draw(|frame| view::draw_error(frame, message))?;
        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                return Ok(());
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn a_styled_line_replaces_the_control_characters_of_the_task() {
        let todo = Todo::from_line("Pay \u{1b}[2Jrent");

        let styled = styled(&view::line(1, &todo, date!(2026 - 09 - 26)));

        assert!(styled.contains("\u{fffd}[2Jrent"));
        assert!(!styled.contains("\u{1b}[2J"));
    }

    #[test]
    fn a_styled_line_leaves_its_number_out() {
        for line in ["(A) Pay rent", "x 2026-09-03 Pay rent"] {
            let styled = styled(&view::line(7, &Todo::from_line(line), date!(2026 - 09 - 26)));

            assert!(!styled.contains(&view::number(7)));
            assert!(styled.contains("rent"));
        }
    }
}
