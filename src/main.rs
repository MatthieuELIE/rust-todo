mod cli;
mod commands;
mod config;
mod editor;
mod filter;
mod repository;
mod terminal;
mod theme;
mod toast;
mod todo;
mod tui;
mod view;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use ratatui::crossterm::style::force_color_output;
use time::{Date, OffsetDateTime};

use cli::{Cli, Commands};
use theme::Theme;

/// Entry point for the todo CLI application.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = todo_path();
    let no_color = config::no_color(std::env::var_os("NO_COLOR"));
    let (theme, icons) = match config::theme(&config::path(std::env::var_os("XDG_CONFIG_HOME"), std::env::home_dir()), no_color) {
        Ok(read) => read,
        Err(message) => {
            eprintln!("{message}");
            if cli.command.is_none() {
                let _ = terminal::show_error(&message, &config::choose(no_color, None));
            }
            return ExitCode::FAILURE;
        }
    };
    if no_color && theme != Theme::NONE {
        force_color_output(true);
    }
    let path = if matches!(cli.command, Some(Commands::List { done: true, .. })) {
        path.with_file_name("done.txt")
    } else {
        path
    };

    let (text, todos) = match repository::load(&path) {
        Ok(loaded) => loaded,
        Err(e) => {
            let message = format!("could not read {}: {e}", path.display());
            eprintln!("{message}");
            if cli.command.is_none() {
                let _ = terminal::show_error(&message, &theme);
            }
            return ExitCode::FAILURE;
        }
    };

    let Some(command) = cli.command else {
        return match terminal::run(todos, text, &path, theme, icons) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    };

    commands::run(command, path, todos, theme)
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
