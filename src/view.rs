use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph};

use crate::todo::Todo;
use crate::tui::{App, Prompt};

/// Keys shown by `?`, one per line.
const HELP: &str = "\
j k  ↓ ↑     move
gg  G        top, bottom
x            done, not done
o            add
dd           delete
/            search
H            show, hide done
Tab          panel
Esc          drop filter and search
q            quit

in the panel
j k          pick a filter
Esc          all
Tab  Enter   back to the list";

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
    let right = app.message.as_deref().map_or("? help ".dim(), |message| format!("{message} ").into());
    frame.render_widget(Paragraph::new(right).right_aligned(), status_area);

    if app.help {
        let area = main_area.centered(Constraint::Length(38), Constraint::Length(HELP.lines().count() as u16 + 2));
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(HELP).block(Block::bordered().title(" keys ")), area);
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
    use crate::store::Store;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Modifier;

    fn app_of(lines: &[&str]) -> App {
        App::new(Store::new(lines.iter().map(|l| Todo::from_line(l)).collect()))
    }

    fn render(app: &App) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(50, 5)).unwrap();
        terminal.draw(|frame| draw(frame, app, &mut ListState::default())).unwrap();
        terminal.backend().buffer().clone()
    }

    fn rows(buffer: &Buffer) -> Vec<String> {
        let row = |y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        };
        (0..buffer.area.height).map(row).collect()
    }

    #[test]
    fn the_list_sits_right_of_the_panel_with_the_cursor_row_marked_and_highlighted() {
        let mut app = app_of(&["Pay +rent", "Call +bank @phone", "x 2026-09-20 Old +rent"]);
        app.cursor = 1;

        let buffer = render(&app);

        assert_eq!(
            rows(&buffer)[..4],
            [
                " all              2│    1  Pay +rent",
                " +bank            1│▸   2  Call +bank @phone",
                " +rent            1│",
                " @phone           1│",
            ]
        );
        assert_eq!(buffer[(30, 1)].bg, Color::DarkGray);
        assert_eq!(buffer[(30, 0)].bg, Color::Reset);
    }

    #[test]
    fn the_active_filter_is_bold_in_the_panel_and_highlighted_once_the_panel_has_focus() {
        let mut app = app_of(&["Pay +rent", "Call +bank"]);
        app.filter = Some("+rent".to_string());

        let buffer = render(&app);
        assert!(buffer[(1, 2)].modifier.contains(Modifier::BOLD));
        assert!(!buffer[(1, 0)].modifier.contains(Modifier::BOLD));

        app.panel = true;
        assert_eq!(render(&app)[(1, 2)].bg, Color::DarkGray);
    }

    #[test]
    fn the_status_bar_shows_the_mode_and_filters_left_and_the_message_or_help_right() {
        let mut app = app_of(&["Pay +rent"]);
        app.filter = Some("+rent".to_string());
        app.search = "pay".to_string();
        app.show_done = true;
        app.message = Some("reloaded".to_string());

        let status = rows(&render(&app))[4].clone();
        assert!(status.starts_with(" LIST  +rent  /pay  +done "), "{status}");
        assert!(status.ends_with(" reloaded"), "{status}");

        app.panel = true;
        app.message = None;
        let status = rows(&render(&app))[4].clone();
        assert!(status.starts_with(" PANEL  +rent"), "{status}");
        assert!(status.ends_with(" ? help"), "{status}");
    }

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
