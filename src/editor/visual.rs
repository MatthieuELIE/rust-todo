use super::find::repeated;
use super::words::motion;
use super::*;

impl Editor {
    /// Applies a key typed in visual mode: motions move the cursor, `y` copies the selection, `Esc` or `v` leave; any other key does nothing.
    pub(super) fn visual(&mut self, key: KeyEvent) -> Option<String> {
        let chars: Vec<char> = self.text.chars().collect();
        let finding = self.finding.take();
        if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
            return None;
        }

        let mut copy = None;
        match (finding, key.code) {
            (_, KeyCode::Esc) | (None, KeyCode::Char('v')) => self.mode = Mode::Normal,
            (Some(kind), KeyCode::Char(c)) => {
                self.find = Some((kind, c));
                self.seek(&chars, None, kind, c, false);
            }
            (Some(_), _) => {}
            (None, KeyCode::Char('y')) => {
                if let Some(range) = self.selection() {
                    copy = Some(self.yank(&chars, range.start..range.end.min(chars.len())));
                }

                self.mode = Mode::Normal;
            }
            (None, KeyCode::Char(kind @ ('f' | 'F' | 't' | 'T'))) => self.finding = Some(kind),
            (None, KeyCode::Char(r @ (';' | ','))) => {
                if let Some(last) = self.find {
                    let (kind, c) = repeated(r, last);
                    self.seek(&chars, None, kind, c, true);
                }
            }
            (None, code) => self.cursor = motion(&chars, self.cursor, code).unwrap_or(self.cursor),
        }
        self.cursor = self.cursor.min(chars.len().saturating_sub(1));

        copy
    }

    /// The characters selected in visual mode, both ends included, `None` in any other mode.
    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        (self.mode == Mode::Visual).then(|| self.anchor.min(self.cursor)..self.anchor.max(self.cursor) + 1)
    }
}

#[cfg(test)]
mod tests;
