use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph};

use crate::todo::Todo;
use crate::tui::{App, Prompt};

/// Draws the filter panel and the task list above a one-line status bar; `scroll` keeps the list's offset from one frame to the
/// next.
pub fn draw(frame: &mut Frame, app: &App, scroll: &mut ListState) {
    let [main_area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let [panel_area, list_area] = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(main_area);

    let filters = app.filters();
    let row = app.filter_row(&filters);
    let entries = filters.iter().map(|(term, count)| format!(" {term:<14.14} {count:>3}"));
    let highlight = if app.panel {
        Style::new().bg(Color::DarkGray)
    } else {
        Style::new().bold()
    };
    let panel = List::new(entries).highlight_style(highlight).block(Block::new().borders(Borders::RIGHT));
    frame.render_stateful_widget(panel, panel_area, &mut ListState::default().with_selected(Some(row)));

    let tasks = app.tasks();
    if tasks.is_empty() {
        frame.render_widget(
            Paragraph::new(if app.search.is_empty() && app.filter.is_none() {
                "nothing to do"
            } else {
                "no matching task"
            })
            .dim(),
            list_area,
        );
    } else {
        let lines = tasks.into_iter().map(|(number, todo)| Line::styled(line(number, todo), style(todo)));
        let list = List::new(lines).highlight_symbol("▸ ").highlight_style(Style::new().bg(Color::DarkGray));
        scroll.select(Some(app.cursor));
        frame.render_stateful_widget(list, list_area, scroll);
    }

    let status = match app.prompt {
        Some(Prompt::Add) => match &app.filter {
            Some(term) => Line::from(format!(" add ({term}): {}▌", app.input)),
            None => Line::from(format!(" add: {}▌", app.input)),
        },
        Some(Prompt::Search) => Line::from(format!(" /{}▌", app.search)),
        None => {
            let mode = if app.panel { " PANEL" } else { " LIST" };
            let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
            let done = app.show_done.then(|| "+done".to_string());
            let filters: String = [app.filter.clone(), search, done]
                .into_iter()
                .flatten()
                .map(|f| format!("  {f}"))
                .collect();
            Line::from_iter([mode.bold(), filters.into()])
        }
    };
    frame.render_widget(Paragraph::new(status), status_area);
    if let Some(message) = &app.message {
        frame.render_widget(Paragraph::new(format!("{message} ")).right_aligned(), status_area);
    }
}

/// A listed task: its number, then its todo.txt line.
pub fn line(number: usize, todo: &Todo) -> String {
    format!("{number:>3}  {}", todo.to_line())
}

/// Bold when the task has a priority, tinted for A to C as `todo.sh` does, dim when it is done.
pub fn style(todo: &Todo) -> Style {
    let bold = Style::new().bold();
    match (todo.done, todo.priority) {
        (true, _) => Style::new().dim(),
        (false, Some('A')) => bold.yellow(),
        (false, Some('B')) => bold.green(),
        (false, Some('C')) => bold.blue(),
        (false, Some(_)) => bold,
        (false, None) => Style::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_tasks_are_dimmed_and_prioritised_ones_bold_with_a_to_c_tinted() {
        let style_of = |line: &str| style(&Todo::from_line(line));

        assert_eq!(style_of("Buy milk"), Style::new());
        assert_eq!(style_of("(A) Call the bank"), Style::new().bold().yellow());
        assert_eq!(style_of("(B) Pay rent"), Style::new().bold().green());
        assert_eq!(style_of("(C) Book dentist"), Style::new().bold().blue());
        assert_eq!(style_of("(D) Read book"), Style::new().bold());
        assert_eq!(style_of("x 2026-09-03 Buy milk"), Style::new().dim());
    }
}
