use std::cmp::Ordering;

use ratatui::Frame;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListState, Padding, Paragraph, Wrap};
use time::Date;

use super::style::{floating, highlight, priority, tag_style, words};
use crate::editor::Editor;
use crate::theme::Theme;
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, Focus, Popup, Target, WAITING};

/// Draws the popup centred in `bounds`, titled by the task edited or the term an add gets, with the tag's completions or the date picker.
pub(super) fn draw_popup(frame: &mut Frame, app: &App, popup: &Popup, bounds: Rect, today: Date) {
    let theme = &app.theme;
    let title = match (&popup.target, app.filter.as_deref()) {
        (Target::Edit(number), _) => Line::from(format!(" EDIT {number} ")),
        (Target::Add, Some(term)) if term != WAITING => {
            Line::from_iter([" ADD (".into(), Span::styled(term.to_string(), tag_style(theme, term)), ") ".into()])
        }
        (Target::Add, _) => Line::from(" ADD "),
    };
    let field = Paragraph::new(field(theme, &popup.editor, today))
        .wrap(Wrap { trim: false })
        .block(floating(theme, popup.picker.is_none()).title(title).padding(Padding::right(1)));
    let width = bounds.width * 4 / 5;
    let height = field.line_count(width.saturating_sub(3)) as u16;
    let area = bounds.centered(Constraint::Length(width), Constraint::Length(height));
    frame.render_widget(Clear, area);
    frame.render_widget(field, area);
    // The cursor's cell is marked reversed then read back from the buffer, since the paragraph does not tell where it wrapped
    // the text; the marks are taken off and the terminal's own cursor stands there. A blank at the end of a wrapped row is not
    // drawn: the cursor then stands after the last cell drawn before it, marked hidden, in the column kept free on the right.
    let marked = |frame: &mut Frame, mark| area.positions().filter(|&cell| frame.buffer_mut()[cell].modifier.contains(mark)).last();
    let after = marked(frame, Modifier::HIDDEN).map(|cell| {
        let width = Span::raw(frame.buffer_mut()[cell].symbol()).width() as u16;
        Position::new(cell.x + width, cell.y)
    });
    let cursor = marked(frame, Modifier::REVERSED).or(after);
    for cell in area.positions() {
        frame.buffer_mut()[cell].modifier.remove(Modifier::REVERSED | Modifier::HIDDEN);
    }
    if let Some(cursor) = cursor
        && popup.picker.is_none()
        && !matches!(app.focus, Focus::Help(_))
    {
        frame.set_cursor_position(cursor);
    }
    if let (Some(tag), Some(cursor)) = (popup.editor.tag(), cursor) {
        let (names, selected) = app.completions();
        let start = Position::new(cursor.x.saturating_sub(tag.chars().count() as u16), cursor.y);
        draw_completions(frame, theme, &names, selected, start, bounds);
    }
    if let (Some(date), Some(cursor)) = (popup.picker, cursor) {
        draw_picker(
            frame,
            theme,
            date,
            today,
            Position::new(cursor.x.saturating_sub(DUE.len() as u16), cursor.y),
            bounds,
        );
    }
}

/// Draws the date picker under the word at `word`: `date`'s month from Monday, `date` in the accent, `today` yellow, past days greyed.
fn draw_picker(frame: &mut Frame, theme: &Theme, date: Date, today: Date, word: Position, bounds: Rect) {
    let first = date.replace_day(1).expect("every month has a first day");
    let offset = first.weekday().number_days_from_monday();
    let mut lines = vec![Line::from("Mo Tu We Th Fr Sa Su".fg(theme.tertiary))];
    let mut week = Line::from("   ".repeat(offset.into()));
    for day in 1..=date.month().length(date.year()) {
        let style = match first.replace_day(day).map(|shown| shown.cmp(&today)) {
            _ if day == date.day() => Style::new().bold().fg(theme.accent).bg(theme.selected),
            Ok(Ordering::Less) => Style::new().fg(theme.tertiary),
            Ok(Ordering::Equal) => Style::new().fg(theme.due_today),
            _ => Style::new(),
        };
        week.push_span(Span::styled(format!("{day:>2}"), style));
        if (offset + day).is_multiple_of(7) {
            lines.push(std::mem::take(&mut week));
        } else {
            week.push_span(" ");
        }
    }
    if !week.spans.is_empty() {
        lines.push(week);
    }
    let title = format!(" {} {} ", date.month().to_string().to_uppercase(), date.year());
    let area = drop_down(word, 22, lines.len() as u16 + 2, bounds);
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(floating(theme, true).title(title)), area);
}

/// Draws the completion `names`, five rows at most, under the tag at `tag` or above it without room, `selected` highlighted.
fn draw_completions(frame: &mut Frame, theme: &Theme, names: &[(String, usize)], selected: usize, tag: Position, bounds: Rect) {
    if names.is_empty() {
        return;
    }
    let width = names.iter().map(|(name, _)| name.chars().count()).max().unwrap_or_default();
    let rows = names.iter().map(|(name, count)| {
        Line::from_iter([
            Span::styled(format!("{name:<width$}"), tag_style(theme, name)),
            format!(" {count:>3}").fg(theme.tertiary),
        ])
    });
    let area = drop_down(tag, width as u16 + 6, names.len().min(5) as u16 + 2, bounds);
    let list = List::new(rows)
        .highlight_style(highlight(theme, true).bold())
        .block(floating(theme, true));
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(selected)));
}

/// Area of `width` × `height` dropping down under the word at `word`, or above it without room, within `bounds`.
fn drop_down(word: Position, width: u16, height: u16, bounds: Rect) -> Rect {
    let y = if word.y + 1 + height <= bounds.bottom() {
        word.y + 1
    } else {
        word.y.saturating_sub(height)
    };
    Rect::new(word.x.saturating_sub(1), y, width, height).intersection(bounds)
}

/// The popup's text styled as a task is, one span per character, the one under the cursor, or a space past the end, reversed, those before it hidden.
fn field(theme: &Theme, editor: &Editor, today: Date) -> Line<'static> {
    let mut chars: Vec<Span<'static>> = typed(theme, &editor.text, today)
        .into_iter()
        .flat_map(|span| span.content.chars().map(|c| Span::styled(c.to_string(), span.style)).collect::<Vec<_>>())
        .collect();
    if editor.cursor >= chars.len() {
        chars.push(" ".into());
    }
    chars[editor.cursor].style = chars[editor.cursor].style.reversed();
    for before in &mut chars[..editor.cursor] {
        before.style = before.style.hidden();
    }
    Line::from(chars)
}

/// Words of a line being typed, the priority and dates before the description styled as in the list, then its words.
fn typed(theme: &Theme, text: &str, today: Date) -> Vec<Span<'static>> {
    let description = Todo::from_line(text).description;
    let head = &text[..text.len() - description.len()];
    let mut spans = Vec::new();
    for word in head.split_inclusive(' ') {
        let letter = word.trim_end().strip_prefix('(').and_then(|rest| rest.strip_suffix(')'));
        let style = match letter.and_then(|letter| letter.parse::<char>().ok()) {
            Some(letter) => Style::new().bold().fg(priority(theme, letter)),
            None if parse_date(word.trim_end()).is_some() => Style::new().fg(theme.tertiary),
            None => Style::new(),
        };
        spans.push(Span::styled(word.to_string(), style));
    }
    spans.extend(words(theme, &description, Some(today)));
    spans
}

#[cfg(test)]
mod tests;
