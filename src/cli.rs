use clap::{Parser, Subcommand};

use crate::todo::Todo;

/// Minimal todo.txt editor.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// List of commands to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for the todo CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// List pending tasks (--all to include done ones)
    #[command(alias = "ls")]
    List {
        /// Show all tasks, including done ones
        #[arg(short, long)]
        all: bool,
    },

    /// Add a task, optionally as a todo.txt fragment ("(A) Buy milk +grocery")
    #[command(alias = "a")]
    Add {
        /// Task text, optionally with priority and projects
        text: String,

        /// Priority letter, A to E
        #[arg(short, long, value_parser = parse_priority)]
        priority: Option<char>,
    },

    /// Remove a task by its number
    #[command(alias = "rm")]
    Remove {
        /// Task number to remove
        number: usize,
    },

    /// Mark a task done by its number
    Do {
        /// Task number to mark as done
        number: usize,
    },
}

/// Parse a priority letter from A to E.
fn parse_priority(input: &str) -> Result<char, String> {
    let mut chars = input.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if Todo::is_valid_priority(c) => Ok(c),
        _ => Err("priority must be a single letter from A to E".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_priority_must_be_a_single_letter_from_a_to_e() {
        assert!(Cli::try_parse_from(["todo", "add", "Lorem", "-p", "A"]).is_ok());
        assert!(Cli::try_parse_from(["todo", "add", "Lorem", "-p", "Z"]).is_err());
        assert!(Cli::try_parse_from(["todo", "add", "Lorem", "-p", "AB"]).is_err());
    }
}
