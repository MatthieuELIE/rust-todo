use time::Date;

use crate::todo::Todo;

/// In-memory store of todo items.
pub struct Store {
    /// Tasks in file order; a task's number is its index plus one.
    pub todos: Vec<Todo>,
}

impl Store {
    /// Create a new store with the given list of todos.
    pub fn new(todos: Vec<Todo>) -> Self {
        Store { todos }
    }

    /// List tasks numbered by file position, filtered by status and terms, sorted by priority with done ones last.
    pub fn list(&self, all: bool, terms: &[String]) -> Vec<(usize, &Todo)> {
        let terms: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
        let mut tasks: Vec<(usize, &Todo)> = self
            .todos
            .iter()
            .enumerate()
            .map(|(i, todo)| (i + 1, todo))
            .filter(|(_, todo)| all || !todo.done)
            .filter(|(_, todo)| matches(&todo.to_line().to_lowercase(), &terms))
            .collect();
        tasks.sort_by_key(|(_, todo)| (todo.done, todo.priority.is_none(), todo.priority));
        tasks
    }

    /// Add a new todo to the store.
    pub fn add(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    /// Remove a todo by its number, returning true if it was found and removed.
    pub fn remove(&mut self, number: usize) -> bool {
        match self.index_of(number) {
            Some(index) => {
                self.todos.remove(index);
                true
            }
            None => false,
        }
    }

    /// Mark a todo as done by its number, returning true if it was found and marked.
    pub fn done(&mut self, number: usize, today: Date) -> bool {
        match self.index_of(number) {
            Some(index) => {
                self.todos[index].complete(today);
                true
            }
            None => false,
        }
    }

    /// Convert a 1-based task number to a 0-based index, returning None if out of range.
    fn index_of(&self, number: usize) -> Option<usize> {
        (1..=self.todos.len()).contains(&number).then(|| number - 1)
    }
}

/// Whether the line contains every term, or lacks it when the term starts with a dash.
fn matches(line: &str, terms: &[String]) -> bool {
    terms.iter().all(|term| match term.strip_prefix('-') {
        Some(term) => !line.contains(term),
        None => line.contains(term.as_str()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn store_of(lines: &[&str]) -> Store {
        Store::new(lines.iter().map(|l| Todo::from_line(l)).collect())
    }

    #[test]
    fn remove_deletes_the_task_at_that_number() {
        let mut store = store_of(&["Lorem ipsum", "Consectetur adipiscing", "Sed do eiusmod"]);

        assert!(store.remove(2));

        let lines: Vec<String> = store.todos.iter().map(Todo::to_line).collect();
        assert_eq!(lines, ["Lorem ipsum", "Sed do eiusmod"]);
    }

    #[test]
    fn done_completes_the_task_at_that_number() {
        let mut store = store_of(&["(A) 2026-08-01 Lorem ipsum", "Consectetur adipiscing"]);

        assert!(store.done(1, date!(2026 - 09 - 01)));

        assert_eq!(store.todos[0].to_line(), "x 2026-09-01 2026-08-01 Lorem ipsum");
        assert!(!store.todos[1].done);
    }

    #[test]
    fn completing_a_task_without_a_creation_date_records_only_the_completion() {
        let mut store = store_of(&["Lorem ipsum"]);

        store.done(1, date!(2026 - 09 - 01));

        assert_eq!(store.todos[0].to_line(), "x 2026-09-01 Lorem ipsum");
    }

    #[test]
    fn an_out_of_range_number_changes_nothing_and_returns_false() {
        let mut store = store_of(&["Lorem ipsum"]);

        assert!(!store.remove(0));
        assert!(!store.remove(2));
        assert!(!store.done(2, date!(2026 - 09 - 01)));
        assert_eq!(store.todos.len(), 1);
    }

    fn listed(store: &Store, all: bool, terms: &[&str]) -> Vec<(usize, String)> {
        let terms: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
        store.list(all, &terms).into_iter().map(|(n, t)| (n, t.to_line())).collect()
    }

    #[test]
    fn list_hides_done_tasks_unless_all_but_keeps_their_numbers() {
        let store = store_of(&["x Lorem ipsum", "Consectetur adipiscing", "Tempor incididunt"]);

        assert_eq!(
            listed(&store, false, &[]),
            [(2, "Consectetur adipiscing".into()), (3, "Tempor incididunt".into())]
        );
        assert_eq!(listed(&store, true, &[]).len(), 3);
    }

    #[test]
    fn every_term_must_appear_in_the_line() {
        let store = store_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

        assert_eq!(listed(&store, false, &["+finance", "Call"]), [(1, "Call the bank +finance".into())]);
    }

    #[test]
    fn a_leading_dash_excludes_lines_containing_the_term() {
        let store = store_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

        assert_eq!(listed(&store, false, &["+finance", "-@home"]), [(1, "Call the bank +finance".into())]);
    }

    #[test]
    fn terms_ignore_case() {
        let store = store_of(&["Call the BANK", "Pay rent @Home"]);

        assert_eq!(listed(&store, false, &["bank"]), [(1, "Call the BANK".into())]);
        assert_eq!(listed(&store, false, &["-@HOME"]), [(1, "Call the BANK".into())]);
    }

    #[test]
    fn list_puts_priorities_first_then_unprioritised_then_done_each_in_file_order() {
        let store = store_of(&["Lorem", "x Ipsum", "(B) Dolor", "Sit", "(A) Amet", "(B) Elit"]);

        let numbers: Vec<usize> = listed(&store, true, &[]).into_iter().map(|(n, _)| n).collect();

        assert_eq!(numbers, [5, 3, 6, 1, 4, 2]);
    }
}
