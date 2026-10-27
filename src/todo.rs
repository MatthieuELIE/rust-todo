use time::Date;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

/// A date format for parsing dates in the todo.txt format (YYYY-MM-DD).
const DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year]-[month]-[day]");

/// Key of the `key:value` naming what a task waits for, as in `wait:figma`.
pub const WAIT: &str = "wait:";

/// Key of the `key:value` holding a task's due date, as in `due:2026-10-15`.
pub const DUE: &str = "due:";

/// Why a task made only of non-text words is refused when typed.
pub const NO_TEXT: &str = "a task needs more than +projects, @contexts and key: words";

/// Whether `c` is a control character or a bidi or zero-width formatting character, none of which a task may hold.
pub fn is_control(c: char) -> bool {
    c.is_control() || matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// A todo item, with optional priority and dates.
#[derive(Clone)]
pub struct Todo {
    /// Task text after the marker, priority and dates, with `+project`, `@context` and `key:value` kept verbatim.
    pub description: String,
    /// Whether the line starts with the `x` marker.
    pub done: bool,
    /// Priority letter, `A` to `E`, dropped once the task is done.
    pub priority: Option<char>,
    /// Creation date.
    pub created: Option<Date>,
    /// Completion date, only on a done task.
    pub completed: Option<Date>,
}

impl Todo {
    /// Convert a todo item to a line of text in the todo.txt format.
    pub fn to_line(&self) -> String {
        let mut parts = Vec::new();
        if self.done {
            parts.push("x".to_string());
            parts.extend(self.completed.map(|date| date.to_string()));
        } else if let Some(priority) = self.priority {
            parts.push(format!("({priority})"));
        }
        parts.extend(self.created.map(|date| date.to_string()));
        parts.push(self.description.clone());
        parts.join(" ")
    }

    /// Parse a todo.txt line, leaving anything unrecognised in the description.
    pub fn from_line(line: &str) -> Self {
        let done = line.starts_with("x ");
        let mut rest = if done { &line[2..] } else { line };

        let completed = if done { strip_date(&mut rest) } else { None };
        let priority = strip_priority(&mut rest);
        let created = strip_date(&mut rest);

        Todo {
            description: rest.to_string(),
            done,
            priority,
            created,
            completed,
        }
    }

    /// Parse a line typed by the user, its trailing spaces dropped, refusing a control character.
    pub fn from_input(text: &str) -> Result<Todo, String> {
        if text.contains(is_control) {
            return Err("a task is one line, without control characters".to_string());
        }
        let mut todo = Todo::from_line(text);
        todo.description.truncate(todo.description.trim_end().len());
        Ok(todo)
    }

    /// Refuse a task whose description has no text word.
    pub fn check_text(&self) -> Result<(), String> {
        if self.has_text() { Ok(()) } else { Err(NO_TEXT.to_string()) }
    }

    /// Whether the description has a word that is not a `+project`, `@context`, `key:` word, priority or date.
    fn has_text(&self) -> bool {
        self.description.split_whitespace().any(|word| {
            let named = |sigil| word.strip_prefix(sigil).is_some_and(|name| !name.is_empty());
            let key = word
                .split_once(':')
                .is_some_and(|(key, _)| key.starts_with(char::is_alphabetic) && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
            let priority = matches!(word.as_bytes(), [b'(', p, b')'] if Todo::is_valid_priority(*p as char));
            !(named('+') || named('@') || key || priority || parse_date(word).is_some())
        })
    }

    /// Build a pending task from user input, stamped with `today` unless it carries a creation date.
    pub fn new_from_input(text: &str, today: Date) -> Result<Todo, String> {
        let mut todo = Todo::from_input(text)?;
        todo.check_text()?;
        if todo.done {
            return Err("cannot add a task that is already done".to_string());
        }
        todo.created.get_or_insert(today);
        Ok(todo)
    }

    /// Mark the task done on `today`, keeping an existing completion date and dropping the priority and the `wait:` key:values.
    pub fn complete(&mut self, today: Date) {
        self.done = true;
        self.completed.get_or_insert(today);
        self.priority = None;
        self.drop_waits();
    }

    /// Remove each `wait:` read as a wait, with the space before it, or after it when it opens the description.
    fn drop_waits(&mut self) {
        let text = self.description.clone();
        let mut spans = Vec::new();
        let mut start = None;
        for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
            match (start, c.is_whitespace()) {
                (None, false) => start = Some(i),
                (Some(s), true) => {
                    let value = text[s..i].strip_prefix(WAIT).unwrap_or("//");
                    if !value.is_empty() && !value.starts_with("//") {
                        spans.push((s, i));
                    }
                    start = None;
                }
                _ => {}
            }
        }
        for (s, e) in spans.into_iter().rev() {
            let current = &self.description;
            let range = match current[..s].chars().next_back() {
                Some(c) => s - c.len_utf8()..e,
                None => s..e + current[e..].chars().next().map_or(0, char::len_utf8),
            };
            self.description.replace_range(range, "");
        }
    }

    /// Mark the task pending again, clearing its completion date and its priority.
    /// Refused with the reason, the task untouched, when it is not done, its line would still read as done or it has no text word.
    pub fn reopen(&mut self) -> Result<(), &'static str> {
        if !self.done {
            return Err("is not done");
        }
        let pending = Todo {
            done: false,
            completed: None,
            priority: None,
            ..self.clone()
        };
        if Todo::from_line(&pending.to_line()).done {
            return Err("cannot be reopened: its text starts with x");
        }
        if !pending.has_text() {
            return Err("cannot be reopened: it is only +projects, @contexts and key: words");
        }
        *self = pending;
        Ok(())
    }

    /// `+project` names in the description, without the `+`, once each in order of appearance.
    pub fn projects(&self) -> Vec<&str> {
        self.words_after("+")
    }

    /// `@context` names in the description, without the `@`, once each in order of appearance.
    pub fn contexts(&self) -> Vec<&str> {
        self.words_after("@")
    }

    /// Words of the description starting with `prefix`, without it, once each in order of appearance.
    fn words_after(&self, prefix: &str) -> Vec<&str> {
        let mut names = Vec::new();
        for name in self.description.split_whitespace().filter_map(|word| word.strip_prefix(prefix)) {
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    /// The description without its trailing run of `key:value` words; a `+project` or `@context` ending it stays.
    pub fn text(&self) -> &str {
        let mut text = self.description.as_str();
        loop {
            let trimmed = text.trim_end();
            let (rest, last) = trimmed.rsplit_once(char::is_whitespace).unwrap_or(("", trimmed));
            if !Todo::is_key_value(last) {
                return trimmed;
            }
            text = rest;
        }
    }

    /// Whether `c` is a priority letter, `A` to `E`.
    fn is_valid_priority(c: char) -> bool {
        ('A'..='E').contains(&c)
    }

    /// Whether the task is pending with a `due:` date on `today` or before.
    pub fn is_due(&self, today: Date) -> bool {
        !self.done && self.words_after(DUE).into_iter().filter_map(parse_date).any(|due| due <= today)
    }

    /// Whether the description holds a `wait:` key:value, what the task waits for.
    pub fn is_waiting(&self) -> bool {
        !self.waits().is_empty()
    }

    /// Values of the `wait:` key:values in the description, once each in order of appearance.
    pub fn waits(&self) -> Vec<&str> {
        self.words_after(WAIT).into_iter().filter(|value| !value.starts_with("//")).collect()
    }

    /// Whether `word` is a `key:value`: a key of letters, digits, `-` and `_` starting with a letter, a value neither empty nor `//`.
    pub fn is_key_value(word: &str) -> bool {
        let Some((key, value)) = word.split_once(':') else {
            return false;
        };
        key.starts_with(char::is_alphabetic)
            && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !value.is_empty()
            && !value.starts_with("//")
    }
}

/// Take a leading `(A) ` off `rest` and return its letter.
fn strip_priority(rest: &mut &str) -> Option<char> {
    let mut chars = rest.chars();
    let priority = match (chars.next(), chars.next(), chars.next(), chars.next()) {
        (Some('('), Some(p), Some(')'), Some(' ')) if Todo::is_valid_priority(p) => p,
        _ => return None,
    };
    *rest = &rest[4..];
    Some(priority)
}

/// Take a leading `YYYY-MM-DD ` off `rest` and return the date.
fn strip_date(rest: &mut &str) -> Option<Date> {
    let (token, remainder) = rest.split_once(' ')?;
    let date = parse_date(token)?;
    *rest = remainder;
    Some(date)
}

/// Parse a `YYYY-MM-DD` date.
pub fn parse_date(text: &str) -> Option<Date> {
    Date::parse(text, DATE_FORMAT).ok()
}

#[cfg(test)]
mod tests;
