use super::*;

use super::find::opposite;
use super::words::{motion, object, run_end};
use crate::todo::Todo;

impl Editor {
    /// Applies a key typed in normal mode, a motion moving the cursor, then keeps the cursor on a character if it stays there.
    pub(super) fn normal(&mut self, code: KeyCode) {
        let chars: Vec<char> = self.text.chars().collect();
        let (len, at) = (chars.len(), self.cursor);
        let (pending, scope, finding) = (self.pending.take(), self.scope.take(), self.finding.take());
        let free = scope.is_none() && matches!(pending, None | Some('d' | 'c'));
        let search = match (finding, code) {
            (Some(kind), KeyCode::Char(c)) => {
                self.find = Some((kind, c));
                Some((kind, c, false))
            }
            (Some(_), _) => return,
            (None, KeyCode::Char(kind @ ('f' | 'F' | 't' | 'T'))) if free => {
                (self.pending, self.finding) = (pending, Some(kind));
                return;
            }
            (None, KeyCode::Char(r @ (';' | ','))) if free => self.find.map(|(kind, c)| (if r == ';' { kind } else { opposite(kind) }, c, true)),
            _ => None,
        };
        if let Some((kind, c, repeat)) = search {
            self.seek(&chars, pending, kind, c, repeat);
            return;
        }
        match code {
            KeyCode::Char(key @ ('i' | 'a')) if matches!(pending, Some('d' | 'c')) && scope.is_none() => {
                self.pending = pending;
                self.scope = Some(key);
            }
            _ if matches!(pending, Some('d' | 'c')) => {
                let range = match code {
                    KeyCode::Char(w @ ('w' | 'W')) if scope.is_some() => Some(object(&chars, at, w == 'W', scope == Some('a'))),
                    _ if scope.is_some() => None,
                    KeyCode::Char(c) if pending == Some(c) => Some(0..len),
                    KeyCode::Char(w @ ('w' | 'W')) if pending == Some('c') && chars.get(at).is_some_and(|c| !c.is_whitespace()) => {
                        Some(at..run_end(&chars, at, w == 'W') + 1)
                    }
                    KeyCode::Char('e' | 'E') => motion(&chars, at, code).map(|end| at..(end + 1).min(len)),
                    code => motion(&chars, at, code).map(|to| at.min(to)..at.max(to)),
                };
                if let Some(range) = range {
                    self.remove(range.start, range.end);
                    if pending == Some('c') {
                        self.mode = Mode::Insert;
                    }
                }
            }
            KeyCode::Char(c @ ('a'..='e' | ' ')) if pending == Some('p') => self.set_priority(c),
            KeyCode::Char(op @ ('d' | 'c' | 'p')) => self.pending = Some(op),
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
            code => self.cursor = motion(&chars, at, code).unwrap_or(at),
        }
        if self.mode == Mode::Normal {
            self.cursor = self.cursor.min(self.text.chars().count().saturating_sub(1));
        }
    }

    /// Writes priority `c`, `a` to `e` or a space to drop it, at the start of a pending line, the cursor kept on its character.
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
}

#[cfg(test)]
mod tests;
