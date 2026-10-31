use super::*;

impl App {
    /// Adds the task typed as `text`, cursor on it, the panel term appended when missing; a rejected text is told in a toast.
    pub(super) fn add(&mut self, text: &str, today: Date) -> bool {
        match Todo::new_from_input(text, today) {
            Ok(mut todo) => {
                if self.filter.as_deref() == Some(WAITING) {
                    self.filter = None;
                } else if self.filter.as_deref() == Some(DUE_NOW) {
                    let dated = |word: &str| word.strip_prefix(DUE).is_some_and(|value| !value.is_empty());
                    if !todo.description.split_whitespace().any(dated) {
                        todo.description = format!("{} {DUE}{today}", todo.description);
                    }
                } else if let Some(term) = &self.filter
                    && !has_word(&todo, term)
                {
                    todo.description = format!("{} {term}", todo.description);
                }
                self.todos.push(todo);
                let hidden = !self.follow(self.todos.len());
                self.notify(Notice::Added { hidden });
                true
            }
            Err(e) => {
                self.notify(Notice::Refused(e));
                false
            }
        }
    }

    /// Replaces task `number` with `text`, or moves it to `done.txt`, done on `today`, when its `x` was typed; a task with no text word is refused, an unchanged line told in a toast.
    pub(super) fn edit(&mut self, number: usize, text: &str, today: Date) -> bool {
        let mut todo = match Todo::from_input(text) {
            Ok(todo) => todo,
            Err(e) => {
                self.notify(Notice::Refused(e));
                return false;
            }
        };
        if todo.done && !todo.description.is_empty() {
            if todo.created.is_none() && todo.completed == self.todos[number - 1].created {
                todo.created = todo.completed.take();
            }
            todo.complete(today);
            self.todos.remove(number - 1);
            self.moves.push(Move::Append(todo));
            self.notify(Notice::Done);
            return true;
        }
        if todo.to_line() == self.todos[number - 1].to_line() {
            self.notify(Notice::NoChanges);
            return false;
        }
        if let Err(e) = todo.check_text() {
            self.notify(Notice::Refused(e));
            return false;
        }
        self.todos[number - 1] = todo;
        let hidden = !self.follow(number);
        self.notify(Notice::Updated { hidden });
        true
    }

    /// Brings task `number` of the history back, pending, at the end of the tasks, as `todo reopen` does and refuses.
    pub(super) fn reopen(&mut self, number: usize) -> bool {
        let mut todo = self.history[number - 1].clone();
        if let Err(reason) = todo.reopen() {
            self.notify(Notice::Refused(format!("task {number} {reason}")));
            return false;
        }
        self.todos.push(todo);
        self.moves.push(Move::Remove(self.history.remove(number - 1), Some(number)));
        self.notify(Notice::Reopened);
        true
    }

    /// Undoes (`back`) or redoes the last change, keeping the current tasks for the way back; says so when there is none.
    pub(super) fn step(&mut self, back: bool) -> bool {
        let (from, to, done, none) = if back {
            (&mut self.undo, &mut self.redo, Notice::ChangeUndone, Notice::NothingToUndo)
        } else {
            (&mut self.redo, &mut self.undo, Notice::ChangeRedone, Notice::NothingToRedo)
        };
        match from.pop() {
            Some((todos, moved)) => {
                to.push((std::mem::replace(&mut self.todos, todos), moved.clone()));
                self.moves.extend(moved.map(|moved| match moved {
                    Move::Append(todo) if back => Move::Remove(todo, None),
                    Move::Remove(todo, _) if back => Move::Append(todo),
                    moved => moved,
                }));
                self.notify(done);
                true
            }
            None => {
                self.notify(none);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests;
