use super::*;

impl App {
    /// The tasks the list shows: those of the task file, or of `done.txt` in the history.
    fn source(&self) -> &[Todo] {
        if self.in_history { &self.history } else { &self.todos }
    }

    /// Tasks having `terms`, numbered: the pending ones of the task file by priority, or the history from its last line.
    pub(super) fn listed(&self, terms: &[String]) -> Vec<(usize, &Todo)> {
        let mut tasks = list(self.source(), terms);
        if self.in_history {
            tasks.sort_by_key(|(number, _)| Reverse(*number));
        } else {
            tasks.retain(|(_, todo)| !todo.done);
        }
        tasks
    }

    /// Tasks on screen, numbered and in display order.
    pub fn tasks(&self) -> Vec<(usize, &Todo)> {
        let terms: Vec<String> = self.search.split_whitespace().map(String::from).collect();
        let mut tasks = self.listed(&terms);
        if let Some(filter) = &self.filter {
            tasks.retain(|(_, todo)| shows(todo, filter, self.today));
        }
        tasks
    }

    /// Groups of the tasks on screen with their counts, in display order; none in the history or when no task on screen has a priority.
    pub(super) fn groups(&self) -> Vec<(Group, usize)> {
        let mut groups: Vec<(Group, usize)> = Vec::new();
        for (_, todo) in self.tasks() {
            let group = Group::of(todo);
            match groups.last_mut() {
                Some((last, count)) if *last == group => *count += 1,
                _ => groups.push((group, 1)),
            }
        }
        if self.in_history || !groups.iter().any(|(group, _)| matches!(group, Group::Priority(_))) {
            groups.clear();
        }
        groups
    }

    /// Lines of the list in display order: each group's header, then its tasks unless it is folded.
    pub fn shown(&self) -> Vec<Shown<'_>> {
        let mut tasks = self.tasks().into_iter().map(|(number, todo)| Shown::Task(number, todo));
        let mut shown = Vec::new();
        for (group, count) in self.groups() {
            let folded = self.folded.contains(&group);
            shown.push(Shown::Header(group, count, folded));
            shown.extend(tasks.by_ref().take(count).filter(|_| !folded));
        }
        shown.extend(tasks);
        shown
    }

    /// Rows the cursor can stand on, in display order: the tasks, except those of a folded group, which is one row.
    pub(super) fn rows(&self) -> Vec<Row> {
        let row = |shown| match shown {
            Shown::Task(number, _) => Some(Row::Task(number)),
            Shown::Header(group, _, true) => Some(Row::Group(group)),
            Shown::Header(..) => None,
        };
        self.shown().into_iter().filter_map(row).collect()
    }

    /// The task under the cursor, none when the cursor stands on a folded group or the list is empty.
    pub fn selected_task(&self) -> Option<&Todo> {
        match self.rows().get(self.cursor) {
            Some(Row::Task(number)) => Some(&self.source()[number - 1]),
            _ => None,
        }
    }

    /// Row of the first task on screen in `group`.
    fn first_task(&self, group: Group) -> Option<Row> {
        self.tasks()
            .iter()
            .find(|(_, todo)| Group::of(todo) == group)
            .map(|(number, _)| Row::Task(*number))
    }

    /// Puts the cursor on `row`, or on its group when that is folded, and tells whether either is on screen.
    fn select(&mut self, row: Row) -> bool {
        let group = match row {
            Row::Task(number) => Row::Group(Group::of(&self.source()[number - 1])),
            Row::Group(_) => row,
        };
        let rows = self.rows();
        match rows.iter().position(|r| *r == row).or_else(|| rows.iter().position(|r| *r == group)) {
            Some(position) => {
                self.cursor = position;
                true
            }
            None => false,
        }
    }

    /// Unfolds the groups no longer on screen, so they come back unfolded.
    pub(super) fn forget_gone_folds(&mut self) {
        let groups = self.groups();
        self.folded.retain(|folded| groups.iter().any(|(group, _)| group == folded));
    }

    /// Folds every group on screen, the cursor on the header of the group of `row`.
    pub(super) fn fold_all(&mut self, row: Option<Row>) {
        self.folded = self.groups().into_iter().map(|(group, _)| group).collect();
        if let Some(row) = row {
            self.select(row);
        }
    }

    /// Unfolds every group, the cursor back on the task of `row`, or on the first task of its group when `row` is a header.
    pub(super) fn unfold_all(&mut self, row: Option<Row>) {
        self.folded.clear();
        let target = match row {
            Some(Row::Group(group)) => self.first_task(group),
            row => row,
        };
        if let Some(target) = target {
            self.select(target);
        }
    }

    /// Folds the group of the task of `row` onto its header, or unfolds the header `row` onto its first task.
    pub(super) fn toggle_fold(&mut self, row: Option<Row>) {
        match row {
            Some(Row::Task(number)) if !self.groups().is_empty() => {
                self.folded.push(Group::of(&self.todos[number - 1]));
                self.select(Row::Task(number));
            }
            Some(Row::Group(group)) => {
                self.folded.retain(|folded| *folded != group);
                if let Some(first) = self.first_task(group) {
                    self.select(first);
                }
            }
            _ => {}
        }
    }

    /// Keeps the cursor on a row, on the last one when it was past the end.
    pub(super) fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.rows().len().saturating_sub(1));
    }

    /// Puts the cursor on task `number`, or on its folded group, or says `hidden` when neither is on screen.
    pub(super) fn follow(&mut self, number: usize, hidden: &str) {
        if !self.select(Row::Task(number)) {
            self.message = Some(hidden.to_string());
        }
    }
}

#[cfg(test)]
mod tests;
