use std::num::ParseIntError;

use clap::{Args, Parser, Subcommand};

/// Minimal todo.txt editor.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Command to run; without one, the task list opens interactively
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands for the todo CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// List pending tasks (--done for done.txt), filtered by terms (-term excludes)
    #[command(alias = "ls")]
    List {
        #[command(flatten)]
        flags: ListFlags,

        /// Case-insensitive terms every listed task must contain, or lack with a leading -
        #[arg(allow_hyphen_values = true)]
        terms: Vec<String>,
    },

    /// Add a task, optionally as a todo.txt fragment ("(A) Buy milk +grocery")
    #[command(alias = "a")]
    Add {
        /// Task text, optionally with priority and projects
        #[arg(allow_hyphen_values = true)]
        text: String,
    },

    /// Remove a task by its number
    #[command(alias = "rm")]
    Remove {
        /// Task number to remove
        number: usize,
    },

    /// Complete a task by its number: it moves to the end of done.txt
    #[command(alias = "done")]
    Do {
        /// Task number to complete
        number: usize,
    },

    /// Replace a pending task, by its number or a term only it matches, with a todo.txt line, keeping its creation date unless the line carries one
    Edit {
        /// Task number to edit, or a case-insensitive term matching a single pending task
        #[arg(value_parser = task)]
        task: Task,

        /// The task's new line, optionally with priority and projects
        #[arg(allow_hyphen_values = true)]
        text: String,
    },

    /// Move a task of done.txt back to the task file, pending again, by its number in done.txt
    Reopen {
        /// Number of the task in done.txt, as list --done shows it
        number: usize,
    },

    /// Move the done tasks to the end of done.txt, next to the task file
    Archive,
}

/// The flags of `list`, of which at most one is given.
#[derive(Args)]
#[group(multiple = false)]
pub struct ListFlags {
    /// Keep the pending tasks whose due date is today or past
    #[arg(long)]
    due: bool,

    /// List done.txt instead, in the order of the file
    #[arg(long)]
    done: bool,

    /// Keep only the pending tasks whose t: date is after today, which list leaves out otherwise
    #[arg(long)]
    sleeping: bool,
}

/// The tasks `list` shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Listing {
    Pending,
    Due,
    Done,
    Sleeping,
}

impl ListFlags {
    /// The listing its given flag names, the pending tasks without one.
    pub fn listing(&self) -> Listing {
        if self.due {
            return Listing::Due;
        }

        if self.done {
            return Listing::Done;
        }

        if self.sleeping {
            return Listing::Sleeping;
        }

        Listing::Pending
    }
}

/// The task a command names: by its number, or by a term.
#[derive(Clone)]
pub enum Task {
    Number(usize),
    Term(String),
}

/// An argument made only of digits as a number, any other as a term.
fn task(arg: &str) -> Result<Task, ParseIntError> {
    if arg.bytes().all(|byte| byte.is_ascii_digit()) {
        return arg.parse().map(Task::Number);
    }

    Ok(Task::Term(arg.to_owned()))
}

#[cfg(test)]
mod tests;
