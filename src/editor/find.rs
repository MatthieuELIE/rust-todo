use super::*;

/// The other direction of a find key.
pub(super) fn opposite(kind: char) -> char {
    match kind {
        'f' => 'F',
        'F' => 'f',
        't' => 'T',
        _ => 't',
    }
}

/// Where `f`, `F`, `t` or `T` takes a cursor at `at` to `target`; a repeat of `t` or `T` skips the character next to the cursor.
fn find(chars: &[char], at: usize, kind: char, target: char, repeat: bool) -> Option<usize> {
    let skip = usize::from(repeat && matches!(kind, 't' | 'T'));
    if matches!(kind, 'f' | 't') {
        let i = (at + 1 + skip..chars.len()).find(|&i| chars[i] == target)?;
        Some(if kind == 't' { i - 1 } else { i })
    } else {
        let i = (0..at.saturating_sub(skip)).rev().find(|&i| chars[i] == target)?;
        Some(if kind == 'T' { i + 1 } else { i })
    }
}

impl Editor {
    /// Moves the cursor to the character `target` found as `kind` says, or after `d` or `c` (`pending`) acts up to it; nothing happens when it is absent.
    pub(super) fn seek(&mut self, chars: &[char], pending: Option<char>, kind: char, target: char, repeat: bool) {
        let at = self.cursor;
        let Some(to) = find(chars, at, kind, target, repeat) else {
            return;
        };
        if pending.is_none() {
            self.cursor = to;
            return;
        }
        let range = if matches!(kind, 'f' | 't') { at..to + 1 } else { to..at };
        self.remove(range.start, range.end);
        if pending == Some('c') {
            self.mode = Mode::Insert;
        } else {
            self.cursor = self.cursor.min(self.text.chars().count().saturating_sub(1));
        }
    }
}

#[cfg(test)]
mod tests;
