use crate::todo::Todo;

pub struct Store {
    pub todos: Vec<Todo>,
}

impl Store {
    pub fn new(todos: Vec<Todo>) -> Self {
        Store { todos }
    }

    pub fn list_all(&self) -> &[Todo] {
        &self.todos
    }

    pub fn list_pending(&self) -> impl Iterator<Item = (usize, &Todo)> {
        self.todos
            .iter()
            .enumerate()
            .filter(|(_, todo)| !todo.done)
            .map(|(i, todo)| (i + 1, todo))
    }

    pub fn add(&mut self, description: String, priority: Option<char>) {
        self.todos.push(Todo {
            description,
            done: false,
            priority,
        });
    }

    pub fn remove(&mut self, number: usize) -> bool {
        match self.index_of(number) {
            Some(index) => {
                self.todos.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn done(&mut self, number: usize) -> bool {
        match self.index_of(number) {
            Some(index) => {
                self.todos[index].done = true;
                true
            }
            None => false,
        }
    }

    fn index_of(&self, number: usize) -> Option<usize> {
        (1..=self.todos.len()).contains(&number).then(|| number - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_of(lines: &[&str]) -> Store {
        Store::new(lines.iter().map(|l| Todo::from_line(l)).collect())
    }

    #[test]
    fn add_appends_a_task_to_the_list() {
        let mut store = store_of(&["Lorem ipsum"]);

        store.add("Consectetur adipiscing".into(), Some('B'));

        assert_eq!(store.list_all().len(), 2);
        assert_eq!(store.list_all()[1].to_line(), "(B) Consectetur adipiscing");
    }

    #[test]
    fn remove_deletes_the_task_at_that_number() {
        let mut store = store_of(&["Lorem ipsum", "Consectetur adipiscing", "Sed do eiusmod"]);

        assert!(store.remove(2));

        let lines: Vec<String> = store.list_all().iter().map(Todo::to_line).collect();
        assert_eq!(lines, ["Lorem ipsum", "Sed do eiusmod"]);
    }

    #[test]
    fn done_marks_the_task_at_that_number() {
        let mut store = store_of(&["Lorem ipsum", "Consectetur adipiscing"]);

        assert!(store.done(1));

        assert!(store.list_all()[0].done);
        assert!(!store.list_all()[1].done);
    }

    #[test]
    fn an_out_of_range_number_changes_nothing_and_returns_false() {
        let mut store = store_of(&["Lorem ipsum"]);

        assert!(!store.remove(0));
        assert!(!store.remove(2));
        assert!(!store.done(2));
        assert_eq!(store.list_all().len(), 1);
    }

    #[test]
    fn list_pending_skips_done_tasks_but_keeps_their_numbers() {
        let store = store_of(&["x Lorem ipsum", "Consectetur adipiscing", "x Sed do", "Tempor incididunt"]);

        let pending: Vec<(usize, String)> = store.list_pending().map(|(n, t)| (n, t.to_line())).collect();

        assert_eq!(pending, [(2, "Consectetur adipiscing".into()), (4, "Tempor incididunt".into())]);
    }
}
