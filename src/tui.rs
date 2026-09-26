use std::cmp::Reverse;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::time::Duration;

use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::widgets::ListState;
use time::Date;

use crate::editor::{Editor, Mode, Outcome};
use crate::repository;
use crate::store::Store;
use crate::todo::Todo;
use crate::view;

/// Said when the key after `p` is not a priority.
const NOT_PRIORITY: &str = "priority is a to e, or space";

/// What the popup's text becomes once submitted.
pub enum Target {
    /// A new task.
    Add,
    /// A new line for the task with this number.
    Edit(usize),
}

/// The centred field where a task is typed.
pub struct Popup {
    /// The text being typed.
    pub editor: Editor,
    /// What the text is for.
    pub target: Target,
    /// Row picked among the completions of the tag being typed.
    pub selected: usize,
}

/// Where the keys go.
pub enum Focus {
    /// The task list.
    List,
    /// The filter panel.
    Panel,
    /// The search line, in the status bar.
    Search,
    /// The key help over the list, closed by any key.
    Help,
    /// The popup where a task is typed.
    Popup(Popup),
}

/// A group of the list, in display order.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Group {
    /// Pending tasks with this priority.
    Priority(char),
    /// Pending tasks without a priority.
    Unprioritised,
    /// Done tasks.
    Done,
}

impl Group {
    /// The group `todo` is listed under.
    fn of(todo: &Todo) -> Group {
        match (todo.done, todo.priority) {
            (true, _) => Group::Done,
            (false, Some(letter)) => Group::Priority(letter),
            (false, None) => Group::Unprioritised,
        }
    }
}

/// A row of the list the cursor can stand on.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Row {
    /// The task with this number.
    Task(usize),
    /// A folded group, shown as its header.
    Group(Group),
}

/// State of the interactive list: the tasks and where the user stands in them.
pub struct App {
    /// The tasks being browsed.
    store: Store,
    /// Position of the selected row among the tasks on screen.
    pub cursor: usize,
    /// First key of a two-key command (`gg`, `dd`, `p` and a letter, `zM`, `zR`, `za`) waiting for its second key.
    pending: Option<char>,
    /// Whether done tasks are listed too.
    pub show_done: bool,
    /// Set once the user asked to leave.
    pub quit: bool,
    /// Last message for the status bar, cleared by the next key.
    pub message: Option<String>,
    /// Where the keys go.
    pub focus: Focus,
    /// Search terms, separated by whitespace, as `todo list` takes them.
    pub search: String,
    /// Term picked in the panel, `None` for `All tasks`.
    pub filter: Option<String>,
    /// Groups shown folded, as their header alone.
    pub folded: Vec<Group>,
    /// Tasks as they were before each change, the latest last.
    undo: Vec<Vec<Todo>>,
    /// Tasks as they were before each `u`, the latest last.
    redo: Vec<Vec<Todo>>,
}

impl App {
    /// Opens the list on its first row.
    pub fn new(store: Store) -> Self {
        App {
            store,
            cursor: 0,
            pending: None,
            show_done: false,
            quit: false,
            message: None,
            focus: Focus::List,
            search: String::new(),
            filter: None,
            folded: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Tasks on screen, numbered and in display order.
    pub fn tasks(&self) -> Vec<(usize, &Todo)> {
        let terms: Vec<String> = self.search.split_whitespace().map(String::from).collect();
        let mut tasks = self.store.list(self.show_done, &terms);
        if let Some(filter) = &self.filter {
            tasks.retain(|(_, todo)| has_word(todo, filter));
        }
        tasks
    }

    /// Groups of the tasks on screen with how many tasks each holds, in display order and without empty ones; none at all when
    /// no task on screen has a priority.
    pub fn groups(&self) -> Vec<(Group, usize)> {
        let mut groups: Vec<(Group, usize)> = Vec::new();
        for (_, todo) in self.tasks() {
            let group = Group::of(todo);
            match groups.last_mut() {
                Some((last, count)) if *last == group => *count += 1,
                _ => groups.push((group, 1)),
            }
        }
        if !groups.iter().any(|(group, _)| matches!(group, Group::Priority(_))) {
            groups.clear();
        }
        groups
    }

    /// Rows the cursor can stand on, in display order: the tasks, except those of a folded group, which is one row.
    fn rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        for (number, todo) in self.tasks() {
            let group = Group::of(todo);
            if !self.folded.contains(&group) {
                rows.push(Row::Task(number));
            } else if rows.last() != Some(&Row::Group(group)) {
                rows.push(Row::Group(group));
            }
        }
        rows
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
            Row::Task(number) => Row::Group(Group::of(&self.store.todos[number - 1])),
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

    /// Panel entries with how many tasks each shows, the search left out: `All tasks`, then the `+projects` and `@contexts` of
    /// the tasks shown, each alphabetical.
    pub fn filters(&self) -> Vec<(String, usize)> {
        let shown = self.store.list(self.show_done, &[]);
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
        for term in terms('+', Todo::projects).into_iter().chain(terms('@', Todo::contexts)) {
            let count = shown.iter().filter(|(_, todo)| has_word(todo, &term)).count();
            filters.push((term, count));
        }
        filters
    }

    /// Completions of the tag typed in the popup and the row picked among them: the `+projects` or `@contexts` of every task, done
    /// ones included, starting like the tag whatever the case, with how many tasks have each, the most used first, then
    /// alphabetical.
    pub fn completions(&self) -> (Vec<(String, usize)>, usize) {
        let Focus::Popup(popup) = &self.focus else {
            return Default::default();
        };
        let Some(tag) = popup.editor.tag() else {
            return Default::default();
        };
        let (sigil, names): (char, fn(&Todo) -> Vec<&str>) = if tag.starts_with('+') {
            ('+', Todo::projects)
        } else {
            ('@', Todo::contexts)
        };
        let mut counts = HashMap::new();
        for name in self.store.todos.iter().flat_map(names) {
            *counts.entry(format!("{sigil}{name}")).or_insert(0) += 1;
        }
        let tag = tag.to_lowercase();
        let mut names: Vec<(String, usize)> = counts.into_iter().filter(|(name, _)| name.to_lowercase().starts_with(&tag)).collect();
        names.sort_by_key(|(name, count)| (Reverse(*count), name.to_lowercase(), name.clone()));
        let selected = popup.selected.min(names.len().saturating_sub(1));
        (names, selected)
    }

    /// Row of the active filter among the panel `filters`, `None` when it is not among them.
    pub fn filter_row(&self, filters: &[(String, usize)]) -> Option<usize> {
        match &self.filter {
            None => Some(0),
            Some(filter) => filters.iter().position(|(term, _)| term == filter),
        }
    }

    /// Applies one key press to the state, dated `today`, and tells whether the file must be rewritten.
    /// A change, other than a step through the history, keeps the tasks as they were for `u`.
    pub fn handle_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let before = self.store.todos.clone();
        let history = (self.undo.len(), self.redo.len());
        let write = self.apply(key, today);
        self.forget_gone_folds();
        if write && history == (self.undo.len(), self.redo.len()) {
            self.undo.push(before);
            self.redo.clear();
        }
        write
    }

    /// Types pasted `text`, its lines joined by spaces, into the popup or the search line; anywhere else it does nothing, so a
    /// pasted line never acts as keys.
    pub fn paste(&mut self, text: &str) {
        let text = text.lines().collect::<Vec<_>>().join(" ");
        match &mut self.focus {
            Focus::Popup(popup) => popup.editor.paste(&text),
            Focus::Search => self.search.push_str(&text),
            _ => {}
        }
    }

    /// Unfolds the groups no longer on screen, so they come back unfolded.
    fn forget_gone_folds(&mut self) {
        let groups = self.groups();
        self.folded.retain(|folded| groups.iter().any(|(group, _)| group == folded));
    }

    /// Applies one key press to the state, dated `today`, and tells whether the file must be rewritten.
    fn apply(&mut self, key: KeyEvent, today: Date) -> bool {
        let pending = self.pending.take();
        self.message = None;
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL && !matches!(self.focus, Focus::Help) {
            self.quit = true;
            return false;
        }
        let write = match self.focus {
            Focus::Help => {
                self.focus = Focus::List;
                false
            }
            Focus::Popup(_) => self.popup_key(key, today),
            Focus::Search => {
                self.search_key(key);
                false
            }
            Focus::Panel => {
                self.panel_key(key);
                false
            }
            Focus::List => self.list_key(key, pending, today),
        };
        self.clamp_cursor();
        write
    }

    /// Applies a key typed in the popup, the arrows, `Ctrl-N`, `Ctrl-P` and `Tab` going to the completions while there are some,
    /// and tells whether the file must be rewritten.
    fn popup_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let (names, selected) = self.completions();
        let Focus::Popup(popup) = &mut self.focus else {
            unreachable!("the popup is open");
        };
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            _ if names.is_empty() => {}
            (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
                popup.selected = (selected + 1).min(names.len() - 1);
                return false;
            }
            (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
                popup.selected = selected.saturating_sub(1);
                return false;
            }
            (KeyCode::Tab, _) => {
                popup.editor.complete(&names[selected].0);
                popup.selected = 0;
                return false;
            }
            _ => {}
        }
        popup.selected = 0;
        match popup.editor.handle_key(key) {
            Outcome::Continue => false,
            Outcome::NotPriority => {
                self.message = Some(NOT_PRIORITY.to_string());
                false
            }
            outcome => self.close_popup(outcome, today),
        }
    }

    /// Closes the popup, adding or editing the task when its text was submitted, and tells whether the file must be rewritten.
    fn close_popup(&mut self, outcome: Outcome, today: Date) -> bool {
        let Focus::Popup(popup) = std::mem::replace(&mut self.focus, Focus::List) else {
            unreachable!("the popup is open");
        };
        match (outcome, popup.target) {
            (Outcome::Submit, Target::Add) => self.add(&popup.editor.text, today),
            (Outcome::Submit, Target::Edit(number)) => self.edit(number, &popup.editor.text),
            _ => false,
        }
    }

    /// Applies a key typed in the search line, the list back on its first row.
    fn search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => self.search.push(c),
            KeyCode::Backspace => _ = self.search.pop(),
            KeyCode::Esc => {
                self.search.clear();
                self.focus = Focus::List;
            }
            KeyCode::Enter => self.focus = Focus::List,
            _ => {}
        }
        self.cursor = 0;
    }

    /// Applies a key typed in the panel, the list back on its first row when the filter changes.
    fn panel_key(&mut self, key: KeyEvent) {
        let filters = self.filters();
        let row = self.filter_row(&filters).unwrap_or(0);
        let pick = |row: usize| (row > 0).then(|| filters[row].0.clone());
        let before = self.filter.clone();
        match key.code {
            KeyCode::Char('q') => self.quit = true,
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

    /// Applies a key typed in the list, after the first key of a two-key command when `pending`, and tells whether the file must
    /// be rewritten.
    fn list_key(&mut self, key: KeyEvent, pending: Option<char>, today: Date) -> bool {
        let rows = self.rows();
        let last = rows.len().saturating_sub(1);
        let row = rows.get(self.cursor).copied();
        let selected = match row {
            Some(Row::Task(number)) => Some(number),
            _ => None,
        };
        let mut write = false;
        match key.code {
            KeyCode::Char(c @ ('a'..='e' | ' ')) if pending == Some('p') => {
                let priority = (c != ' ').then(|| c.to_ascii_uppercase());
                if let Some(number) = selected
                    && let todo = &mut self.store.todos[number - 1]
                    && !todo.done
                    && todo.priority != priority
                {
                    todo.priority = priority;
                    self.follow(number, "priority set, hidden by the filter");
                    write = true;
                }
            }
            _ if pending == Some('p') => self.message = Some(NOT_PRIORITY.to_string()),
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
            KeyCode::Char('g') if pending == Some('g') => self.cursor = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => self.cursor = last,
            KeyCode::Char('x') => {
                if let Some(number) = selected {
                    let todo = &mut self.store.todos[number - 1];
                    if todo.done {
                        todo.reopen()
                    } else {
                        todo.complete(today)
                    }
                    write = true;
                }
            }
            KeyCode::Char('d') if pending == Some('d') => write = selected.is_some_and(|number| self.store.remove(number)),
            KeyCode::Char('d') => self.pending = Some('d'),
            KeyCode::Char('o') => {
                self.focus = Focus::Popup(Popup {
                    editor: Editor::default(),
                    target: Target::Add,
                    selected: 0,
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
                        editor: Editor::new(self.store.todos[number - 1].to_line(), Mode::Normal),
                        target: Target::Edit(number),
                        selected: 0,
                    })
                }
            }
            KeyCode::Tab => self.focus = Focus::Panel,
            KeyCode::Char('?') => self.focus = Focus::Help,
            KeyCode::Char('H') => {
                self.show_done = !self.show_done;
                self.cursor = 0;
            }
            _ => {}
        }
        write
    }

    /// Folds every group on screen, the cursor on the header of the group of `row`.
    fn fold_all(&mut self, row: Option<Row>) {
        self.folded = self.groups().into_iter().map(|(group, _)| group).collect();
        if let Some(row) = row {
            self.select(row);
        }
    }

    /// Unfolds every group, the cursor back on the task of `row`, or on the first task of its group when `row` is a header.
    fn unfold_all(&mut self, row: Option<Row>) {
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
    fn toggle_fold(&mut self, row: Option<Row>) {
        match row {
            Some(Row::Task(number)) if !self.groups().is_empty() => {
                self.folded.push(Group::of(&self.store.todos[number - 1]));
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
    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.rows().len().saturating_sub(1));
    }

    /// Adds the task typed as `text`, dated `today`, with the cursor on it; a rejected text only leaves a message.
    /// Under a panel filter the term is appended when the task would not match it.
    fn add(&mut self, text: &str, today: Date) -> bool {
        match Todo::new_from_input(text, today) {
            Ok(mut todo) => {
                if let Some(term) = &self.filter
                    && !has_word(&todo, term)
                {
                    todo.description = format!("{} {term}", todo.description);
                }
                self.store.add(todo);
                self.follow(self.store.todos.len(), "added, hidden by the filter");
                true
            }
            Err(e) => {
                self.message = Some(e);
                false
            }
        }
    }

    /// Replaces task `number` with the line `text` as typed, with the cursor on it; an empty description only leaves a message,
    /// and an unchanged line changes nothing.
    fn edit(&mut self, number: usize, text: &str) -> bool {
        let todo = Todo::from_line(text);
        if todo.description.trim().is_empty() {
            self.message = Some("a task needs a description".to_string());
            return false;
        }
        if text == self.store.todos[number - 1].to_line() {
            return false;
        }
        self.store.todos[number - 1] = todo;
        self.follow(number, "edited, hidden by the filter");
        true
    }

    /// Puts the cursor on task `number`, or on its folded group, or says `hidden` when neither is on screen.
    fn follow(&mut self, number: usize, hidden: &str) {
        if !self.select(Row::Task(number)) {
            self.message = Some(hidden.to_string());
        }
    }

    /// Brings back the tasks as they were before the last change (`back`) or before the last `u`, keeping the current ones for
    /// the way back; with no such state it only says so.
    fn step(&mut self, back: bool) -> bool {
        let (from, to, done, none) = if back {
            (&mut self.undo, &mut self.redo, "undone", "nothing to undo")
        } else {
            (&mut self.redo, &mut self.undo, "redone", "nothing to redo")
        };
        match from.pop() {
            Some(todos) => {
                to.push(std::mem::replace(&mut self.store.todos, todos));
                self.message = Some(done.to_string());
                true
            }
            None => {
                self.message = Some(none.to_string());
                false
            }
        }
    }

    /// Replaces the tasks with a fresh read of the file, keeping the cursor on its row and forgetting the history.
    /// An edit is cancelled, since its task number may now name another task.
    pub fn reload(&mut self, store: Store) {
        self.store = store;
        self.undo.clear();
        self.redo.clear();
        self.forget_gone_folds();
        self.clamp_cursor();
        self.message = Some("reloaded".to_string());
        if let Focus::Popup(Popup { target: Target::Edit(_), .. }) = self.focus {
            self.focus = Focus::List;
            self.message = Some("reloaded, edit cancelled".to_string());
        }
    }
}

/// Whether `term` is one of the words of `todo`'s description, case included, as the panel names a `+project` or an `@context`.
fn has_word(todo: &Todo, term: &str) -> bool {
    todo.description.split_whitespace().any(|word| word == term)
}

/// Runs the interactive list until the user quits, saving to `path` after every change.
/// `text` is the file as last read or written: when the file no longer matches it, the list is reloaded and the key ignored.
pub fn run(store: Store, mut text: String, path: &Path) -> io::Result<()> {
    let mut app = App::new(store);
    let mut scroll = ListState::default();
    ratatui::run(|terminal| {
        execute!(io::stdout(), EnableBracketedPaste)?;
        while !app.quit {
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll))?;
            let event = if event::poll(Duration::from_millis(250))? {
                Some(event::read()?)
            } else {
                None
            };
            if let Ok((current, todos)) = repository::load(path)
                && current != text
            {
                text = current;
                app.reload(Store::new(todos));
            } else if let Some(Event::Key(key)) = event
                && key.kind == KeyEventKind::Press
                && app.handle_key(key, crate::today())
            {
                match repository::save(path, &app.store.todos) {
                    Ok(written) => text = written,
                    Err(e) => {
                        if let Ok((_, todos)) = repository::load(path) {
                            app.reload(Store::new(todos));
                        }
                        app.message = Some(format!("could not save: {e} (file left unchanged)"));
                    }
                }
            } else if let Some(Event::Paste(pasted)) = event {
                app.paste(&pasted);
            }
        }
        execute!(io::stdout(), DisableBracketedPaste)
    })
}

/// Shows `message` full screen until a key is pressed, so a popup that closes when the program exits does not swallow it.
pub fn show_error(message: &str) -> io::Result<()> {
    ratatui::run(|terminal| {
        terminal.draw(|frame| view::draw_error(frame, message))?;
        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                return Ok(());
            }
        }
    })
}

#[cfg(test)]
mod tests;
