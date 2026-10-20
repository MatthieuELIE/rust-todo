use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListState};
use time::Date;

use super::style::{arrow, card, highlight, icon, tag_style};
use crate::tui::{App, DUE_NOW, Focus, WAITING};

const PROJECTS_ICON: char = '\u{f07b}';
const CONTEXTS_ICON: char = '\u{f1fa}';
const ALL_TASKS_ICON: char = '\u{f0ca}';
const DUE_ICON: char = '\u{f073}';
const WAITING_ICON: char = '\u{f252}';

/// Width of the filter panel's card, its borders included.
pub(super) const PANEL_WIDTH: u16 = 22;

/// Draws the filter panel's card in `area`, the active filter's row highlighted.
pub(super) fn draw_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let (entries, row) = panel(app);
    let focused = matches!(app.focus, Focus::Panel);
    let panel = List::new(entries).highlight_style(highlight(theme, focused)).block(card(theme, focused));
    frame.render_stateful_widget(panel, area, &mut ListState::default().with_selected(row));
}

/// Rows of the filter panel, entries under their section headers, and the row of the active filter when it is among them.
fn panel(app: &App) -> (Vec<Line<'static>>, Option<usize>) {
    let theme = &app.theme;
    let focused = matches!(app.focus, Focus::Panel);
    let filters = app.filters();
    let active = app.filter_row(&filters);
    let (mut lines, mut row) = (Vec::new(), None);
    for (i, (term, count)) in filters.iter().enumerate() {
        let sigil = term.chars().next();
        let (header, header_icon) = match sigil {
            Some('+') => ("PROJECTS", PROJECTS_ICON),
            Some('@') => ("CONTEXTS", CONTEXTS_ICON),
            _ => ("", ' '),
        };
        let style = if term == DUE_NOW {
            let late = app
                .today
                .and_then(Date::previous_day)
                .is_some_and(|eve| app.todos.iter().any(|todo| todo.is_due(eve)));
            Style::new().bold().fg(if late { theme.alert } else { theme.due_today })
        } else if header.is_empty() {
            Style::new().bold()
        } else {
            tag_style(theme, term)
        };
        if !header.is_empty() && filters[i - 1].0.chars().next() != sigil {
            let indent = if app.icons { "  " } else { " " };
            lines.extend([
                Line::default(),
                Line::from(format!("{indent}{}{header}", icon(app.icons, header_icon)).fg(theme.secondary).bold()),
            ]);
        }
        let marker = if Some(i) == active {
            row = Some(lines.len());
            arrow(theme, focused)
        } else {
            "  ".into()
        };
        let style = if app.in_history { style.fg(theme.secondary) } else { style };
        let entry_icon = match term.as_str() {
            "All tasks" => icon(app.icons, ALL_TASKS_ICON),
            DUE_NOW => icon(app.icons, DUE_ICON),
            WAITING => icon(app.icons, WAITING_ICON),
            _ => String::new(),
        };
        let name = Span::styled(format!("{:<13.13}", format!("{entry_icon}{term}")), style);
        lines.push(Line::from_iter([marker, name, " ".into(), format!("{count:>3}").fg(theme.tertiary)]));
    }
    (lines, row)
}

#[cfg(test)]
mod tests;
