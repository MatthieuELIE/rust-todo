use super::*;

/// The kind of a character search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Search {
    /// `f`: onto the next character.
    Find,
    /// `F`: onto the previous character.
    FindBack,
    /// `t`: just before the next character.
    Till,
    /// `T`: just after the previous character.
    TillBack,
}

impl Search {
    /// The search typed with `code`, if there is one.
    pub(super) const fn from_code(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::Char('f') => Some(Self::Find),
            KeyCode::Char('F') => Some(Self::FindBack),
            KeyCode::Char('t') => Some(Self::Till),
            KeyCode::Char('T') => Some(Self::TillBack),
            _ => None,
        }
    }

    /// The key that types the search.
    pub(super) const fn key(self) -> char {
        match self {
            Self::Find => 'f',
            Self::FindBack => 'F',
            Self::Till => 't',
            Self::TillBack => 'T',
        }
    }

    /// The same search in the other direction.
    const fn opposite(self) -> Self {
        match self {
            Self::Find => Self::FindBack,
            Self::FindBack => Self::Find,
            Self::Till => Self::TillBack,
            Self::TillBack => Self::Till,
        }
    }
}

/// The search `;` or `,` (`key`) repeats after `last`: the same one, or its opposite direction.
pub(super) fn repeated(key: char, (kind, target): (Search, char)) -> (Search, char) {
    if key == ';' {
        return (kind, target);
    }

    (kind.opposite(), target)
}

/// Where `f`, `F`, `t` or `T` takes a cursor at `at` to `target`; a repeat of `t` or `T` skips the character next to the cursor.
fn find(chars: &[char], at: usize, kind: Search, target: char, repeat: bool) -> Option<usize> {
    let skip = usize::from(repeat && matches!(kind, Search::Till | Search::TillBack));
    let forward = || (at + 1 + skip..chars.len()).find(|&i| chars[i] == target);
    let backward = || (0..at.saturating_sub(skip)).rev().find(|&i| chars[i] == target);

    match kind {
        Search::Find => forward(),
        Search::Till => forward().map(|i| i - 1),
        Search::FindBack => backward(),
        Search::TillBack => backward().map(|i| i + 1),
    }
}

impl Editor {
    /// Moves the cursor to the character `target` found as `kind` says, or after `d`, `c` or `y` (`pending`) acts up to it; nothing happens when it is absent.
    pub(super) fn seek(&mut self, chars: &[char], pending: Option<Operator>, kind: Search, target: char, repeat: bool) -> Option<String> {
        let at = self.cursor;
        let to = find(chars, at, kind, target, repeat)?;

        let Some(op) = pending else {
            self.cursor = to;
            return None;
        };

        let range = match kind {
            Search::Find | Search::Till => at..to + 1,
            Search::FindBack | Search::TillBack => to..at,
        };

        self.operate(chars, op, range)
    }
}

#[cfg(test)]
mod tests;
