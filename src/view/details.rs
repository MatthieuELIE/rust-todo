use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use time::Date;

use super::style::{badge, card, due_style, icon, words};
use crate::theme::Theme;
use crate::todo::{DUE, Todo, parse_date};

const DETAILS_ICON: char = '\u{f05a}';

/// Height of the detail card under the list when its text fits one line: its borders and five rows; `Text` wraps and the card grows with it.
pub(super) const DETAILS_HEIGHT: u16 = 7;

/// Height of the detail card for `todo` in a list `width` wide: `DETAILS_HEIGHT` plus the rows its text wraps onto, at most `spare`.
pub(super) fn details_height(theme: &Theme, todo: Option<&Todo>, width: u16, spare: u16, today: Date) -> u16 {
    let text_width = width.saturating_sub(2 + label(theme, "Text").width() as u16);
    let text_rows = todo.map_or(1, |todo| {
        text_of(theme, todo, (!todo.done).then_some(today)).line_count(text_width) as u16
    });
    DETAILS_HEIGHT + text_rows.saturating_sub(1).min(spare)
}

/// Draws the detail zone: a ` DETAILS ` rule, then the priority or completion, text, dates, projects, contexts and key:values of `todo`.
/// The text wraps under its value over the rows `area` leaves it, its last row ended by `…` when it is cut; the other rows are cut by `…`.
pub(super) fn draw_details(frame: &mut Frame, theme: &Theme, icons: bool, todo: Option<&Todo>, area: Rect, today: Date) {
    let block = card(theme, false).title(format!(" {}DETAILS ", icon(icons, DETAILS_ICON)).fg(theme.secondary).bold());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(todo) = todo else {
        return;
    };
    let [first_row, text_row, dates_row, tags_row, key_values_row] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let [created_area, due_area] = Layout::horizontal([Constraint::Fill(1); 2]).areas(dates_row);
    let [projects_area, contexts_area] = Layout::horizontal([Constraint::Fill(1); 2]).areas(tags_row);
    let today = (!todo.done).then_some(today);
    let tags = |sigil: char, names: Vec<&str>| {
        words(
            theme,
            &names.iter().map(|name| format!("{sigil}{name}")).collect::<Vec<_>>().join(" "),
            today,
        )
    };
    let date = |date: Option<Date>| date.map(|date| date.to_string().into()).into_iter().collect();
    let key_values: Vec<&str> = todo.description.split_whitespace().filter(|word| Todo::is_key_value(word)).collect();
    let due = key_values.iter().find_map(|word| word.strip_prefix(DUE)).map(|value| {
        let span = Span::styled(value.to_string(), due_style(theme, value, today).unwrap_or_default());
        let late = parse_date(value).zip(today).is_some_and(|(due, today)| due < today);
        if late { span.bold() } else { span }
    });
    let mut others = Line::from(" ");
    for (i, word) in key_values.iter().filter(|word| !word.starts_with(DUE)).enumerate() {
        let (key, value) = word.split_once(':').unwrap_or_default();
        others.extend([
            if i > 0 { "  " } else { "" }.into(),
            format!("{key}:").fg(theme.tertiary),
            value.to_string().into(),
        ]);
    }

    if todo.done {
        detail(frame, theme, "Done", date(todo.completed), first_row);
    } else {
        let priority = todo.priority.map(|letter| badge(theme, letter));
        detail(frame, theme, "Priority", priority.into_iter().collect(), first_row);
    }
    draw_text(frame, theme, text_of(theme, todo, today), text_row);
    detail(frame, theme, "Created", date(todo.created), created_area);
    detail(frame, theme, "Due", due.into_iter().collect(), due_area);
    detail(frame, theme, "Projects", tags('+', todo.projects()), projects_area);
    detail(frame, theme, "Contexts", tags('@', todo.contexts()), contexts_area);
    draw_cut(frame, others, key_values_row);
}

/// Draws a row of the detail zone, its `label` dimmed then its `value`.
fn detail(frame: &mut Frame, theme: &Theme, label: &str, value: Vec<Span<'static>>, area: Rect) {
    let mut line = Line::from(self::label(theme, label));
    line.extend(value);
    draw_cut(frame, line, area);
}

/// The dimmed label of a row of the detail zone, padded to the column the values start at.
fn label(theme: &Theme, label: &str) -> Span<'static> {
    format!(" {label:<10}").fg(theme.label)
}

/// The words of the text of `todo`, wrapped like the popup's field.
fn text_of(theme: &Theme, todo: &Todo, today: Option<Date>) -> Paragraph<'static> {
    Paragraph::new(Line::from(words(theme, todo.text(), today))).wrap(Wrap { trim: false })
}

/// Draws the `Text` row: its label, then `text` beside it, the lines after the first aligned under the value; `…` ends the last row when `text` is cut.
fn draw_text(frame: &mut Frame, theme: &Theme, text: Paragraph, area: Rect) {
    let label = label(theme, "Text");
    let [label_area, value_area] = Layout::horizontal([Constraint::Length(label.width() as u16), Constraint::Fill(1)]).areas(area);
    frame.render_widget(label, label_area);
    let cut = text.line_count(value_area.width) > value_area.height as usize;
    frame.render_widget(text, value_area);
    if cut {
        frame.buffer_mut()[(value_area.right() - 1, value_area.bottom() - 1)].set_symbol("…");
    }
}

/// Draws `line` in `area`, ended by `…` when it overflows.
fn draw_cut(frame: &mut Frame, line: Line, area: Rect) {
    let overflows = line.width() > area.width as usize;
    frame.render_widget(line, area);
    if overflows {
        frame.buffer_mut()[(area.right() - 1, area.y)].set_symbol("…");
    }
}

#[cfg(test)]
mod tests;
