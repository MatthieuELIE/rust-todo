use super::*;

impl App {
    /// Applies a key typed in the list, `pending` being the first key of a two-key command; tells whether to write.
    pub(super) fn list_key(&mut self, key: KeyEvent, pending: Option<char>, today: Date) -> bool {
        let rows = self.rows();
        let last = rows.len().saturating_sub(1);
        let row = rows.get(self.cursor).copied();
        let selected = match row {
            Some(Row::Task(number)) => Some(number),
            _ => None,
        };
        let writes = match key.code {
            KeyCode::Char('x' | 'o' | 'p' | 'u') | KeyCode::Enter => true,
            KeyCode::Char('d') => pending == Some('d'),
            KeyCode::Char('r') => key.modifiers == KeyModifiers::CONTROL,
            _ => false,
        };
        if self.in_history && writes {
            self.notify(Notice::HistoryReadOnly);
            return false;
        }
        let mut write = false;
        match key.code {
            KeyCode::Char(c @ ('a'..='e' | ' ')) if pending == Some('p') => {
                let priority = (c != ' ').then(|| c.to_ascii_uppercase());
                if let Some(number) = selected
                    && let todo = &mut self.todos[number - 1]
                    && todo.priority != priority
                {
                    todo.priority = priority;
                    let hidden = !self.follow(number);
                    self.notify(Notice::PrioritySet { priority, hidden });
                    write = true;
                }
            }
            _ if pending == Some('p') => self.notify(Notice::NotPriority),
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.cursor = (self.cursor + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Char('p') => self.pending = Some('p'),
            KeyCode::Char('M') if pending == Some('z') => self.fold_all(row),
            KeyCode::Char('R') if pending == Some('z') => self.unfold_all(row),
            KeyCode::Char('a') if pending == Some('z') => self.toggle_fold(row),
            KeyCode::Char('z') => self.pending = Some('z'),
            KeyCode::Char('u') => write = self.step(true),
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => write = self.step(false),
            KeyCode::Char('r') if self.in_history && key.modifiers.is_empty() => {
                if let Some(number) = selected {
                    write = self.reopen(number);
                }
            }
            KeyCode::Char('g') if pending == Some('g') => self.cursor = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => self.cursor = last,
            KeyCode::Char('x') => {
                if let Some(number) = selected {
                    let mut todo = self.todos.remove(number - 1);
                    todo.complete(today);
                    self.moves.push(Move::Append(todo));
                    self.notify(Notice::Done);
                    write = true;
                }
            }
            KeyCode::Char('d') if pending == Some('d') => {
                if let Some(number) = selected {
                    self.todos.remove(number - 1);
                    self.notify(Notice::Deleted);
                    write = true;
                }
            }
            KeyCode::Char('d') => self.pending = Some('d'),
            KeyCode::Char('y') if pending == Some('y') => match self.selected_task() {
                Some(todo) => self.copy = Some(todo.to_line()),
                None => self.notify(Notice::NoTaskUnderCursor),
            },
            KeyCode::Char('y') => self.pending = Some('y'),
            KeyCode::Char('o') => {
                self.focus = Focus::Popup(Popup {
                    editor: Editor::default(),
                    target: Target::Add,
                    selected: 0,
                    picker: None,
                })
            }
            KeyCode::Char('/') => {
                self.search.clear();
                self.focus = Focus::Search;
                self.cursor = 0;
            }
            KeyCode::Esc => {
                self.search.clear();
                self.filter = None;
                self.cursor = 0;
            }
            KeyCode::Enter => {
                if let Some(number) = selected {
                    self.focus = Focus::Popup(Popup {
                        editor: Editor::new(self.todos[number - 1].to_line().replace(crate::todo::is_control, " ")),
                        target: Target::Edit(number),
                        selected: 0,
                        picker: None,
                    })
                }
            }
            KeyCode::Tab => self.focus = Focus::Panel,
            KeyCode::Char('?') => self.open_help(),
            KeyCode::Char('H') => {
                self.in_history = !self.in_history;
                if matches!(self.filter.as_deref(), Some(DUE_NOW | WAITING)) {
                    self.filter = None;
                }
                self.history_read = false;
                self.cursor = 0;
            }
            KeyCode::Char('D') if self.in_history => self.notify(Notice::HistoryReadOnly),
            KeyCode::Char('D') if self.filters().iter().any(|(term, _)| term == DUE_NOW) && !self.tasks_due_in_search() => {
                self.notify(Notice::NothingDueMatchesSearch)
            }
            KeyCode::Char('D') if self.filters().iter().any(|(term, _)| term == DUE_NOW) => {
                if self.filter.as_deref() != Some(DUE_NOW) {
                    self.filter = Some(DUE_NOW.to_string());
                    self.cursor = 0;
                }
            }
            KeyCode::Char('D') => self.notify(Notice::NothingDue),
            _ => {}
        }
        write
    }
}

#[cfg(test)]
mod tests;
