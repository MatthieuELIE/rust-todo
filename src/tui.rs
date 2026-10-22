use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::io;
use std::path::Path;

use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use time::{Date, Month};

use crate::editor::{Editor, Mode, Outcome};
use crate::repository;
use crate::theme::Theme;
use crate::todo::{DUE, Todo, WAIT, list};
use crate::view::HELP;

/// Said when the key after `p` is not a priority.
const NOT_PRIORITY: &str = "priority is a to e, or space";

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
    /// The key help over the list, scrolled down by this many lines.
    Help(u16),
    /// The popup where a task is typed.
    Popup(Popup),
}

/// What a change asks of `done.txt`, which is never rewritten from a copy held here.
#[derive(Clone)]
pub enum Move {
    /// Add this done task at its end.
    Append(Todo),
    /// Take this done task out of it: the task with this number when `r` reopened it, else the last line reading as it.
    Remove(Todo, Option<usize>),
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
    /// First key of a two-key command (`gg`, `dd`, `yy`, `p` and a letter, `zM`, `zR`, `za`) waiting for its second key.
    pending: Option<char>,
    /// Set once the user asked to leave.
    pub quit: bool,
    /// Last message for the status bar, cleared by the next key.
    pub message: Option<String>,
    /// Whether the message tells of something refused rather than done.
    pub refused: bool,
    /// Line of the task `yy` asked to copy, taken by the loop of the list.
    pub copy: Option<String>,
    /// Where the keys go.
    pub focus: Focus,
    /// Lines of the help the last drawing showed, which bound its scrolling.
    pub help_rows: Cell<u16>,
    /// Search terms, separated by whitespace, as `todo list` takes them.
    pub search: String,
    /// Term picked in the panel, `None` for `All tasks`.
    pub filter: Option<String>,
    /// Groups shown folded, as their header alone.
    folded: Vec<Group>,
    /// Tasks as they were before each change, the latest last, with what the change asked of `done.txt`.
    undo: Vec<(Vec<Todo>, Option<Move>)>,
    /// Tasks as they were before each `u`, the latest last, with what the change undone asked of `done.txt`.
    redo: Vec<(Vec<Todo>, Option<Move>)>,
    /// Moves not yet applied to `done.txt`, the oldest first.
    moves: Vec<Move>,
    /// Tasks of `done.txt` in file order, as last read.
    history: Vec<Todo>,
    /// Whether `done.txt` was read since the list opened, the last `H` or the last reload.
    history_read: bool,
    /// Whether the list shows `done.txt` in place of the task file.
    pub in_history: bool,
    /// Colours the list is drawn in.
    pub theme: Theme,
    /// Whether the panel and the detail zone carry Nerd Font icons.
    pub icons: bool,
}

impl App {
    /// Opens the list on its first row.
    pub fn new(todos: Vec<Todo>) -> Self {
        App { todos, ..App::default() }
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

    /// Applies one key dated `today`, keeping the tasks before a change for `u`; tells whether the file must be rewritten.
    pub fn handle_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let before = self.todos.clone();
        let history = (self.undo.len(), self.redo.len());
        let queued = self.moves.len();
        let write = self.apply(key, today);
        self.forget_gone_folds();
        if write && history == (self.undo.len(), self.redo.len()) {
            self.undo.push((before, self.moves.get(queued).cloned()));
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

    /// Applies one key press to the state, dated `today`, and tells whether the file must be rewritten.
    fn apply(&mut self, key: KeyEvent, today: Date) -> bool {
        let pending = self.pending.take();
        self.message = None;
        self.refused = false;
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
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
            Focus::Help(_) => {
                self.help_key(key, pending);
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

    /// Applies a key typed in the help: `j`/`k` and `gg`/`G` scroll within the lines the last drawing showed, `Esc`, `Enter`, `?` and `q` close.
    fn help_key(&mut self, key: KeyEvent, pending: Option<char>) {
        let Focus::Help(scroll) = &mut self.focus else {
            unreachable!("the help is open");
        };
        let max = (HELP.lines().count() as u16).saturating_sub(self.help_rows.get());
        match key.code {
            _ if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {}
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?' | 'q') => self.focus = Focus::List,
            KeyCode::Char('j') | KeyCode::Down => *scroll = (*scroll + 1).min(max),
            KeyCode::Char('k') | KeyCode::Up => *scroll = scroll.saturating_sub(1).min(max),
            KeyCode::Char('g') if pending == Some('g') => *scroll = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => *scroll = max,
            _ => {}
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
            KeyCode::Char('y') if pending == Some('y') => match self.selected_task() {
                Some(todo) => self.copy = Some(todo.to_line()),
                None => self.refuse("no task under the cursor"),
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
                        editor: Editor::new(self.todos[number - 1].to_line().replace(char::is_control, " ")),
                        target: Target::Edit(number),
                        selected: 0,
                        picker: None,
                    })
                }
            }
            KeyCode::Tab => self.focus = Focus::Panel,
            KeyCode::Char('?') => self.focus = Focus::Help(0),
            KeyCode::Char('H') => {
                self.in_history = !self.in_history;
                if matches!(self.filter.as_deref(), Some(DUE_NOW | WAITING)) {
                    self.filter = None;
                }
                self.history_read = false;
                self.cursor = 0;
            }
            KeyCode::Char('D') if self.in_history => self.refuse("history is read-only"),
            KeyCode::Char('D') if self.filters().iter().any(|(term, _)| term == DUE_NOW) && !self.tasks_due_in_search() => {
                self.message = Some("nothing due matches the search".to_string())
            }
            KeyCode::Char('D') if self.filters().iter().any(|(term, _)| term == DUE_NOW) => {
                if self.filter.as_deref() != Some(DUE_NOW) {
                    self.filter = Some(DUE_NOW.to_string());
                    self.cursor = 0;
                }
            }
            KeyCode::Char('D') => self.message = Some("nothing due".to_string()),
            _ => {}
        }
        write
    }

    /// Leaves `message` in the status bar as a refusal.
    pub fn refuse(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.refused = true;
    }
}

mod changes;
mod files;
mod panel;
mod popup;
mod rows;

pub use panel::{DUE_NOW, WAITING};
use panel::{has_word, shows};
pub use popup::{Popup, Target};

#[cfg(test)]
mod tests;
