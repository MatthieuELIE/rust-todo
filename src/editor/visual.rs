use super::find::opposite;
use super::words::motion;
use super::*;

impl Editor {
    /// Applies a key typed in visual mode: motions move the cursor, `y` copies the selection, `Esc` or `v` leave; any other key does nothing.
    pub(super) fn visual(&mut self, key: KeyEvent) {
        let chars: Vec<char> = self.text.chars().collect();
        let finding = self.finding.take();
        if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
            return;
        }
        match (finding, key.code) {
            (_, KeyCode::Esc) | (None, KeyCode::Char('v')) => self.mode = Mode::Normal,
            (Some(kind), KeyCode::Char(c)) => {
                self.find = Some((kind, c));
                self.seek(&chars, None, kind, c, false);
            }
            (Some(_), _) => {}
            (None, KeyCode::Char('y')) => {
                let start = self.anchor.min(self.cursor);
                let end = (self.anchor.max(self.cursor) + 1).min(chars.len());
                self.yank(&chars, start..end);
                self.mode = Mode::Normal;
            }
            (None, KeyCode::Char(kind @ ('f' | 'F' | 't' | 'T'))) => self.finding = Some(kind),
            (None, KeyCode::Char(r @ (';' | ','))) => {
                if let Some((kind, c)) = self.find {
                    self.seek(&chars, None, if r == ';' { kind } else { opposite(kind) }, c, true);
                }
            }
            (None, code) => self.cursor = motion(&chars, self.cursor, code).unwrap_or(self.cursor),
        }
        self.cursor = self.cursor.min(chars.len().saturating_sub(1));
    }

    /// The characters selected in visual mode, both ends included, `None` in any other mode.
    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        (self.mode == Mode::Visual).then(|| self.anchor.min(self.cursor)..self.anchor.max(self.cursor) + 1)
    }
}

#[cfg(test)]
mod tests;
