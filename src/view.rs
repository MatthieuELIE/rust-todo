use std::cmp::Ordering;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListState, Padding, Paragraph, Wrap};
use time::Date;

use crate::editor::{Editor, Mode};
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, Focus, Group, Popup, Target, WAITING};

/// Catppuccin Mocha's mantle, herdr's popup colour, painted on every unset cell so the list and the popup's frame read as one.
const BACKGROUND: Color = Color::Rgb(0x18, 0x18, 0x25);

/// Height of the detail zone under the list: its rule and five rows.
const DETAILS_HEIGHT: u16 = 6;

/// Fewest rows the list keeps; on a screen too low for them and the detail zone, the zone is hidden.
const MIN_LIST_ROWS: u16 = 5;

/// Keys of the list shown by `?` under the mode's name, one per line, a key's alternatives separated by `/`.
const HELP_LIST: &str = "\
LIST
j/k/↓/↑      move
gg/G         top, bottom
Enter        edit
o            add
x            done, not done
dd           delete
p a…e        priority
p Space      no priority
u/Ctrl-r     undo, redo
zM/zR        fold, unfold all
za           fold, unfold group
/            search
H            show, hide done
Tab          panel
Esc          drop filter and search
?            these keys
q            quit";

/// Keys of the popup's two modes and of the panel shown by `?`, each under its mode's name, one per line.
const HELP_EDIT: &str = "\
INSERT
Enter        save
Esc          normal mode
Ctrl-w       erase a word
Ctrl-u       erase to the start
Tab          complete + or @ word
↓/↑/Ctrl-n/p pick a completion

NORMAL
h/l/0/$      move
w/b/e        word
W/B/E        blank-separated word
x/D/C        delete, to end, change
dw/cw/dW/cW  delete, change a word
i/a/I/A      insert
p a…e        priority
p Space      no priority
Esc          cancel

PANEL
j/k          pick a filter
Esc          all tasks
Tab/Enter    back to the list";

/// Draws the filter panel and the task list above the status bar, due dates against `today`; `scroll` keeps the list's offset between frames.
pub fn draw(frame: &mut Frame, app: &App, scroll: &mut ListState, today: Date) {
    let [main_area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let [panel_area, list_area] = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(main_area);

    let (entries, row) = panel(app);
    let highlight = if matches!(app.focus, Focus::Panel) {
        Style::new().bg(Color::DarkGray)
    } else {
        Style::new()
    };
    let panel = List::new(entries).highlight_style(highlight).block(Block::new().borders(Borders::RIGHT));
    frame.render_stateful_widget(panel, panel_area, &mut ListState::default().with_selected(row));

    if list_area.height >= MIN_LIST_ROWS + DETAILS_HEIGHT {
        let [list_area, details_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(DETAILS_HEIGHT)]).areas(list_area);
        draw_list(frame, app, list_area, scroll, today);
        draw_details(frame, app.selected_task(), details_area, today);
    } else {
        draw_list(frame, app, list_area, scroll, today);
    }
    draw_status(frame, app, status_area);
    match &app.focus {
        Focus::Help => draw_help(frame, main_area),
        Focus::Popup(popup) => draw_popup(frame, app, popup, main_area),
        _ => {}
    }
    for cell in frame.buffer_mut().content.iter_mut().filter(|cell| cell.bg == Color::Reset) {
        cell.bg = BACKGROUND;
    }
}

/// Draws the tasks on screen under their group headers, or says there is none; `scroll` keeps the offset between frames.
fn draw_list(frame: &mut Frame, app: &App, area: Rect, scroll: &mut ListState, today: Date) {
    let tasks = app.tasks();
    if tasks.is_empty() {
        frame.render_widget(
            Paragraph::new(if app.search.is_empty() && app.filter.is_none() {
                "nothing to do"
            } else {
                "no matching task"
            })
            .dim(),
            area,
        );
    } else {
        let mut tasks = tasks.into_iter().map(|(number, todo)| line(number, todo, today));
        let (mut lines, mut rows) = (Vec::new(), Vec::new());
        for (group, count) in app.groups() {
            let folded = app.folded.contains(&group);
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            if folded {
                rows.push(lines.len());
            }
            lines.push(header(group, count, folded, area.width.saturating_sub(2) as usize));
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
        frame.render_stateful_widget(list, area, scroll);
    }
}

/// Draws the detail zone: a ` DETAILS ` rule, then the priority or completion, text, dates, projects, contexts and key:values of `todo`.
fn draw_details(frame: &mut Frame, todo: Option<&Todo>, area: Rect, today: Date) {
    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().dim())
        .title(" DETAILS ".dim());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(todo) = todo else {
        return;
    };
    let [first_row, text_row, dates_row, tags_row, key_values_row] = Layout::vertical([Constraint::Length(1); 5]).areas(inner);
    let [created_area, due_area] = Layout::horizontal([Constraint::Fill(1); 2]).areas(dates_row);
    let [projects_area, contexts_area] = Layout::horizontal([Constraint::Fill(1); 2]).areas(tags_row);
    let today = (!todo.done).then_some(today);
    let tags = |sigil: char, names: Vec<&str>| words(&names.iter().map(|name| format!("{sigil}{name}")).collect::<Vec<_>>().join(" "), today);
    let date = |date: Option<Date>| date.map(|date| date.to_string().into()).into_iter().collect();
    let key_values: Vec<&str> = todo.description.split_whitespace().filter(|word| Todo::is_key_value(word)).collect();
    let due = key_values
        .iter()
        .find_map(|word| word.strip_prefix(DUE))
        .map(|value| Span::styled(value.to_string(), due_style(value, today).unwrap_or_default()));
    let mut others = Line::from(" ");
    for (i, word) in key_values.iter().filter(|word| !word.starts_with(DUE)).enumerate() {
        let (key, value) = word.split_once(':').unwrap_or_default();
        others.extend([if i > 0 { "  " } else { "" }.into(), format!("{key}:").dim(), value.to_string().into()]);
    }

    if todo.done {
        detail(frame, "Done", date(todo.completed), first_row);
    } else {
        let priority = todo.priority.map(|letter| Span::styled(letter.to_string(), priority(letter)));
        detail(frame, "Priority", priority.into_iter().collect(), first_row);
    }
    detail(frame, "Text", words(todo.text(), today), text_row);
    detail(frame, "Created", date(todo.created), created_area);
    detail(frame, "Due", due.into_iter().collect(), due_area);
    detail(frame, "Projects", tags('+', todo.projects()), projects_area);
    detail(frame, "Contexts", tags('@', todo.contexts()), contexts_area);
    draw_cut(frame, others, key_values_row);
    if todo.done {
        frame.buffer_mut().set_style(inner, Style::new().dim());
    }
}

/// Draws a row of the detail zone, its `label` dimmed then its `value`.
fn detail(frame: &mut Frame, label: &str, value: Vec<Span<'static>>, area: Rect) {
    let mut line = Line::from(format!(" {label:<10}").dim());
    line.extend(value);
    draw_cut(frame, line, area);
}

/// Draws `line` in `area`, ended by `…` when it overflows.
fn draw_cut(frame: &mut Frame, line: Line, area: Rect) {
    let overflows = line.width() > area.width as usize;
    frame.render_widget(line, area);
    if overflows {
        frame.buffer_mut()[(area.right() - 1, area.y)].set_symbol("…");
    }
}

/// Draws the status bar: mode block, active filters, then the mode's keys when they fit, or the message on the right.
fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let (mode, keys) = match &app.focus {
        Focus::Search => ("SEARCH", "⏎ keep · esc clear"),
        Focus::Popup(popup) => match popup.editor.mode {
            Mode::Insert if app.completions().0.is_empty() => ("INSERT", "esc normal · ⏎ save"),
            Mode::Insert => ("INSERT", "esc normal · ⏎ save · tab complete"),
            Mode::Normal => ("NORMAL", "i insert · p priority · ⏎ save · esc cancel"),
        },
        Focus::Panel => ("PANEL", "j/k filter · esc all tasks · tab back"),
        Focus::List | Focus::Help => ("LIST", "⏎ edit · o add · x done · p priority · ? help"),
    };
    let filters = if matches!(app.focus, Focus::Search) {
        format!("/{}▌", app.search)
    } else {
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        let done = app.show_done.then(|| "+done".to_string());
        [app.filter.clone(), search, done].into_iter().flatten().collect::<Vec<_>>().join("  ")
    };
    let mut status = Line::from_iter([mode_block(mode), format!(" {filters}").into()]);
    let mut hints = Line::from(if filters.is_empty() { "" } else { "  " });
    for (i, hint) in keys.split(" · ").enumerate() {
        let (key, action) = hint.split_once(' ').unwrap_or_default();
        hints.extend([if i > 0 { " · " } else { "" }.dim(), key.bold(), format!(" {action}").dim()]);
    }
    if app.message.is_none() && status.width() + hints.width() <= area.width as usize {
        status.extend(hints);
    }
    frame.render_widget(Paragraph::new(status), area);
    if let Some(message) = &app.message {
        frame.render_widget(Paragraph::new(format!("{message} ")).right_aligned(), area);
    }
}

/// Draws the key help in two columns in a rounded box, centred in `area`.
fn draw_help(frame: &mut Frame, area: Rect) {
    let height = HELP_LIST.lines().count().max(HELP_EDIT.lines().count()) as u16 + 4;
    let area = area.centered(Constraint::Length(82), Constraint::Length(height));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" HELP ".bold())
        .title_bottom(Line::from(" any key closes ".dim()).right_aligned())
        .padding(Padding::new(3, 3, 1, 1));
    let [list, edit] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(2).areas(block.inner(area));
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(help_lines(HELP_LIST)), list);
    frame.render_widget(Paragraph::new(help_lines(HELP_EDIT)), edit);
}

/// Lines of a help column: a mode name as its status bar block, keys bold, the `/` between alternatives and actions dimmed.
fn help_lines(text: &'static str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| {
            if matches!(line, "LIST" | "INSERT" | "NORMAL" | "PANEL") {
                return Line::from(mode_block(line));
            }
            let (key, action) = line.split_at(line.char_indices().nth(13).map_or(line.len(), |(i, _)| i));
            let alternatives: Vec<&str> = if key.trim_end() == "/" { vec![key] } else { key.split('/').collect() };
            let mut spans = Vec::new();
            for (i, alternative) in alternatives.into_iter().enumerate() {
                if i > 0 {
                    spans.push("/".dim());
                }
                spans.push(alternative.bold());
            }
            spans.push(action.dim());
            Line::from(spans)
        })
        .collect()
}

/// A mode's name in bold black on the mode's colour, as the status bar and the help show it.
fn mode_block(mode: &str) -> Span<'static> {
    let colour = match mode {
        "INSERT" => Color::Green,
        "PANEL" => Color::Magenta,
        "SEARCH" => Color::Yellow,
        _ => Color::Blue,
    };
    format!(" {mode} ").bold().black().bg(colour)
}

/// Draws the popup centred in `bounds`, titled by the task edited or the term an add gets, with the tag's completions.
fn draw_popup(frame: &mut Frame, app: &App, popup: &Popup, bounds: Rect) {
    let title = match (&popup.target, app.filter.as_deref()) {
        (Target::Edit(number), _) => format!(" EDIT {number} "),
        (Target::Add, Some(term)) if term != WAITING => format!(" ADD ({term}) "),
        (Target::Add, _) => " ADD ".to_string(),
    };
    let field = Paragraph::new(field(&popup.editor))
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title(title));
    let width = bounds.width * 4 / 5;
    let height = field.line_count(width.saturating_sub(2)) as u16;
    let area = bounds.centered(Constraint::Length(width), Constraint::Length(height));
    frame.render_widget(Clear, area);
    frame.render_widget(field, area);
    // The cursor's cell is read back from the buffer: the paragraph does not tell where it wrapped the text.
    let cursor = area
        .positions()
        .find(|&cell| frame.buffer_mut()[cell].modifier.contains(Modifier::REVERSED));
    if let (Some(tag), Some(cursor)) = (popup.editor.tag(), cursor) {
        let (names, selected) = app.completions();
        let start = Position::new(cursor.x.saturating_sub(tag.chars().count() as u16), cursor.y);
        draw_completions(frame, &names, selected, start, bounds);
    }
}

/// Draws the completion `names`, five rows at most, under the tag at `tag` or above it without room, `selected` highlighted.
fn draw_completions(frame: &mut Frame, names: &[(String, usize)], selected: usize, tag: Position, bounds: Rect) {
    if names.is_empty() {
        return;
    }
    let width = names.iter().map(|(name, _)| name.chars().count()).max().unwrap_or_default();
    let rows = names
        .iter()
        .map(|(name, count)| Line::from_iter([Span::styled(format!("{name:<width$}"), tag_style(name)), format!(" {count:>3}").dim()]));
    let height = names.len().min(5) as u16 + 2;
    let y = if tag.y + 1 + height <= bounds.bottom() {
        tag.y + 1
    } else {
        tag.y.saturating_sub(height)
    };
    let area = Rect::new(tag.x.saturating_sub(1), y, width as u16 + 6, height).intersection(bounds);
    let list = List::new(rows).highlight_style(Style::new().bg(Color::DarkGray)).block(Block::bordered());
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(selected)));
}

/// The popup's text with the character under the cursor, or a space past the end, in reverse video.
fn field(editor: &Editor) -> Line<'static> {
    let mut chars = editor.text.chars();
    let before: String = chars.by_ref().take(editor.cursor).collect();
    let under = chars.next().map_or(" ".to_string(), String::from);
    Line::from_iter([before.into(), under.reversed(), chars.collect::<String>().into()])
}

/// Rows of the filter panel, entries under their section headers, and the row of the active filter when it is among them.
fn panel(app: &App) -> (Vec<Line<'static>>, Option<usize>) {
    let filters = app.filters();
    let active = app.filter_row(&filters);
    let (mut lines, mut row) = (Vec::new(), None);
    for (i, (term, count)) in filters.iter().enumerate() {
        let sigil = term.chars().next();
        let header = match sigil {
            Some('+') => " PROJECTS",
            Some('@') => " CONTEXTS",
            _ => "",
        };
        let style = if header.is_empty() { Style::new().bold() } else { tag_style(term) };
        if !header.is_empty() && filters[i - 1].0.chars().next() != sigil {
            lines.extend([Line::default(), Line::from(Span::styled(header, style.bold()))]);
        }
        let marker = if Some(i) == active {
            row = Some(lines.len());
            "▸ "
        } else {
            "  "
        };
        let name = Span::styled(format!("{term:<13.13}"), style);
        lines.push(Line::from_iter([marker.into(), name, " ".into(), format!("{count:>3}").dim()]));
    }
    (lines, row)
}

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, message: &str) {
    let text = vec![Line::from(format!(" {message}")).red(), Line::from(" press any key to quit").dim()];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
}

/// A listed task: its number then its todo.txt line, styled word by word with due dates against `today`, or all dimmed when done.
pub fn line(number: usize, todo: &Todo, today: Date) -> Line<'static> {
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
    spans.extend(words(&todo.description, Some(today)));
    Line::from(spans)
}

/// Words of `text`, a `due:` date styled against `today` when given, any other word by `tag_style`, with the spaces between them kept.
fn words(text: &str, today: Option<Date>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            spans.push(" ".into());
        }
        let due = word.strip_prefix(DUE).and_then(|value| due_style(value, today));
        spans.push(Span::styled(word.to_string(), due.unwrap_or_else(|| tag_style(word))));
    }
    spans
}

/// Style of a due date: red once past `today`, yellow on the day, none when later, not a date, or with no `today`.
fn due_style(value: &str, today: Option<Date>) -> Option<Style> {
    let (due, today) = (parse_date(value)?, today?);
    match due.cmp(&today) {
        Ordering::Less => Some(Style::new().red()),
        Ordering::Equal => Some(Style::new().yellow()),
        Ordering::Greater => None,
    }
}

/// Style of a word of a task: a `+project` magenta, an `@context` cyan, a `key:value` dimmed, anything else plain.
fn tag_style(word: &str) -> Style {
    match word.chars().next() {
        Some('+') if word.len() > 1 => Style::new().magenta(),
        Some('@') if word.len() > 1 => Style::new().cyan(),
        _ if Todo::is_key_value(word) => Style::new().dim(),
        _ => Style::new(),
    }
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
