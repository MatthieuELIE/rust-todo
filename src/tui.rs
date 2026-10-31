use std::cell::Cell;
use std::cmp::Reverse;
use std::io;
use std::path::Path;

use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use time::Date;

use crate::editor::{Editor, Mode, Outcome};
use crate::filter::list;
use crate::repository;
use crate::theme::Theme;
use crate::todo::{DUE, Todo};

const MAX_UNDO: usize = 100;

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
    /// Where the keys go back to when the help closes: what the help covers while it is open.
    pub under_help: Focus,
    /// Lines of the help the last drawing left out below its box, which bound its scrolling.
    pub help_max: Cell<u16>,
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
            if self.undo.len() == MAX_UNDO {
                self.undo.remove(0);
            }
            self.undo.push((before, self.moves.get(queued).cloned()));
            self.redo.clear();
        }
        write
    }

    /// Types pasted `text` as one line into the popup or the search, and drops it elsewhere so it never acts as keys.
    pub fn paste(&mut self, text: &str) {
        let text = text
            .split(crate::todo::is_control)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
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

    /// Leaves `message` in the status bar as a refusal.
    pub fn refuse(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.refused = true;
    }
}

mod changes;
mod completions;
mod files;
mod help;
mod list;
mod panel;
mod picker;
mod popup;
mod rows;
mod search;

pub use panel::{DUE_NOW, WAITING};
use panel::{has_word, shows};
pub use popup::{Popup, Target};

#[cfg(test)]
mod tests;
