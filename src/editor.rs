use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::todo::Todo;

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
    /// Position of the cursor, in characters: in insert mode the next character typed goes before the one at this position, in
    /// normal mode it is on a character.
    pub cursor: usize,
    /// Whether keys are typed or are commands.
    pub mode: Mode,
    /// Key (`d`, `c`, `p`) waiting for its motion or priority.
    pending: Option<char>,
}

impl Editor {
    /// Opens on `text` in `mode`, the cursor at the start.
    pub fn new(text: String, mode: Mode) -> Self {
        Editor {
            text,
            mode,
            ..Editor::default()
        }
    }

    /// Applies one key press to the text and tells whether editing goes on.
    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if self.pending == Some('p') && !matches!(key.code, KeyCode::Char('a'..='e' | ' ')) {
            self.pending = None;
            return Outcome::NotPriority;
        }
        match (self.mode, key.code) {
            (_, KeyCode::Enter) => return Outcome::Submit,
            (Mode::Normal, KeyCode::Esc) => return Outcome::Cancel,
            (Mode::Insert, _) => self.insert(key),
            (Mode::Normal, code) => self.normal(code),
        }
        Outcome::Continue
    }

    /// Inserts `text` at the cursor, which moves past it, staying on a character in normal mode.
    pub fn paste(&mut self, text: &str) {
        self.text.insert_str(self.byte(self.cursor), text);
        self.cursor += text.chars().count();
        if self.mode == Mode::Normal {
            self.cursor = self.cursor.min(self.text.chars().count().saturating_sub(1));
        }
    }

    /// The `+project` or `@context` being typed: in insert mode, the word before the cursor when it starts with `+` or `@`.
    pub fn tag(&self) -> Option<&str> {
        let word = self.text[..self.byte(self.cursor)].rsplit(char::is_whitespace).next()?;
        (self.mode == Mode::Insert && word.starts_with(['+', '@'])).then_some(word)
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

    /// Applies a key typed in normal mode, then keeps the cursor on a character if it stays there.
    fn normal(&mut self, code: KeyCode) {
        let chars: Vec<char> = self.text.chars().collect();
        let (len, at) = (chars.len(), self.cursor);
        let last = len.saturating_sub(1);
        let pending = self.pending.take();
        match code {
            KeyCode::Char(w @ ('w' | 'W')) if pending == Some('d') => self.remove(at, next_start(&chars, at, w == 'W')),
            KeyCode::Char(w @ ('w' | 'W')) if pending == Some('c') => {
                let big = w == 'W';
                let end = match chars.get(at) {
                    Some(c) if !c.is_whitespace() => run_end(&chars, at, big) + 1,
                    _ => next_start(&chars, at, big),
                };
                self.remove(at, end);
                self.mode = Mode::Insert;
            }
            KeyCode::Char(c @ ('a'..='e' | ' ')) if pending == Some('p') => self.set_priority(c),
            KeyCode::Char(op @ ('d' | 'c' | 'p')) => self.pending = Some(op),
            KeyCode::Char('h') => self.cursor = at.saturating_sub(1),
            KeyCode::Char('l') => self.cursor = (at + 1).min(last),
            KeyCode::Char('0') => self.cursor = 0,
            KeyCode::Char('$') => self.cursor = last,
            KeyCode::Char(w @ ('w' | 'W')) => self.cursor = next_start(&chars, at, w == 'W').min(last),
            KeyCode::Char(b @ ('b' | 'B')) => self.cursor = previous_start(&chars, at, b == 'B'),
            KeyCode::Char(e @ ('e' | 'E')) => self.cursor = next_end(&chars, at, e == 'E'),
            KeyCode::Char('x') => self.remove(at, (at + 1).min(len)),
            KeyCode::Char('D') => self.remove(at, len),
            KeyCode::Char('C') => {
                self.remove(at, len);
                self.mode = Mode::Insert;
            }
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('a') => {
                self.cursor = (at + 1).min(len);
                self.mode = Mode::Insert;
            }
            KeyCode::Char('I') => {
                self.cursor = chars.iter().position(|c| !c.is_whitespace()).unwrap_or(len);
                self.mode = Mode::Insert;
            }
            KeyCode::Char('A') => {
                self.cursor = len;
                self.mode = Mode::Insert;
            }
            _ => {}
        }
        if self.mode == Mode::Normal {
            self.cursor = self.cursor.min(self.text.chars().count().saturating_sub(1));
        }
    }

    /// Writes the priority `c`, `a` to `e`, at the start of a pending line in place of the one there, or drops it for a space;
    /// the cursor stays on its character.
    fn set_priority(&mut self, c: char) {
        let todo = Todo::from_line(&self.text);
        if todo.done {
            return;
        }
        let old = if todo.priority.is_some() { 4 } else { 0 };
        let new = if c == ' ' {
            String::new()
        } else {
            format!("({}) ", c.to_ascii_uppercase())
        };
        self.text.replace_range(..old, &new);
        self.cursor = if self.cursor < old {
            self.cursor.min(new.len())
        } else {
            self.cursor - old + new.len()
        };
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

/// Class of a character for word motions: blank, word (letter, digit, `_`) or punctuation; `big` makes every non-blank a word.
fn class(c: char, big: bool) -> u8 {
    if c.is_whitespace() {
        0
    } else if big || c.is_alphanumeric() || c == '_' {
        1
    } else {
        2
    }
}

/// Start of the word after the one at `at`, or the length of `chars` when there is none.
fn next_start(chars: &[char], at: usize, big: bool) -> usize {
    let class = |i: usize| class(chars[i], big);
    let mut i = at;
    if i < chars.len() && class(i) != 0 {
        let word = class(i);
        while i < chars.len() && class(i) == word {
            i += 1;
        }
    }
    while i < chars.len() && class(i) == 0 {
        i += 1;
    }
    i
}

/// Start of the word at `at`, or of the previous one when `at` already starts a word.
fn previous_start(chars: &[char], at: usize, big: bool) -> usize {
    let class = |i: usize| class(chars[i], big);
    let mut i = at.saturating_sub(1);
    while i > 0 && class(i) == 0 {
        i -= 1;
    }
    while i > 0 && class(i - 1) == class(i) {
        i -= 1;
    }
    i
}

/// End of the word at `at`, or of the next one when `at` already ends a word; `at` itself when no word follows.
fn next_end(chars: &[char], at: usize, big: bool) -> usize {
    let mut i = at + 1;
    while i < chars.len() && class(chars[i], big) == 0 {
        i += 1;
    }
    if i < chars.len() { run_end(chars, i, big) } else { at }
}

/// Last character of the run of characters of the same class as the one at `at`.
fn run_end(chars: &[char], at: usize, big: bool) -> usize {
    let mut i = at;
    while i + 1 < chars.len() && class(chars[i + 1], big) == class(chars[at], big) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests;
