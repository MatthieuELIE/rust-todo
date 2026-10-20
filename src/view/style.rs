use std::cmp::Ordering;

use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType};
use time::Date;

use crate::theme::Theme;
use crate::todo::{DUE, Todo, parse_date};

/// The glyph and the space after it when `icons` is on, else nothing.
pub(super) fn icon(icons: bool, glyph: char) -> String {
    if icons { format!("{glyph} ") } else { String::new() }
}

/// Words of `text`, a `due:` date styled against `today` when given, any other word by `tag_style`, with the spaces between them kept.
pub(super) fn words(theme: &Theme, text: &str, today: Option<Date>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            spans.push(" ".into());
        }
        let due = word.strip_prefix(DUE).and_then(|value| due_style(theme, value, today));
        spans.push(Span::styled(word.to_string(), due.unwrap_or_else(|| tag_style(theme, word))));
    }
    spans
}

/// Style of a due date: red once past `today`, yellow on the day, none when later, not a date, or with no `today`.
pub(super) fn due_style(theme: &Theme, value: &str, today: Option<Date>) -> Option<Style> {
    let (due, today) = (parse_date(value)?, today?);
    match due.cmp(&today) {
        Ordering::Less => Some(Style::new().fg(theme.alert)),
        Ordering::Equal => Some(Style::new().fg(theme.due_today)),
        Ordering::Greater => None,
    }
}

/// Style of a word of a task: a `+project` mauve, an `@context` teal, a `key:value` greyed, anything else plain.
pub(super) fn tag_style(theme: &Theme, word: &str) -> Style {
    match word.chars().next() {
        Some('+') if word.len() > 1 => Style::new().fg(theme.project),
        Some('@') if word.len() > 1 => Style::new().fg(theme.context),
        _ if Todo::is_key_value(word) => Style::new().fg(theme.tertiary),
        _ => Style::new(),
    }
}

/// A zone's card: a rounded border, in the accent when the zone gets the keys.
pub(super) fn card(theme: &Theme, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { theme.accent } else { theme.structure }))
}

/// A floating window's frame: rounded on a raised background, its border in the accent when it gets the keys, its titles bold.
pub(super) fn floating(theme: &Theme, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { theme.accent } else { theme.separator }))
        .title_style(Style::new().fg(theme.primary).bold())
        .style(Style::new().bg(theme.raised))
}

/// The cursor's mark, in the accent when its zone gets the keys.
pub(super) fn arrow(theme: &Theme, focused: bool) -> Span<'static> {
    "→ ".fg(if focused { theme.accent } else { theme.tertiary })
}

/// Style of the row under the cursor: highlighted only when its zone gets the keys.
pub(super) fn highlight(theme: &Theme, focused: bool) -> Style {
    if focused { Style::new().bg(theme.selected) } else { Style::new() }
}

/// Colour of a priority, `A` to `E`.
pub(super) fn priority(theme: &Theme, letter: char) -> Color {
    theme.priorities[(letter as usize).saturating_sub('A' as usize).min(theme.priorities.len() - 1)]
}

/// A priority as a badge: its letter bold in its colour between two spaces, on a dark background.
pub(super) fn badge(theme: &Theme, letter: char) -> Span<'static> {
    Span::styled(format!(" {letter} "), Style::new().bold().fg(priority(theme, letter)).bg(theme.badge))
}

#[cfg(test)]
mod tests;
