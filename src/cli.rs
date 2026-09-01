use clap::{Parser, Subcommand};

use crate::todo::Todo;

/// Minimal todo.txt editor.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List pending tasks (--all to include done ones)
    List {
        #[arg(short, long)]
        all: bool,
    },
    /// Add a task
    Add {
        description: String,

        /// Priority letter, A to E
        #[arg(short, long, value_parser = parse_priority)]
        priority: Option<char>,
    },
    /// Remove a task by its number
    Remove { number: usize },
    /// Mark a task done by its number
    Done { number: usize },
}

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
