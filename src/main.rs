mod cli;
mod editor;
mod repository;
mod todo;
mod tui;
mod view;

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use ratatui::backend::IntoCrossterm;
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::widgets::ListState;
use time::{Date, OffsetDateTime};

use cli::{Cli, Commands};
use todo::Todo;
use tui::App;

/// Entry point for the todo CLI application.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = todo_path();

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

    match command {
        Commands::List { all, terms } => {
            let tasks = todo::list(&todos, all, &terms);
            if tasks.is_empty() {
                eprintln!("{}", if terms.is_empty() { "nothing to do" } else { "no matching task" });
            }
            let colour = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty());
            let today = today();
            for (number, todo) in tasks {
                if colour {
                    let spans: String = view::line(number, todo, today)
                        .iter()
                        .map(|span| span.style.into_crossterm().apply(&span.content).to_string())
                        .collect();
                    println!("{spans}");
                } else {
                    println!("{number:>3}  {}", todo.to_line());
                }
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

        Commands::Remove { number } | Commands::Do { number } if !(1..=todos.len()).contains(&number) => {
            eprintln!("no task numbered {number}");
            return ExitCode::FAILURE;
        }

        Commands::Remove { number } => {
            todos.remove(number - 1);
        }

        Commands::Do { number } => todos[number - 1].complete(today()),
    }

    if let Err(e) = repository::save(&path, &todos) {
        eprintln!("could not save {}: {e} (file left unchanged)", path.display());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
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
    let mut scroll = ListState::default();
    ratatui::run(|terminal| {
        execute!(io::stdout(), EnableBracketedPaste)?;
        while !app.quit {
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll, today()))?;
            execute!(io::stdout(), app.cursor_shape())?;
            let event = if event::poll(Duration::from_millis(250))? {
                Some(event::read()?)
            } else {
                None
            };
            if app.turn(repository::read(path), &mut text, event, today()) {
                match repository::save(path, &app.todos) {
                    Ok(written) => text = written,
                    Err(e) => {
                        if let Ok((_, todos)) = repository::load(path) {
                            app.reload(todos);
                        }
                        app.refuse(&format!("could not save: {e} (file left unchanged)"));
                    }
                }
            }
        }
        execute!(io::stdout(), DisableBracketedPaste, SetCursorStyle::DefaultUserShape)
    })
}

/// Shows `message` full screen until a key is pressed, so a popup that closes when the program exits does not swallow it.
fn show_error(message: &str) -> io::Result<()> {
    ratatui::run(|terminal| {
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
