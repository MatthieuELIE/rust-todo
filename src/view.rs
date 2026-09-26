use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph, Wrap};

use crate::editor::{Editor, Mode};
use crate::todo::Todo;
use crate::tui::{App, Group, Target};

/// Keys of the list shown by `?`, one per line.
const HELP_LIST: &str = "\
j k  ↓ ↑     move
gg  G        top, bottom
Enter        edit
o            add
x            done, not done
dd           delete
p a…e        priority
p Space      no priority
u  Ctrl-r    undo, redo
zM  zR       fold, unfold all
za           fold, unfold group
/            search
H            show, hide done
Tab          panel
Esc          drop filter and search
?            these keys
q            quit";

/// Keys of the popup and the panel shown by `?`, one per line.
const HELP_EDIT: &str = "\
in the popup
Enter        save
Esc          normal mode
Ctrl-w       erase a word
Ctrl-u       erase to the start

in normal mode
h l  0 $     move
w b e        word
W B E        blank-separated word
x  D  C      delete, to end, change
dw cw dW cW  delete, change a word
i a  I A     insert
Esc          cancel

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
        let height = HELP_LIST.lines().count().max(HELP_EDIT.lines().count()) as u16 + 2;
        let area = main_area.centered(Constraint::Length(76), Constraint::Length(height));
        let block = Block::bordered().title(" keys ");
        let [list, edit] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(2).areas(block.inner(area));
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(Paragraph::new(HELP_LIST), list);
        frame.render_widget(Paragraph::new(HELP_EDIT), edit);
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
mod tests;
