use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph, Wrap};

use crate::editor::{Editor, Mode};
use crate::todo::Todo;
use crate::tui::{App, Group, Target};

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

    let (entries, row) = panel(app);
    let highlight = if app.panel { Style::new().bg(Color::DarkGray) } else { Style::new() };
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
        let mut tasks = tasks.into_iter().map(|(number, todo)| line(number, todo));
        let (mut lines, mut rows) = (Vec::new(), Vec::new());
        for (group, count) in app.groups() {
            let folded = app.folded.contains(&group);
            if folded {
                rows.push(lines.len());
            }
            lines.push(header(group, count, folded, list_area.width.saturating_sub(2) as usize));
            for task in tasks.by_ref().take(count).filter(|_| !folded) {
                rows.push(lines.len());
                lines.push(task);
            }
        }
        for task in tasks {
            rows.push(lines.len());
            lines.push(task);
        }
        let list = List::new(lines)
            .highlight_symbol("▸ ")
            .highlight_style(Style::new().bg(Color::DarkGray))
            .scroll_padding(1);
        scroll.select(rows.get(app.cursor).copied());
        frame.render_stateful_widget(list, list_area, scroll);
    }

    let (mode, colour, keys) = if app.searching {
        ("SEARCH", Color::Yellow, "⏎ keep · esc clear")
    } else if let Some(popup) = &app.popup {
        match popup.editor.mode {
            Mode::Insert => ("INSERT", Color::Green, "esc normal · ⏎ save"),
            Mode::Normal => ("NORMAL", Color::Blue, "i/a insert · w/b/e word · x delete · ⏎ save · esc cancel"),
        }
    } else if app.panel {
        ("PANEL", Color::Magenta, "j/k filter · esc all · tab back")
    } else {
        (
            "LIST",
            Color::Blue,
            "⏎ edit · o add · x done · dd delete · p priority · u undo · zM fold · / search · ? help",
        )
    };
    let filters = if app.searching {
        format!("/{}▌", app.search)
    } else {
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        let done = app.show_done.then(|| "+done".to_string());
        [app.filter.clone(), search, done].into_iter().flatten().collect::<Vec<_>>().join("  ")
    };
    let keys = format!("{}{keys}", if filters.is_empty() { "" } else { "  " }).dim();
    let mut status = Line::from_iter([format!(" {mode} ").bold().black().bg(colour), format!(" {filters}").into()]);
    if app.message.is_none() && status.width() + keys.width() <= status_area.width as usize {
        status.push_span(keys);
    }
    frame.render_widget(Paragraph::new(status), status_area);
    if let Some(message) = &app.message {
        frame.render_widget(Paragraph::new(format!("{message} ")).right_aligned(), status_area);
    }

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

/// Rows of the filter panel and the one of the active filter: `all`, then the projects and the contexts, each section after a
/// blank row and a header, and left out when empty.
fn panel(app: &App) -> (Vec<Line<'static>>, usize) {
    let filters = app.filters();
    let active = app.filter_row(&filters);
    let (mut lines, mut row) = (Vec::new(), 0);
    for (i, (term, count)) in filters.iter().enumerate() {
        let sigil = term.chars().next();
        let (header, colour) = match sigil {
            Some('+') => (" PROJECTS", Color::Magenta),
            Some('@') => (" CONTEXTS", Color::Cyan),
            _ => ("", Color::Reset),
        };
        if i > 0 && filters[i - 1].0.chars().next() != sigil {
            lines.extend([Line::default(), Line::from(header.bold().fg(colour))]);
        }
        let marker = if i == active {
            row = lines.len();
            "▸ "
        } else {
            "  "
        };
        let name = Span::styled(format!("{term:<13.13}"), colour);
        lines.push(Line::from_iter([marker.into(), name, " ".into(), format!("{count:>3}").dim()]));
    }
    (lines, row)
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

/// Header of a group of the list: its count, its title, then a rule filling `width`, ended by ` ▸` when the group is folded.
fn header(group: Group, count: usize, folded: bool, width: usize) -> Line<'static> {
    let (title, style) = match group {
        Group::Priority(letter) => (format!("PRIORITY {letter}"), priority(letter)),
        Group::Unprioritised => ("NO PRIORITY".to_string(), Style::new().bold().dim()),
        Group::Done => ("DONE".to_string(), Style::new().bold().dim()),
    };
    let count = format!(" ({count})");
    let end = if folded { " ▸" } else { "" };
    let rule = "─".repeat(width.saturating_sub(count.len() + title.len() + 4 + end.chars().count()));
    Line::from_iter([count.dim(), "  ".into(), Span::styled(title, style), "  ".into(), rule.dim(), end.into()])
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
        render_in(app, 50, 5)
    }

    fn render_in(app: &App, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
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
                "▸ all             2│    1  Pay +rent",
                "                   │▸   2  Call +bank @phone",
                " PROJECTS          │",
                "  +bank           1│",
            ]
        );
        assert_eq!(buffer[(30, 1)].bg, Color::DarkGray);
        assert_eq!(buffer[(30, 0)].bg, Color::Reset);
    }

    #[test]
    fn the_active_filter_is_marked_in_the_panel_and_highlighted_once_the_panel_has_focus() {
        let mut app = app_of(&["Pay +rent", "Call +bank"]);
        app.filter = Some("+rent".to_string());

        let buffer = render_in(&app, 50, 8);
        assert!(rows(&buffer)[0].starts_with("  all "));
        assert!(rows(&buffer)[4].starts_with("▸ +rent "));
        assert_eq!(buffer[(5, 4)].bg, Color::Reset);

        app.panel = true;
        assert_eq!(render_in(&app, 50, 8)[(5, 4)].bg, Color::DarkGray);
    }

    #[test]
    fn the_panel_puts_projects_and_contexts_under_coloured_headers_and_drops_an_empty_section() {
        let buffer = render_in(&app_of(&["Pay +rent", "Call +bank @phone"]), 50, 9);
        let panel: Vec<String> = rows(&buffer)
            .iter()
            .map(|row| row.chars().take(19).collect::<String>().trim_end().to_string())
            .collect();

        assert_eq!(
            panel[..8],
            [
                "▸ all             2",
                "",
                " PROJECTS",
                "  +bank           1",
                "  +rent           1",
                "",
                " CONTEXTS",
                "  @phone          1"
            ]
        );
        assert_eq!((buffer[(1, 2)].fg, buffer[(1, 6)].fg), (Color::Magenta, Color::Cyan));
        assert!(buffer[(1, 2)].modifier.contains(Modifier::BOLD));
        assert_eq!(
            (buffer[(2, 3)].fg, buffer[(2, 7)].fg, buffer[(2, 0)].fg),
            (Color::Magenta, Color::Cyan, Color::Reset)
        );
        assert!(buffer[(18, 3)].modifier.contains(Modifier::DIM));

        let contexts_only = rows(&render_in(&app_of(&["Call @phone"]), 50, 9));
        assert_eq!(contexts_only[2].chars().take(9).collect::<String>(), " CONTEXTS");
        assert!(!contexts_only.iter().any(|row| row.contains("PROJECTS")));
    }

    #[test]
    fn group_headers_sit_above_their_tasks_with_the_cursor_on_the_same_task() {
        let mut app = app_of(&["c", "(B) b", "(A) a"]);
        app.cursor = 1;

        let buffer = render_in(&app, 50, 8);
        let list: Vec<String> = rows(&buffer).iter().map(|row| row.chars().skip(20).collect()).collect();

        assert_eq!(
            list[..6],
            [
                format!("   (1)  PRIORITY A  {}", "─".repeat(10)),
                "    3  (A) a".to_string(),
                format!("   (1)  PRIORITY B  {}", "─".repeat(10)),
                "▸   2  (B) b".to_string(),
                format!("   (1)  NO PRIORITY  {}", "─".repeat(9)),
                "    1  c".to_string(),
            ]
        );
        assert_eq!(buffer[(30, 3)].bg, Color::DarkGray);
        assert_eq!((buffer[(28, 0)].fg, buffer[(28, 2)].fg), (Color::Yellow, Color::Green));
        assert!(buffer[(28, 4)].modifier.contains(Modifier::DIM));
        assert!(buffer[(49, 0)].modifier.contains(Modifier::DIM));
    }

    #[test]
    fn a_folded_group_is_its_header_alone_ended_by_a_marker_and_the_cursor_can_stand_on_it() {
        let mut app = app_of(&["(A) a", "(A) b", "(B) c", "d"]);
        app.folded = vec![Group::Priority('A'), Group::Priority('B'), Group::Unprioritised];
        app.cursor = 1;

        let buffer = render_in(&app, 50, 8);
        let list: Vec<String> = rows(&buffer).iter().map(|row| row.chars().skip(20).collect()).collect();

        assert_eq!(
            list[..4],
            [
                format!("   (2)  PRIORITY A  {} ▸", "─".repeat(8)),
                format!("▸  (1)  PRIORITY B  {} ▸", "─".repeat(8)),
                format!("   (1)  NO PRIORITY  {} ▸", "─".repeat(7)),
                String::new(),
            ]
        );
        assert_eq!(buffer[(30, 1)].bg, Color::DarkGray);
    }

    #[test]
    fn the_status_bar_shows_the_mode_and_filters_left_and_the_message_right() {
        let mut app = app_of(&["Pay +rent"]);
        app.filter = Some("+rent".to_string());
        app.search = "pay".to_string();
        app.show_done = true;
        app.message = Some("reloaded".to_string());

        let buffer = render(&app);
        let status = rows(&buffer)[4].clone();
        assert!(status.starts_with(" LIST  +rent  /pay  +done "), "{status}");
        assert!(status.ends_with(" reloaded"), "{status}");
        assert_eq!((buffer[(1, 4)].bg, buffer[(1, 4)].fg), (Color::Blue, Color::Black));
        assert!(buffer[(1, 4)].modifier.contains(Modifier::BOLD));
        assert_eq!(buffer[(7, 4)].bg, Color::Reset);
    }

    #[test]
    fn the_mode_keys_follow_when_they_fit_whole_and_no_message_is_shown() {
        let mut app = app_of(&["Pay +rent"]);
        app.panel = true;

        let buffer = render(&app);
        assert_eq!(rows(&buffer)[4], " PANEL  j/k filter · esc all · tab back");
        assert_eq!(buffer[(1, 4)].bg, Color::Magenta);
        assert!(buffer[(9, 4)].modifier.contains(Modifier::DIM));

        app.message = Some("reloaded".to_string());
        assert!(!rows(&render(&app))[4].contains("j/k"));

        app.panel = false;
        app.message = None;
        assert_eq!(rows(&render(&app))[4], " LIST");
        assert!(rows(&render_in(&app, 100, 5))[4].ends_with("/ search · ? help"));

        app.searching = true;
        app.search = "ca".to_string();
        let buffer = render(&app);
        assert_eq!(rows(&buffer)[4], " SEARCH  /ca▌  ⏎ keep · esc clear");
        assert_eq!(buffer[(1, 4)].bg, Color::Yellow);
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
        assert_eq!(buffer[(1, 4)].bg, Color::Green);

        app.popup.as_mut().unwrap().editor.mode = Mode::Normal;
        let buffer = render(&app);
        assert!(self::rows(&buffer)[4].starts_with(" NORMAL"));
        assert_eq!(buffer[(1, 4)].bg, Color::Blue);
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

        assert_eq!(rows(&buffer)[0], "▸ all             1│▸   1  Call +bank @phone");
        assert_eq!(buffer[(32, 0)].fg, Color::Magenta);
        assert_eq!(buffer[(38, 0)].fg, Color::Cyan);
        assert_eq!(buffer[(27, 0)].fg, Color::Reset);
    }
}
