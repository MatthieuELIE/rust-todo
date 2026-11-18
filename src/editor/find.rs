use super::*;

/// The other direction of a find key.
fn opposite(kind: char) -> char {
    match kind {
        'f' => 'F',
        'F' => 'f',
        't' => 'T',
        _ => 't',
    }
}

/// The search `;` or `,` (`key`) repeats after `last`: the same one, or its opposite direction.
pub(super) fn repeated(key: char, (kind, target): (char, char)) -> (char, char) {
    if key == ';' {
        return (kind, target);
    }

    (opposite(kind), target)
}

/// Where `f`, `F`, `t` or `T` takes a cursor at `at` to `target`; a repeat of `t` or `T` skips the character next to the cursor.
fn find(chars: &[char], at: usize, kind: char, target: char, repeat: bool) -> Option<usize> {
    let skip = usize::from(repeat && matches!(kind, 't' | 'T'));
    let forward = || (at + 1 + skip..chars.len()).find(|&i| chars[i] == target);
    let backward = || (0..at.saturating_sub(skip)).rev().find(|&i| chars[i] == target);

    match kind {
        'f' => forward(),
        't' => forward().map(|i| i - 1),
        'F' => backward(),
        _ => backward().map(|i| i + 1),
    }
}

impl Editor {
    /// Moves the cursor to the character `target` found as `kind` says, or after `d`, `c` or `y` (`pending`) acts up to it; nothing happens when it is absent.
    pub(super) fn seek(&mut self, chars: &[char], pending: Option<char>, kind: char, target: char, repeat: bool) -> Option<String> {
        let at = self.cursor;
        let to = find(chars, at, kind, target, repeat)?;

        let Some(op) = pending else {
            self.cursor = to;
            return None;
        };

        let range = match kind {
            'f' | 't' => at..to + 1,
            _ => to..at,
        };

        self.operate(chars, op, range)
    }
}

#[cfg(test)]
mod tests;
