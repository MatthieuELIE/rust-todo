use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph, Wrap};

use crate::editor::{Editor, Mode};
use crate::todo::Todo;
use crate::tui::{App, Target};

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
        let lines = tasks.into_iter().map(|(number, todo)| line(number, todo));
        let list = List::new(lines).highlight_symbol("▸ ").highlight_style(Style::new().bg(Color::DarkGray));
        scroll.select(Some(app.cursor));
        frame.render_stateful_widget(list, list_area, scroll);
    }

    let status = if app.searching {
        Line::from(format!(" /{}▌", app.search))
    } else {
        let mode = if let Some(popup) = &app.popup {
            match popup.editor.mode {
                Mode::Insert => " INSERT",
                Mode::Normal => " NORMAL",
            }
        } else if app.panel {
            " PANEL"
        } else {
            " LIST"
        };
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        let done = app.show_done.then(|| "+done".to_string());
        let filters: String = [app.filter.clone(), search, done]
            .into_iter()
            .flatten()
            .map(|f| format!("  {f}"))
            .collect();
        Line::from_iter([mode.bold(), filters.into()])
    };
    frame.render_widget(Paragraph::new(status), status_area);
    let right = app.message.as_deref().map_or("? help ".dim(), |message| format!("{message} ").into());
    frame.render_widget(Paragraph::new(right).right_aligned(), status_area);

    if app.help {
        let area = main_area.centered(Constraint::Length(38), Constraint::Length(HELP.lines().count() as u16 + 2));
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(HELP).block(Block::bordered().title(" keys ")), area);
    }

    if let Some(popup) = &app.popup {
        let title = match (&popup.target, &app.filter) {
            (Target::Edit(number), _) => format!(" edit {number} "),
            (Target::Add, Some(term)) => format!(" add ({term}) "),
            (Target::Add, None) => " add ".to_string(),
        };
        let field = Paragraph::new(field(&popup.editor))
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(title));
        let width = main_area.width * 4 / 5;
        let height = field.line_count(width.saturating_sub(2)) as u16;
        let area = main_area.centered(Constraint::Length(width), Constraint::Length(height));
        frame.render_widget(Clear, area);
        frame.render_widget(field, area);
    }
}

/// The popup's text with the character under the cursor, or a space past the end, in reverse video.
fn field(editor: &Editor) -> Line<'static> {
    let mut chars = editor.text.chars();
    let before: String = chars.by_ref().take(editor.cursor).collect();
    let under = chars.next().map_or(" ".to_string(), String::from);
    Line::from_iter([before.into(), under.reversed(), chars.collect::<String>().into()])
}

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, message: &str) {
    let text = vec![Line::from(format!(" {message}")).red(), Line::from(" press any key to quit").dim()];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
}

/// A listed task, its number then its todo.txt line: a done task all dimmed, a pending one with its priority styled, its
/// creation date dimmed, `+projects` magenta and `@contexts` cyan.
pub fn line(number: usize, todo: &Todo) -> Line<'static> {
    let number = format!("{number:>3}  ");
    if todo.done {
        return Line::from(format!("{number}{}", todo.to_line()).dim());
    }
    let mut spans = vec![Span::raw(number)];
    if let Some(letter) = todo.priority {
        spans.extend([Span::styled(format!("({letter})"), priority(letter)), " ".into()]);
    }
    if let Some(created) = todo.created {
        spans.extend([created.to_string().dim(), " ".into()]);
    }
    for (i, word) in todo.description.split(' ').enumerate() {
        if i > 0 {
            spans.push(" ".into());
        }
        let style = match word.chars().next() {
            Some('+') if word.len() > 1 => Style::new().magenta(),
            Some('@') if word.len() > 1 => Style::new().cyan(),
            _ => Style::new(),
        };
        spans.push(Span::styled(word.to_string(), style));
    }
    Line::from(spans)
}

/// A priority bold, tinted yellow, green and blue for A to C as `todo.sh` does.
fn priority(letter: char) -> Style {
    let bold = Style::new().bold();
    match letter {
        'A' => bold.yellow(),
        'B' => bold.green(),
        'C' => bold.blue(),
        _ => bold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use crate::tui::Popup;
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
    fn the_popup_wraps_its_text_under_its_title_with_the_cursor_cell_reversed_while_the_status_bar_says_insert() {
        let mut app = app_of(&["Pay +rent"]);
        app.filter = Some("+rent".to_string());
        let mut editor = Editor::default();
        editor.text = "Call the bank about the loan and ask for a quote".to_string();
        editor.cursor = 5;
        app.popup = Some(Popup { editor, target: Target::Add });

        let buffer = render(&app);
        let rows = rows(&buffer);

        assert!(rows[0].contains("┌ add (+rent) ───"), "{}", rows[0]);
        assert!(rows[1].contains("│Call the bank about the loan and ask  │"), "{}", rows[1]);
        assert!(rows[2].contains("│for a quote                           │"), "{}", rows[2]);
        assert!(rows[3].contains("└───"), "{}", rows[3]);
        assert_eq!(buffer[(11, 1)].symbol(), "t");
        assert!(buffer[(11, 1)].modifier.contains(Modifier::REVERSED));
        assert!(!buffer[(10, 1)].modifier.contains(Modifier::REVERSED));
        assert!(rows[4].starts_with(" INSERT"), "{}", rows[4]);

        app.popup.as_mut().unwrap().editor.mode = Mode::Normal;
        assert!(self::rows(&render(&app))[4].starts_with(" NORMAL"));
    }

    fn style_of(line_text: &str, token: &str) -> Style {
        let line = line(1, &Todo::from_line(line_text));
        line.iter().find(|span| span.content == token).expect(token).style
    }

    #[test]
    fn a_pending_line_keeps_its_text_and_styles_priority_date_projects_and_contexts() {
        let text = "(A) 2026-09-26 Call  +bank @phone due:2026-10-01 a+b + @";

        assert_eq!(line(12, &Todo::from_line(text)).to_string(), format!(" 12  {text}"));
        assert_eq!(style_of(text, "(A)"), Style::new().bold().yellow());
        assert_eq!(style_of(text, "2026-09-26"), Style::new().dim());
        assert_eq!(style_of(text, "+bank"), Style::new().magenta());
        assert_eq!(style_of(text, "@phone"), Style::new().cyan());
        for plain in ["Call", "due:2026-10-01", "a+b", "+", "@"] {
            assert_eq!(style_of(text, plain), Style::new(), "{plain}");
        }
        assert_eq!(style_of("(B) x", "(B)"), Style::new().bold().green());
        assert_eq!(style_of("(C) x", "(C)"), Style::new().bold().blue());
        assert_eq!(style_of("(D) x", "(D)"), Style::new().bold());
    }

    #[test]
    fn a_done_line_is_dimmed_all_through() {
        let line = line(3, &Todo::from_line("x 2026-09-26 2026-09-20 Call +bank @phone"));

        assert_eq!(line.to_string(), "  3  x 2026-09-26 2026-09-20 Call +bank @phone");
        assert!(line.iter().all(|span| span.style == Style::new().dim()));
    }

    #[test]
    fn projects_and_contexts_are_coloured_on_screen() {
        let buffer = render(&app_of(&["Call +bank @phone"]));

        assert_eq!(rows(&buffer)[0], " all              1│▸   1  Call +bank @phone");
        assert_eq!(buffer[(32, 0)].fg, Color::Magenta);
        assert_eq!(buffer[(38, 0)].fg, Color::Cyan);
        assert_eq!(buffer[(27, 0)].fg, Color::Reset);
    }
}
