use std::ops::Range;

use super::*;

/// Where the motion key `code` takes a cursor at `at`, the length of `chars` when it runs past the text; `None` for any other key.
pub(super) fn motion(chars: &[char], at: usize, code: KeyCode) -> Option<usize> {
    Some(match code {
        KeyCode::Char('h') => at.saturating_sub(1),
        KeyCode::Char('l') => at + 1,
        KeyCode::Char('0') => 0,
        KeyCode::Char('$') => chars.len(),
        KeyCode::Char(w @ ('w' | 'W')) => next_start(chars, at, w == 'W'),
        KeyCode::Char(b @ ('b' | 'B')) => previous_start(chars, at, b == 'B'),
        KeyCode::Char(e @ ('e' | 'E')) => next_end(chars, at, e == 'E'),
        _ => return None,
    })
}

/// The word under `at`, a run of one class, blanks being one; `around` adds the blanks after it, those before it when none follow.
pub(super) fn object(chars: &[char], at: usize, big: bool, around: bool) -> Range<usize> {
    if at >= chars.len() {
        return at..at;
    }
    let mut start = at;
    while start > 0 && class(chars[start - 1], big) == class(chars[at], big) {
        start -= 1;
    }
    let mut end = run_end(chars, at, big) + 1;
    if around && end < chars.len() && (chars[end].is_whitespace() || chars[at].is_whitespace()) {
        end = run_end(chars, end, big) + 1;
    } else if around {
        while start > 0 && chars[start - 1].is_whitespace() {
            start -= 1;
        }
    }
    start..end
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
pub(super) fn run_end(chars: &[char], at: usize, big: bool) -> usize {
    let mut i = at;
    while i + 1 < chars.len() && class(chars[i + 1], big) == class(chars[at], big) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests;
