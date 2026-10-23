use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::todo::{DUE, WAIT};

mod find;
mod normal;
mod words;

/// What a key did to the line being edited.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    /// Keep editing.
    Continue,
    /// The text is to be saved.
    Submit,
    /// The text is to be dropped.
    Cancel,
    /// The key after `p` was not a priority, and did nothing.
    NotPriority,
    /// What an undo or a redo did, to be said in the status bar.
    Message(&'static str),
}

/// How keys act on the text, as in vim.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Mode {
    /// Keys are typed into the text.
    #[default]
    Insert,
    /// Keys move the cursor and change the text by commands.
    Normal,
}

/// A one-line text field with a cursor, edited with a small subset of vim.
#[derive(Default)]
pub struct Editor {
    /// The text typed so far.
    pub text: String,
    /// Cursor position in characters: before the next typed character in insert mode, on a character in normal mode.
    pub cursor: usize,
    /// Whether keys are typed or are commands.
    pub mode: Mode,
    /// Key (`d`, `c`, `p`) waiting for its motion or priority; after `d` or `c`, a key completing no command only drops it.
    pending: Option<char>,
    /// `i` or `a` typed after `d` or `c`, waiting for the `w` or `W` of its word.
    scope: Option<char>,
    /// `f`, `F`, `t` or `T` waiting for the character to find.
    finding: Option<char>,
    /// Last search as its key and character, kept even when it failed, for `;` and `,`.
    find: Option<(char, char)>,
    /// Text and cursor when normal mode last had no command under way: what `u` goes back to after the next change.
    rest: (String, usize),
    /// Texts and cursors before each change, the latest last.
    undo: Vec<(String, usize)>,
    /// Texts and cursors before each `u`, the latest last.
    redo: Vec<(String, usize)>,
}

impl Editor {
    /// Opens on `text` in normal mode, the cursor at the start.
    pub fn new(text: String) -> Self {
        Editor {
            rest: (text.clone(), 0),
            text,
            mode: Mode::Normal,
            ..Editor::default()
        }
    }

    /// Applies one key press to the text and tells whether editing goes on.
    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let modified = key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        if self.pending == Some('p') && (modified || !matches!(key.code, KeyCode::Char('a'..='e' | ' '))) {
            self.pending = None;
            return Outcome::NotPriority;
        }
        let busy = self.pending.is_some() || self.finding.is_some();
        match (self.mode, key.code) {
            (Mode::Normal, KeyCode::Enter | KeyCode::Esc) if busy => (self.pending, self.scope, self.finding) = (None, None, None),
            (_, KeyCode::Enter) => return Outcome::Submit,
            (Mode::Normal, KeyCode::Esc) => return Outcome::Cancel,
            (Mode::Normal, KeyCode::Char('r')) if !busy && key.modifiers.contains(KeyModifiers::CONTROL) => return self.step(false),
            (Mode::Normal, _) if modified => (self.pending, self.scope, self.finding) = (None, None, None),
            (Mode::Normal, KeyCode::Char('u')) if !busy => return self.step(true),
            (Mode::Insert, _) => self.insert(key),
            (Mode::Normal, code) => self.normal(code),
        }
        self.settle();
        Outcome::Continue
    }

    /// Keeps the text before a change for `u`, once normal mode has no command under way: typing up to `Esc` is one change.
    fn settle(&mut self) {
        if self.mode == Mode::Insert || self.pending.is_some() {
            return;
        }
        if self.text == self.rest.0 {
            self.rest.1 = self.cursor;
        } else {
            self.undo.push(std::mem::replace(&mut self.rest, (self.text.clone(), self.cursor)));
            self.redo.clear();
        }
    }

    /// Undoes (`back`) or redoes the last change, keeping the current text for the way back, and tells what it did.
    fn step(&mut self, back: bool) -> Outcome {
        let (from, to, done, none) = if back {
            (&mut self.undo, &mut self.redo, "undone", "nothing to undo")
        } else {
            (&mut self.redo, &mut self.undo, "redone", "nothing to redo")
        };
        let Some(rest) = from.pop() else {
            return Outcome::Message(none);
        };
        to.push(std::mem::replace(&mut self.rest, rest));
        (self.text, self.cursor) = self.rest.clone();
        Outcome::Message(done)
    }

    /// Keys typed so far of a command still waiting for its end, as `d` or `ci`.
    pub fn pending(&self) -> String {
        self.pending.iter().chain(&self.scope).chain(&self.finding).collect()
    }

    /// Inserts `text` at the cursor, which moves past it, staying on a character in normal mode.
    pub fn paste(&mut self, text: &str) {
        self.text.insert_str(self.byte(self.cursor), text);
        self.cursor += text.chars().count();
        if self.mode == Mode::Normal {
            self.cursor = self.cursor.min(self.text.chars().count().saturating_sub(1));
        }
        self.settle();
    }

    /// The `+project`, `@context` or `wait:` value being typed before the cursor, in insert mode only.
    pub fn tag(&self) -> Option<&str> {
        let word = self.word();
        (self.mode == Mode::Insert && (word.starts_with(['+', '@']) || word.starts_with(WAIT))).then_some(word)
    }

    /// Whether the word before the cursor is a bare `due:` waiting for its date, in insert mode only.
    pub fn wants_date(&self) -> bool {
        self.mode == Mode::Insert && self.word() == DUE
    }

    /// The word before the cursor, empty after a space.
    fn word(&self) -> &str {
        self.text[..self.byte(self.cursor)].rsplit(char::is_whitespace).next().unwrap_or_default()
    }

    /// Writes `name` over the whole word at the cursor, then a space unless one follows, the cursor after it.
    pub fn complete(&mut self, name: &str) {
        let chars: Vec<char> = self.text.chars().collect();
        let start = chars[..self.cursor].iter().rposition(|c| c.is_whitespace()).map_or(0, |i| i + 1);
        let end = chars[self.cursor..]
            .iter()
            .position(|c| c.is_whitespace())
            .map_or(chars.len(), |i| self.cursor + i);
        self.remove(start, end);
        self.paste(name);
        if chars.get(end).is_some_and(|c| c.is_whitespace()) {
            self.cursor += 1;
        } else {
            self.paste(" ");
        }
    }

    /// Applies a key typed in insert mode.
    fn insert(&mut self, key: KeyEvent) {
        let len = self.text.chars().count();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Char('w') if ctrl => {
                let before: Vec<char> = self.text.chars().take(self.cursor).collect();
                let spaces = before.iter().rev().take_while(|c| c.is_whitespace()).count();
                let word = before.iter().rev().skip(spaces).take_while(|c| !c.is_whitespace()).count();
                self.remove(self.cursor - spaces - word, self.cursor);
            }
            KeyCode::Char('u') if ctrl => self.remove(0, self.cursor),
            KeyCode::Char(c) if !ctrl => {
                self.text.insert(self.byte(self.cursor), c);
                self.cursor += 1;
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(len),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = len,
            KeyCode::Backspace if self.cursor > 0 => self.remove(self.cursor - 1, self.cursor),
            KeyCode::Delete if self.cursor < len => self.remove(self.cursor, self.cursor + 1),
            _ => {}
        }
    }

    /// Erases the characters from position `start` up to `end`, leaving the cursor at `start`.
    fn remove(&mut self, start: usize, end: usize) {
        let range = self.byte(start)..self.byte(end);
        self.text.replace_range(range, "");
        self.cursor = start;
    }

    /// Byte offset of the character at `position`, the text's length past its end.
    fn byte(&self, position: usize) -> usize {
        self.text.char_indices().nth(position).map_or(self.text.len(), |(i, _)| i)
    }
}

#[cfg(test)]
mod tests;
