use super::*;

/// Panel entry of the tasks waiting for something, right under `All tasks`.
pub const WAITING: &str = "Waiting";

/// Name of the panel entry listing the tasks due today or before.
pub const DUE_NOW: &str = "Due";

/// Panel entry of the Sleeping tasks, under `Waiting`, the only one that shows them.
pub const SLEEPING: &str = "Sleeping";

impl App {
    /// Whether a task due is left by the search, whatever the filter.
    pub(super) fn tasks_due_in_search(&self) -> bool {
        let terms: Vec<String> = self.search.split_whitespace().map(String::from).collect();
        self.listed(&terms).iter().any(|(_, todo)| shows(todo, Some(DUE_NOW), self.today))
    }

    /// Panel entries with their counts, search left out: `All tasks`, `Due`, `Waiting` and `Sleeping` if any task is and not in the history, then `+projects` and `@contexts`.
    pub fn filters(&self) -> Vec<(String, usize)> {
        let (shown, sleeping): (Vec<_>, Vec<_>) = self.listed(&[]).into_iter().partition(|(_, todo)| shows(todo, None, self.today));
        let terms = |sigil: char, names: fn(&Todo) -> Vec<&str>| {
            let mut terms: Vec<String> = shown
                .iter()
                .flat_map(|(_, todo)| names(todo))
                .map(|name| format!("{sigil}{name}"))
                .collect();
            terms.sort_by_key(|term| (term.to_lowercase(), term.clone()));
            terms.dedup();
            terms
        };
        let mut filters = vec![("All tasks".to_string(), shown.len())];
        let due = shown.iter().filter(|(_, todo)| shows(todo, Some(DUE_NOW), self.today)).count();

        if due > 0 && !self.in_history {
            filters.push((DUE_NOW.to_string(), due));
        }

        let waiting = shown.iter().filter(|(_, todo)| todo.is_waiting()).count();

        if waiting > 0 && !self.in_history {
            filters.push((WAITING.to_string(), waiting));
        }

        if !sleeping.is_empty() && !self.in_history {
            filters.push((SLEEPING.to_string(), sleeping.len()));
        }

        for term in terms('+', Todo::projects).into_iter().chain(terms('@', Todo::contexts)) {
            let count = shown.iter().filter(|(_, todo)| has_word(todo, &term)).count();
            filters.push((term, count));
        }
        filters
    }

    /// Row of the active filter among the panel `filters`, `None` when it is not among them.
    pub fn filter_row(&self, filters: &[(String, usize)]) -> Option<usize> {
        match &self.filter {
            None => Some(0),
            Some(filter) => filters.iter().position(|(term, _)| term == filter),
        }
    }

    /// Applies a key typed in the panel, the list back on its first row when the filter changes.
    pub(super) fn panel_key(&mut self, key: KeyEvent) {
        let filters = self.filters();
        let row = self.filter_row(&filters).unwrap_or(0);
        let pick = |row: usize| (row > 0).then(|| filters[row].0.clone());
        let before = self.filter.clone();

        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.open_help(),
            KeyCode::Char('j') | KeyCode::Down => self.filter = pick((row + 1).min(filters.len() - 1)),
            KeyCode::Char('k') | KeyCode::Up => self.filter = pick(row.saturating_sub(1)),
            KeyCode::Esc => self.filter = None,
            KeyCode::Tab | KeyCode::Enter => self.focus = Focus::List,
            _ => {}
        }

        if self.filter != before {
            self.cursor = 0;
        }
    }
}

/// Whether the panel entry `term`, `None` for `All tasks`, keeps `todo` on screen, a Sleeping task only under `Sleeping`.
pub(super) fn shows(todo: &Todo, term: Option<&str>, today: Option<Date>) -> bool {
    if today.is_some_and(|today| todo.is_sleeping(today)) {
        return term == Some(SLEEPING);
    }

    match term {
        None => true,
        Some(SLEEPING) => false,
        Some(WAITING) => todo.is_waiting(),
        Some(DUE_NOW) => today.is_some_and(|today| todo.is_due(today)),
        Some(term) => has_word(todo, term),
    }
}

/// The word a task added under the panel entry `term` lacks to show there: today's due date under `Due`, the term itself under a `+project` or an `@context`.
pub(super) fn missing_word(todo: &Todo, term: Option<&str>, today: Date) -> Option<String> {
    match term {
        None | Some(WAITING | SLEEPING) => None,
        Some(DUE_NOW) if has_due_date(todo) => None,
        Some(DUE_NOW) => Some(format!("{DUE}{today}")),
        Some(term) if has_word(todo, term) => None,
        Some(term) => Some(term.to_owned()),
    }
}

/// Whether `todo`'s description carries a `due:` word with a value.
fn has_due_date(todo: &Todo) -> bool {
    todo.description
        .split_whitespace()
        .any(|word| word.strip_prefix(DUE).is_some_and(|value| !value.is_empty()))
}

/// Whether `term` is one of the words of `todo`'s description, case included, as the panel names a `+project` or an `@context`.
fn has_word(todo: &Todo, term: &str) -> bool {
    todo.description.split_whitespace().any(|word| word == term)
}

#[cfg(test)]
mod tests;
