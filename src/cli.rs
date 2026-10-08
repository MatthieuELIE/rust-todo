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
    /// List pending tasks (--all to include done ones, --done for done.txt), filtered by terms (-term excludes)
    #[command(alias = "ls")]
    List {
        /// Show all tasks, including done ones
        #[arg(short, long)]
        all: bool,

        /// Keep the pending tasks whose due date is today or past
        #[arg(long)]
        due: bool,

        /// List done.txt instead, in the order of the file
        #[arg(long, conflicts_with_all = ["all", "due"])]
        done: bool,

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

    /// Mark a task done by its number
    #[command(alias = "done")]
    Do {
        /// Task number to mark as done
        number: usize,
    },

    /// Replace a pending task by its number with a todo.txt line, keeping its creation date unless the line carries one
    Edit {
        /// Task number to edit
        number: usize,

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
    fn a_task_text_may_start_with_a_dash() {
        let command = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap();

        assert!(matches!(command(&["todo", "add", "-5 degrees"]), Commands::Add { text } if text == "-5 degrees"));
        assert!(matches!(
            command(&["todo", "edit", "2", "-5 degrees"]),
            Commands::Edit { number: 2, text } if text == "-5 degrees"
        ));
    }

    #[test]
    fn list_flags_come_before_terms_which_may_start_with_a_dash() {
        let terms_of = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
            Some(Commands::List { all, terms, .. }) => (all, terms),
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
        assert!(matches!(
            Cli::try_parse_from(["todo", "list", "+work", "--due"]).unwrap().command,
            Some(Commands::List { due: false, .. })
        ));
        assert!(Cli::try_parse_from(["todo", "list", "--done", "--all"]).is_err());
        assert!(Cli::try_parse_from(["todo", "list", "--done", "--due"]).is_err());
    }
}
