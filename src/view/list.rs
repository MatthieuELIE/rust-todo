use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListState, Paragraph};
use time::Date;

use super::style::{arrow, badge, card, highlight, priority, words};
use crate::theme::Theme;
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, Focus, Group, Shown};

/// Draws the tasks on screen under their group headers, or says there is none; `scroll` keeps the offset between frames.
pub(super) fn draw_list(frame: &mut Frame, app: &App, area: Rect, scroll: &mut ListState, today: Date) {
    let theme = &app.theme;
    let focused = matches!(app.focus, Focus::List | Focus::Search);
    let block = card(theme, focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let area = inner;
    let shown = app.shown();
    if shown.is_empty() {
        frame.render_widget(
            Paragraph::new(if !app.search.is_empty() || app.filter.is_some() {
                "no matching task"
            } else if app.in_history {
                "nothing done"
            } else {
                "nothing to do"
            })
            .fg(theme.tertiary),
            area,
        );
    } else {
        let (mut lines, mut rows) = (Vec::new(), Vec::new());
        for shown in shown {
            if matches!(shown, Shown::Header(..)) && !lines.is_empty() {
                lines.push(Line::default());
            }
            if !matches!(shown, Shown::Header(_, _, false)) {
                rows.push(lines.len());
            }
            lines.push(match shown {
                Shown::Header(group, count, folded) => header(theme, group, count, folded, area.width.saturating_sub(2) as usize),
                Shown::Task(number, todo) if app.in_history => Line::from(self::number(number) + &todo.to_line()).fg(theme.secondary),
                Shown::Task(number, todo) => line(theme, number, todo, today),
            });
        }
        let list = List::new(lines)
            .highlight_symbol(arrow(theme, focused))
            .highlight_style(highlight(theme, focused))
            .scroll_padding(1);
        scroll.select(rows.get(app.cursor).copied());
        frame.render_stateful_widget(list, area, scroll);
    }
}

/// The number column in front of a listed task.
pub fn number(number: usize) -> String {
    format!("{number:>3}  ")
}

/// A listed task: its number then its todo.txt line, the priority as a badge, a late due date in bold, due dates against `today`, struck through when done.
pub fn line(theme: &Theme, number: usize, todo: &Todo, today: Date) -> Line<'static> {
    let number = self::number(number);
    if todo.done {
        return Line::from_iter([number.into(), todo.to_line().fg(theme.tertiary).crossed_out()]);
    }
    let mut spans = vec![Span::raw(number)];
    spans.extend(todo.priority.map(|letter| badge(theme, letter)));
    if todo.priority.is_some() {
        spans.push(" ".into());
    }
    if let Some(created) = todo.created {
        spans.extend([created.to_string().fg(theme.tertiary), " ".into()]);
    }
    spans.extend(words(theme, &todo.description, Some(today)).into_iter().map(|span| {
        let late = span.content.strip_prefix(DUE).and_then(parse_date).is_some_and(|due| due < today);
        if late { span.bold() } else { span }
    }));
    Line::from(spans)
}

/// Header of a group of the list: its count, its title, then a rule filling `width`, ended by ` ▸` when the group is folded.
fn header(theme: &Theme, group: Group, count: usize, folded: bool, width: usize) -> Line<'static> {
    let (title, style) = match group {
        Group::Priority(letter) => (format!("PRIORITY {letter}"), Style::new().bold().fg(priority(theme, letter))),
        Group::Unprioritised => ("NO PRIORITY".to_string(), Style::new().bold().fg(theme.secondary)),
    };
    let count = format!(" ({count})");
    let end = if folded { " ▸" } else { "" };
    let rule = "─".repeat(width.saturating_sub(count.len() + title.len() + 4 + end.chars().count()));
    Line::from_iter([
        count.fg(theme.tertiary),
        "  ".into(),
        Span::styled(title, style),
        "  ".into(),
        rule.fg(theme.structure),
        end.into(),
    ])
}

#[cfg(test)]
mod tests;
