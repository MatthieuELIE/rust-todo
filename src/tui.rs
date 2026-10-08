use std::cmp::Reverse;
use std::collections::HashMap;
use std::io;
use std::path::Path;

use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use time::{Date, Month};

use crate::editor::{Editor, Mode, Outcome};
use crate::repository;
use crate::todo::{DUE, Todo, WAIT, list};

/// Panel entry of the tasks waiting for something, right under `All tasks`.
pub const WAITING: &str = "Waiting";

/// Name of the panel entry listing the tasks due today or before.
pub const DUE_NOW: &str = "Due";

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
    /// Date picked in the date picker, open while set.
    pub picker: Option<Date>,
}

/// Where the keys go.
#[derive(Default)]
pub enum Focus {
    /// The task list.
    #[default]
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

/// What a change asks of `done.txt`, which is never rewritten from a copy held here.
pub enum Move {
    /// Add this done task at its end.
    Append(Todo),
    /// Take the last line equal to this done task out of it.
    Remove(Todo),
}

/// A group of the list, in display order.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Group {
    /// Pending tasks with this priority.
    Priority(char),
    /// Pending tasks without a priority.
    Unprioritised,
}

impl Group {
    /// The group `todo` is listed under.
    fn of(todo: &Todo) -> Group {
        match todo.priority {
            Some(letter) => Group::Priority(letter),
            None => Group::Unprioritised,
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

/// A line of the list as drawn.
pub enum Shown<'a> {
    /// A group's header, with its count and whether it is folded.
    Header(Group, usize, bool),
    /// A task with its number.
    Task(usize, &'a Todo),
}

/// State of the interactive list: the tasks and where the user stands in them.
#[derive(Default)]
pub struct App {
    /// Tasks in file order; a task's number is its index plus one.
    pub todos: Vec<Todo>,
    /// The day the due dates are read against, brought by each turn of the loop; no task is due without it.
    pub today: Option<Date>,
    /// Position of the selected row among the tasks on screen.
    pub cursor: usize,
    /// First key of a two-key command (`gg`, `dd`, `p` and a letter, `zM`, `zR`, `za`) waiting for its second key.
    pending: Option<char>,
    /// Set once the user asked to leave.
    pub quit: bool,
    /// Last message for the status bar, cleared by the next key.
    pub message: Option<String>,
    /// Whether the message tells of something refused rather than done.
    pub refused: bool,
    /// Where the keys go.
    pub focus: Focus,
    /// Search terms, separated by whitespace, as `todo list` takes them.
    pub search: String,
    /// Term picked in the panel, `None` for `All tasks`.
    pub filter: Option<String>,
    /// Groups shown folded, as their header alone.
    folded: Vec<Group>,
    /// Tasks as they were before each change, the latest last, with the task the change moved to `done.txt`.
    undo: Vec<(Vec<Todo>, Option<Todo>)>,
    /// Tasks as they were before each `u`, the latest last, with the task `u` took back from `done.txt`.
    redo: Vec<(Vec<Todo>, Option<Todo>)>,
    /// Moves not yet applied to `done.txt`, the oldest first.
    moves: Vec<Move>,
    /// Tasks of `done.txt` in file order, as last read.
    history: Vec<Todo>,
    /// Whether `done.txt` was read since the list opened, the last `H` or the last reload.
    history_read: bool,
    /// Whether the list shows `done.txt` in place of the task file.
    pub in_history: bool,
}

impl App {
    /// Opens the list on its first row.
    pub fn new(todos: Vec<Todo>) -> Self {
        App { todos, ..App::default() }
    }

    /// The tasks the list shows: those of the task file, or of `done.txt` in the history.
    fn source(&self) -> &[Todo] {
        if self.in_history { &self.history } else { &self.todos }
    }

    /// Tasks having `terms`, numbered: the pending ones of the task file by priority, or the history from its last line.
    fn listed(&self, terms: &[String]) -> Vec<(usize, &Todo)> {
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
    fn groups(&self) -> Vec<(Group, usize)> {
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
    fn rows(&self) -> Vec<Row> {
        let row = |shown| match shown {
            Shown::Task(number, _) => Some(Row::Task(number)),
            Shown::Header(group, _, true) => Some(Row::Group(group)),
            Shown::Header(..) => None,
        };
        self.shown().into_iter().filter_map(row).collect()
    }

    /// Keys typed so far of a command still waiting for its end, in the popup or the list.
    pub fn pending(&self) -> String {
        match &self.focus {
            Focus::Popup(popup) => popup.editor.pending(),
            _ => self.pending.iter().collect(),
        }
    }

    /// The terminal cursor's shape: a bar while typing in the popup, a block otherwise, as in Neovim.
    pub fn cursor_shape(&self) -> SetCursorStyle {
        match &self.focus {
            Focus::Popup(popup) if popup.editor.mode == Mode::Insert => SetCursorStyle::SteadyBar,
            _ => SetCursorStyle::SteadyBlock,
        }
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

    /// Panel entries with their counts, search left out: `All tasks`, `Due` and `Waiting` if any task is and not in the history, then `+projects` and `@contexts`.
    pub fn filters(&self) -> Vec<(String, usize)> {
        let shown = self.listed(&[]);
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
        let due = shown.iter().filter(|(_, todo)| shows(todo, DUE_NOW, self.today)).count();
        if due > 0 && !self.in_history {
            filters.push((DUE_NOW.to_string(), due));
        }
        let waiting = shown.iter().filter(|(_, todo)| todo.is_waiting()).count();
        if waiting > 0 && !self.in_history {
            filters.push((WAITING.to_string(), waiting));
        }
        for term in terms('+', Todo::projects).into_iter().chain(terms('@', Todo::contexts)) {
            let count = shown.iter().filter(|(_, todo)| has_word(todo, &term)).count();
            filters.push((term, count));
        }
        filters
    }

    /// Names completing the popup's tag, from every task, with their counts, most used first; and the row picked among them.
    pub fn completions(&self) -> (Vec<(String, usize)>, usize) {
        let Focus::Popup(popup) = &self.focus else {
            return Default::default();
        };
        let Some(tag) = popup.editor.tag() else {
            return Default::default();
        };
        let (sigil, names): (&str, fn(&Todo) -> Vec<&str>) = match tag.chars().next() {
            Some('+') => ("+", Todo::projects),
            Some('@') => ("@", Todo::contexts),
            _ => (WAIT, Todo::waits),
        };
        let mut counts = HashMap::new();
        for name in self.todos.iter().flat_map(names) {
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

    /// Applies one key dated `today`, keeping the tasks before a change for `u`; tells whether the file must be rewritten.
    pub fn handle_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let before = self.todos.clone();
        let history = (self.undo.len(), self.redo.len());
        let queued = self.moves.len();
        let write = self.apply(key, today);
        self.forget_gone_folds();
        if write && history == (self.undo.len(), self.redo.len()) {
            let moved = match self.moves.get(queued) {
                Some(Move::Append(todo)) => Some(todo.clone()),
                _ => None,
            };
            self.undo.push((before, moved));
            self.redo.clear();
        }
        write
    }

    /// Types pasted `text` as one line into the popup or the search, and drops it elsewhere so it never acts as keys.
    pub fn paste(&mut self, text: &str) {
        let text = text.split(char::is_control).filter(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
        match &mut self.focus {
            Focus::Popup(popup) if popup.picker.is_none() => popup.editor.paste(&text),
            Focus::Search => self.search.push_str(&text),
            _ => {}
        }
        self.forget_gone_folds();
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
        self.refused = false;
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL && !matches!(self.focus, Focus::Help) {
            self.quit = true;
            return false;
        }
        if matches!(self.focus, Focus::List | Focus::Panel)
            && key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            && key.code != KeyCode::Char('r')
        {
            if pending == Some('p') {
                self.refuse(NOT_PRIORITY);
            }
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

    /// Applies a key typed in the popup, to the date picker when open, arrows and `Tab` to the completions when shown; tells whether to write.
    fn popup_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let (names, selected) = self.completions();
        let Focus::Popup(popup) = &mut self.focus else {
            unreachable!("the popup is open");
        };
        if let Some(date) = popup.picker {
            popup.picker = match key.code {
                _ if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => Some(date),
                KeyCode::Enter => {
                    popup.editor.complete(&format!("{DUE}{date}"));
                    None
                }
                KeyCode::Esc => None,
                code => Some(shift(date, code)),
            };
            return false;
        }
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
        let outcome = popup.editor.handle_key(key);
        if popup.editor.wants_date() {
            popup.picker = Some(today);
        }
        match outcome {
            Outcome::Continue => false,
            Outcome::NotPriority => {
                self.refuse(NOT_PRIORITY);
                false
            }
            Outcome::Message(message) => {
                self.message = Some(message.to_string());
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
            (Outcome::Submit, Target::Edit(number)) => self.edit(number, &popup.editor.text, today),
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

    /// Applies a key typed in the list, `pending` being the first key of a two-key command; tells whether to write.
    fn list_key(&mut self, key: KeyEvent, pending: Option<char>, today: Date) -> bool {
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
            self.refuse("history is read-only");
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
                    self.follow(number, "priority set, hidden by the filter");
                    write = true;
                }
            }
            _ if pending == Some('p') => self.refuse(NOT_PRIORITY),
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
                    let mut todo = self.todos.remove(number - 1);
                    todo.complete(today);
                    self.moves.push(Move::Append(todo));
                    write = true;
                }
            }
            KeyCode::Char('d') if pending == Some('d') => {
                if let Some(number) = selected {
                    self.todos.remove(number - 1);
                    write = true;
                }
            }
            KeyCode::Char('d') => self.pending = Some('d'),
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
                        editor: Editor::new(self.todos[number - 1].to_line().replace(char::is_control, " ")),
                        target: Target::Edit(number),
                        selected: 0,
                        picker: None,
                    })
                }
            }
            KeyCode::Tab => self.focus = Focus::Panel,
            KeyCode::Char('?') => self.focus = Focus::Help,
            KeyCode::Char('H') => {
                self.in_history = !self.in_history;
                self.history_read = false;
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
    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.rows().len().saturating_sub(1));
    }

    /// Adds the task typed as `text`, cursor on it, the panel term appended when missing; a rejected text leaves a message.
    fn add(&mut self, text: &str, today: Date) -> bool {
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
                self.follow(self.todos.len(), "added, hidden by the filter");
                true
            }
            Err(e) => {
                self.refuse(&e);
                false
            }
        }
    }

    /// Replaces task `number` with `text`, or moves it to `done.txt`, done on `today`, when its `x` was typed; an empty description is refused, an unchanged line ignored.
    fn edit(&mut self, number: usize, text: &str, today: Date) -> bool {
        let mut todo = match Todo::from_input(text) {
            Ok(todo) => todo,
            Err(e) => {
                self.refuse(&e);
                return false;
            }
        };
        if todo.done {
            if todo.created.is_none() && todo.completed == self.todos[number - 1].created {
                todo.created = todo.completed.take();
            }
            todo.complete(today);
            self.todos.remove(number - 1);
            self.moves.push(Move::Append(todo));
            return true;
        }
        if todo.to_line() == self.todos[number - 1].to_line() {
            return false;
        }
        self.todos[number - 1] = todo;
        self.follow(number, "edited, hidden by the filter");
        true
    }

    /// Leaves `message` in the status bar as a refusal.
    pub fn refuse(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.refused = true;
    }

    /// Puts the cursor on task `number`, or on its folded group, or says `hidden` when neither is on screen.
    fn follow(&mut self, number: usize, hidden: &str) {
        if !self.select(Row::Task(number)) {
            self.message = Some(hidden.to_string());
        }
    }

    /// Undoes (`back`) or redoes the last change, keeping the current tasks for the way back; says so when there is none.
    fn step(&mut self, back: bool) -> bool {
        let (from, to, done, none) = if back {
            (&mut self.undo, &mut self.redo, "undone", "nothing to undo")
        } else {
            (&mut self.redo, &mut self.undo, "redone", "nothing to redo")
        };
        match from.pop() {
            Some((todos, moved)) => {
                to.push((std::mem::replace(&mut self.todos, todos), moved.clone()));
                self.moves.extend(moved.map(if back { Move::Remove } else { Move::Append }));
                self.message = Some(done.to_string());
                true
            }
            None => {
                self.message = Some(none.to_string());
                false
            }
        }
    }

    /// One turn of the loop: reloads when the file, `read` just now, no longer matches `text`, else applies `event`; tells whether to save.
    pub fn turn(&mut self, read: io::Result<String>, text: &mut String, event: Option<Event>, today: Date) -> bool {
        self.today = Some(today);
        match (read, event) {
            (Ok(current), _) if current != *text => {
                self.reload(repository::parse(&current));
                *text = current;
            }
            (read, Some(Event::Key(key))) if key.kind == KeyEventKind::Press && self.handle_key(key, today) => match read {
                Ok(_) => return true,
                Err(e) => self.refuse(&format!("could not read the file: {e} (file left unchanged)")),
            },
            (_, Some(Event::Paste(pasted))) => self.paste(&pasted),
            _ => {}
        }
        false
    }

    /// The save `turn` asked for, destination first: the tasks completed go to the end of `done.txt`, the list is saved to `path`, then the tasks `u` brought back leave `done.txt`.
    pub fn save(&mut self, path: &Path, text: &mut String) {
        let done_path = path.with_file_name("done.txt");
        let (mut gone, mut back) = (Vec::new(), Vec::new());
        for moved in std::mem::take(&mut self.moves) {
            match moved {
                Move::Append(todo) => gone.push(todo),
                Move::Remove(todo) => back.push(todo),
            }
        }
        let appended = if gone.is_empty() && back.is_empty() {
            Ok(())
        } else if repository::same_file(path, &done_path) {
            Err(io::Error::other("the task file is done.txt itself"))
        } else if gone.is_empty() {
            Ok(())
        } else {
            let writable = std::fs::OpenOptions::new().write(true).open(path);
            writable.and_then(|_| repository::append(&done_path, &gone))
        };
        let moved = !gone.is_empty() && appended.is_ok();
        let written = appended.and_then(|()| repository::save(path, &self.todos));
        let failed = written.as_ref().err().map(|e| e.to_string());
        let saved = failed.is_none();
        self.saved(written, text, repository::read(path));
        if let Some(e) = failed
            && moved
        {
            self.refuse(&format!("could not save: {e} (done.txt already holds the task)"));
        }
        for todo in back.iter().filter(|_| saved) {
            match repository::remove_last(&done_path, &todo.to_line()) {
                Ok(true) => {}
                Ok(false) => self.message = Some("undone, the task was no longer in done.txt".to_string()),
                Err(e) => self.refuse(&format!("undone, but done.txt still holds the task: {e}")),
            }
        }
    }

    /// What follows the save `turn` asked for: `text` takes what was `written`, or the tasks go back to the file, `reread` just now, and the failure is told.
    pub fn saved(&mut self, written: io::Result<String>, text: &mut String, reread: io::Result<String>) {
        match written {
            Ok(written) => *text = written,
            Err(e) => {
                if let Ok(current) = reread {
                    self.reload(repository::parse(&current));
                }
                self.refuse(&format!("could not save: {e} (file left unchanged)"));
            }
        }
    }

    /// Reads `done.txt`, next to the task file at `path`, when it was not read since the list opened, the last `H` or the last reload.
    pub fn refresh_history(&mut self, path: &Path) {
        if !std::mem::replace(&mut self.history_read, true) {
            self.set_history(repository::read(&path.with_file_name("done.txt")));
        }
    }

    /// Takes the tasks of `done.txt`, `read` just now, as the history; a failed read empties it and is told.
    pub fn set_history(&mut self, read: io::Result<String>) {
        self.history = repository::parse(&read.unwrap_or_else(|e| {
            self.refuse(&format!("could not read done.txt: {e}"));
            String::new()
        }));
    }

    /// Replaces the tasks with a fresh read, cursor on its row, history dropped; an edit is cancelled as its number may shift.
    pub fn reload(&mut self, todos: Vec<Todo>) {
        self.todos = todos;
        self.undo.clear();
        self.redo.clear();
        self.moves.clear();
        self.history_read = false;
        self.refused = false;
        self.forget_gone_folds();
        self.clamp_cursor();
        self.message = Some("reloaded".to_string());
        if let Focus::Popup(Popup { target: Target::Edit(_), .. }) = self.focus {
            self.focus = Focus::List;
            self.message = Some("reloaded, edit cancelled".to_string());
        }
    }
}

/// `date` moved by a date picker key: a day with `h` `l`, a week with `k` `j`, a month with `H` `L`, arrows as their letters.
fn shift(date: Date, code: KeyCode) -> Date {
    match code {
        KeyCode::Char('h') | KeyCode::Left => date.previous_day().unwrap_or(date),
        KeyCode::Char('l') | KeyCode::Right => date.next_day().unwrap_or(date),
        KeyCode::Char('k') | KeyCode::Up => date.checked_sub(time::Duration::WEEK).unwrap_or(date),
        KeyCode::Char('j') | KeyCode::Down => date.checked_add(time::Duration::WEEK).unwrap_or(date),
        KeyCode::Char('H') => add_months(date, -1),
        KeyCode::Char('L') => add_months(date, 1),
        _ => date,
    }
}

/// `date` moved by `months`, its day brought back to the last one of a shorter month.
fn add_months(date: Date, months: i32) -> Date {
    let index = date.year() * 12 + i32::from(u8::from(date.month())) - 1 + months;
    let year = index.div_euclid(12);
    let month = Month::try_from(index.rem_euclid(12) as u8 + 1).expect("a month is 1 to 12");
    Date::from_calendar_date(year, month, date.day().min(month.length(year))).unwrap_or(date)
}

/// Whether the panel entry `term` keeps `todo` on screen.
fn shows(todo: &Todo, term: &str, today: Option<Date>) -> bool {
    match term {
        WAITING => todo.is_waiting(),
        DUE_NOW => today.is_some_and(|today| todo.is_due(today)),
        _ => has_word(todo, term),
    }
}

/// Whether `term` is one of the words of `todo`'s description, case included, as the panel names a `+project` or an `@context`.
fn has_word(todo: &Todo, term: &str) -> bool {
    todo.description.split_whitespace().any(|word| word == term)
}

#[cfg(test)]
mod tests;
