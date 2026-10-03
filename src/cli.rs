use clap::{Parser, Subcommand};

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
    /// List pending tasks (--all to include done ones), filtered by terms (-term excludes)
    #[command(alias = "ls")]
    List {
        /// Show all tasks, including done ones
        #[arg(short, long)]
        all: bool,

        /// Case-insensitive terms every listed task must contain, or lack with a leading -
        #[arg(allow_hyphen_values = true)]
        terms: Vec<String>,
    },

    /// Add a task, optionally as a todo.txt fragment ("(A) Buy milk +grocery")
    #[command(alias = "a")]
    Add {
        /// Task text, optionally with priority and projects
        text: String,
    },

    /// Remove a task by its number
    #[command(alias = "rm")]
    Remove {
        /// Task number to remove
        number: usize,
    },

    /// Mark a task done by its number
    #[command(alias = "done")]
    Do {
        /// Task number to mark as done
        number: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_answer_to_their_short_and_former_names() {
        let command = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap();

        assert!(matches!(command(&["todo", "ls"]), Commands::List { .. }));
        assert!(matches!(command(&["todo", "a", "Lorem"]), Commands::Add { .. }));
        assert!(matches!(command(&["todo", "rm", "1"]), Commands::Remove { number: 1 }));
        assert!(matches!(command(&["todo", "done", "1"]), Commands::Do { number: 1 }));
    }

    #[test]
    fn list_flags_come_before_terms_which_may_start_with_a_dash() {
        let terms_of = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
            Some(Commands::List { all, terms }) => (all, terms),
            _ => panic!("not a list command"),
        };

        assert_eq!(
            terms_of(&["todo", "list", "--all", "+work", "-@home"]),
            (true, vec!["+work".into(), "-@home".into()])
        );
        assert_eq!(
            terms_of(&["todo", "list", "+work", "--all"]),
            (false, vec!["+work".into(), "--all".into()])
        );
    }
}
