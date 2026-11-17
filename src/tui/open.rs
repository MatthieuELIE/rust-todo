use super::*;

/// What `gx` asked to open: a program to start on a value, with the task's projects and contexts.
#[derive(Clone, PartialEq, Debug)]
pub struct Opening {
    /// The program set for the key.
    pub program: PathBuf,
    /// The value of the key:value, the program's only argument.
    pub value: String,
    /// The task's projects, without their `+`, separated by spaces.
    pub projects: String,
    /// The task's contexts, without their `@`, separated by spaces.
    pub contexts: String,
}

impl App {
    /// Leaves to open the program set for a key:value of the task under the cursor, or says there is no task there.
    pub(super) fn open_selected(&mut self) {
        match self.selected_task().map(|todo| opening(todo, &self.programs)) {
            Some(opening) => self.open = opening,
            None => self.notify(Notice::NoTaskUnderCursor),
        }
    }
}

/// The program `programs` sets for the first key:value of `todo` whose key has one, with its value and the task's projects and contexts.
fn opening(todo: &Todo, programs: &HashMap<String, PathBuf>) -> Option<Opening> {
    todo.description
        .split_whitespace()
        .filter(|word| Todo::is_key_value(word))
        .filter_map(|word| word.split_once(':'))
        .find_map(|(key, value)| {
            programs.get(key).map(|program| Opening {
                program: program.clone(),
                value: value.to_string(),
                projects: todo.projects().join(" "),
                contexts: todo.contexts().join(" "),
            })
        })
}

#[cfg(test)]
mod tests;
