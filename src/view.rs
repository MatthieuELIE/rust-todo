use std::cmp::Ordering;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListState, Padding, Paragraph, Wrap};
use time::Date;

use crate::editor::{Editor, Mode};
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, Focus, Group, Popup, Shown, Target, WAITING};

/// Catppuccin Mocha's mantle, herdr's popup colour, painted on every unset cell so the list and the popup's frame read as one.
const BACKGROUND: Color = Color::Rgb(0x18, 0x18, 0x25);

/// Where the keys go: the mode block, the border of the focused card or floating window (peach).
const ACCENT: Color = Color::Rgb(0xfa, 0xb3, 0x87);

/// Text on the accent (crust).
const ON_ACCENT: Color = Color::Rgb(0x11, 0x11, 0x1b);

/// Text read first, painted on every unset cell (text).
const PRIMARY: Color = Color::Rgb(0xcd, 0xd6, 0xf4);

/// Text read second: labels, section names, a message (subtext0).
const SECONDARY: Color = Color::Rgb(0xa6, 0xad, 0xc8);

/// Labels of the detail zone, a step greyer than the values they name (overlay2).
const LABEL: Color = Color::Rgb(0x93, 0x99, 0xb2);

/// Text read only when looked for: dates, `key:value` words, counts, key actions, done tasks (overlay1).
const TERTIARY: Color = Color::Rgb(0x7f, 0x84, 0x9c);

/// Rules and borders (surface1).
const STRUCTURE: Color = Color::Rgb(0x45, 0x47, 0x5a);

/// Background of a floating window, a step above the screen's (base).
const RAISED: Color = Color::Rgb(0x1e, 0x1e, 0x2e);

/// Background of the row under the cursor in the zone that gets the keys (surface0).
const SELECTED: Color = Color::Rgb(0x31, 0x32, 0x44);

/// Separators between keys, border of a floating window without the keys (surface2).
const SEPARATOR: Color = Color::Rgb(0x58, 0x5b, 0x70);

/// Background of a priority badge (surface1).
const BADGE: Color = STRUCTURE;

/// A `+project` (mauve).
const PROJECT: Color = Color::Rgb(0xcb, 0xa6, 0xf7);

/// An `@context` (teal).
const CONTEXT: Color = Color::Rgb(0x94, 0xe2, 0xd5);

/// A due date past, or something refused (red).
const ALERT: Color = Color::Rgb(0xf3, 0x8b, 0xa8);

/// A due date on the day (yellow).
const DUE_TODAY: Color = Color::Rgb(0xf9, 0xe2, 0xaf);

/// Priorities A to E: pink, green, blue, lavender, sky.
const PRIORITIES: [Color; 5] = [
    Color::Rgb(0xf5, 0xc2, 0xe7),
    Color::Rgb(0xa6, 0xe3, 0xa1),
    Color::Rgb(0x89, 0xb4, 0xfa),
    Color::Rgb(0xb4, 0xbe, 0xfe),
    Color::Rgb(0x89, 0xdc, 0xeb),
];

/// Height of the detail card under the list: its borders and five rows.
const DETAILS_HEIGHT: u16 = 7;

/// Width of the filter panel's card, its borders included.
const PANEL_WIDTH: u16 = 22;

/// Fewest rows the list keeps inside its card; on a screen too low for them and the detail card, that card is hidden.
const MIN_LIST_ROWS: u16 = 5;

/// Keys of the list and of the date picker shown by `?` under the mode's name, one per line, a key's alternatives separated by `/`.
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
q            quit

DATE
h/l/←/→      day
k/j/↑/↓      week
H/L          month
Enter/Esc    pick, close";

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
    let [panel_area, list_area] = Layout::horizontal([Constraint::Length(PANEL_WIDTH), Constraint::Fill(1)])
        .spacing(1)
        .areas(main_area);

    let (entries, row) = panel(app);
    let panel_focused = matches!(app.focus, Focus::Panel);
    let panel = List::new(entries).highlight_style(highlight(panel_focused)).block(card(panel_focused));
    frame.render_stateful_widget(panel, panel_area, &mut ListState::default().with_selected(row));

    if list_area.height >= MIN_LIST_ROWS + 2 + 1 + DETAILS_HEIGHT {
        let [list_area, details_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(DETAILS_HEIGHT)])
            .spacing(1)
            .areas(list_area);
        draw_list(frame, app, list_area, scroll, today);
        draw_details(frame, app.selected_task(), details_area, today);
    } else {
        draw_list(frame, app, list_area, scroll, today);
    }
    draw_status(frame, app, status_area);
    match &app.focus {
        Focus::Help => draw_help(frame, main_area),
        Focus::Popup(popup) => draw_popup(frame, app, popup, main_area, today),
        _ => {}
    }
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = BACKGROUND;
        }
        if cell.fg == Color::Reset {
            cell.fg = PRIMARY;
        }
    }
}

/// Draws the tasks on screen under their group headers, or says there is none; `scroll` keeps the offset between frames.
fn draw_list(frame: &mut Frame, app: &App, area: Rect, scroll: &mut ListState, today: Date) {
    let focused = matches!(app.focus, Focus::List | Focus::Search);
    let block = card(focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let area = inner;
    let shown = app.shown();
    if shown.is_empty() {
        frame.render_widget(
            Paragraph::new(if app.search.is_empty() && app.filter.is_none() {
                "nothing to do"
            } else {
                "no matching task"
            })
            .fg(TERTIARY),
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
                Shown::Header(group, count, folded) => header(group, count, folded, area.width.saturating_sub(2) as usize),
                Shown::Task(number, todo) => line(number, todo, today),
            });
        }
        let list = List::new(lines)
            .highlight_symbol(arrow(focused))
            .highlight_style(highlight(focused))
            .scroll_padding(1);
        scroll.select(rows.get(app.cursor).copied());
        frame.render_stateful_widget(list, area, scroll);
    }
}

/// Draws the detail zone: a ` DETAILS ` rule, then the priority or completion, text, dates, projects, contexts and key:values of `todo`.
fn draw_details(frame: &mut Frame, todo: Option<&Todo>, area: Rect, today: Date) {
    let block = card(false).title(" DETAILS ".fg(SECONDARY).bold());
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
        others.extend([
            if i > 0 { "  " } else { "" }.into(),
            format!("{key}:").fg(TERTIARY),
            value.to_string().into(),
        ]);
    }

    if todo.done {
        detail(frame, "Done", date(todo.completed), first_row);
    } else {
        let priority = todo.priority.map(badge);
        detail(frame, "Priority", priority.into_iter().collect(), first_row);
    }
    detail(frame, "Text", words(todo.text(), today), text_row);
    detail(frame, "Created", date(todo.created), created_area);
    detail(frame, "Due", due.into_iter().collect(), due_area);
    detail(frame, "Projects", tags('+', todo.projects()), projects_area);
    detail(frame, "Contexts", tags('@', todo.contexts()), contexts_area);
    draw_cut(frame, others, key_values_row);
    if todo.done {
        frame.buffer_mut().set_style(inner, Style::new().fg(TERTIARY));
    }
}

/// Draws a row of the detail zone, its `label` dimmed then its `value`.
fn detail(frame: &mut Frame, label: &str, value: Vec<Span<'static>>, area: Rect) {
    let mut line = Line::from(format!(" {label:<10}").fg(LABEL));
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
        Focus::Popup(popup) if popup.picker.is_some() => ("DATE", "hjkl move · H/L month · ⏎ pick · esc close"),
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
    let mut status = Line::from_iter([mode_block(mode), " ".into()]);
    for (i, word) in filters.split(' ').enumerate() {
        let style = if word == "+done" { Style::new() } else { tag_style(word) };
        status.extend([if i > 0 { " " } else { "" }.into(), Span::styled(word.to_string(), style)]);
    }
    let mut hints = Line::from(if filters.is_empty() { "" } else { "  " });
    for (i, hint) in keys.split(" · ").enumerate() {
        let (key, action) = hint.split_once(' ').unwrap_or_default();
        hints.extend([
            if i > 0 { " · " } else { "" }.fg(SEPARATOR),
            key.bold(),
            format!(" {action}").fg(TERTIARY),
        ]);
    }
    if app.message.is_none() && status.width() + hints.width() <= area.width as usize {
        status.extend(hints);
    }
    frame.render_widget(Paragraph::new(status), area);
    if let Some(message) = &app.message {
        let colour = if app.refused { ALERT } else { SECONDARY };
        frame.render_widget(Paragraph::new(format!("{message} ").fg(colour)).right_aligned(), area);
    }
}

/// Draws the key help in two columns in a rounded box, centred in `area`.
fn draw_help(frame: &mut Frame, area: Rect) {
    let height = HELP_LIST.lines().count().max(HELP_EDIT.lines().count()) as u16 + 4;
    let area = area.centered(Constraint::Length(82), Constraint::Length(height));
    let block = floating(true)
        .title(" HELP ")
        .title_bottom(Line::from(" any key closes ".fg(TERTIARY)).right_aligned())
        .padding(Padding::new(3, 3, 1, 1));
    let [list, edit] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(2).areas(block.inner(area));
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(help_lines(HELP_LIST)), list);
    frame.render_widget(Paragraph::new(help_lines(HELP_EDIT)), edit);
}

/// Lines of a help column: a mode name bold, keys bold, the `/` between alternatives and actions greyed.
fn help_lines(text: &'static str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| {
            if !line.contains(' ') {
                return Line::from(line.fg(SECONDARY).bold());
            }
            let (key, action) = line.split_at(line.char_indices().nth(13).map_or(line.len(), |(i, _)| i));
            let alternatives: Vec<&str> = if key.trim_end() == "/" { vec![key] } else { key.split('/').collect() };
            let mut spans = Vec::new();
            for (i, alternative) in alternatives.into_iter().enumerate() {
                if i > 0 {
                    spans.push("/".fg(SEPARATOR));
                }
                spans.push(alternative.bold());
            }
            spans.push(action.fg(TERTIARY));
            Line::from(spans)
        })
        .collect()
}

/// A mode's name in bold on the accent, as the status bar shows it.
fn mode_block(mode: &str) -> Span<'static> {
    format!(" {mode} ").bold().fg(ON_ACCENT).bg(ACCENT)
}

/// Draws the popup centred in `bounds`, titled by the task edited or the term an add gets, with the tag's completions or the date picker.
fn draw_popup(frame: &mut Frame, app: &App, popup: &Popup, bounds: Rect, today: Date) {
    let title = match (&popup.target, app.filter.as_deref()) {
        (Target::Edit(number), _) => Line::from(format!(" EDIT {number} ")),
        (Target::Add, Some(term)) if term != WAITING => {
            Line::from_iter([" ADD (".into(), Span::styled(term.to_string(), tag_style(term)), ") ".into()])
        }
        (Target::Add, _) => Line::from(" ADD "),
    };
    let field = Paragraph::new(field(&popup.editor, today))
        .wrap(Wrap { trim: false })
        .block(floating(popup.picker.is_none()).title(title));
    let width = bounds.width * 4 / 5;
    let height = field.line_count(width.saturating_sub(2)) as u16;
    let area = bounds.centered(Constraint::Length(width), Constraint::Length(height));
    frame.render_widget(Clear, area);
    frame.render_widget(field, area);
    // The cursor's cell is marked reversed then read back from the buffer, since the paragraph does not tell where it wrapped
    // the text; the mark is taken off and the terminal's own cursor stands there.
    let cursor = area
        .positions()
        .find(|&cell| frame.buffer_mut()[cell].modifier.contains(Modifier::REVERSED));
    if let Some(cursor) = cursor {
        frame.buffer_mut()[cursor].modifier.remove(Modifier::REVERSED);
        if popup.picker.is_none() {
            frame.set_cursor_position(cursor);
        }
    }
    if let (Some(tag), Some(cursor)) = (popup.editor.tag(), cursor) {
        let (names, selected) = app.completions();
        let start = Position::new(cursor.x.saturating_sub(tag.chars().count() as u16), cursor.y);
        draw_completions(frame, &names, selected, start, bounds);
    }
    if let (Some(date), Some(cursor)) = (popup.picker, cursor) {
        draw_picker(
            frame,
            date,
            today,
            Position::new(cursor.x.saturating_sub(DUE.len() as u16), cursor.y),
            bounds,
        );
    }
}

/// Draws the date picker under the word at `word`: `date`'s month from Monday, `date` in the accent, `today` yellow, past days greyed.
fn draw_picker(frame: &mut Frame, date: Date, today: Date, word: Position, bounds: Rect) {
    let first = date.replace_day(1).expect("every month has a first day");
    let offset = first.weekday().number_days_from_monday();
    let mut lines = vec![Line::from("Mo Tu We Th Fr Sa Su".fg(TERTIARY))];
    let mut week = Line::from("   ".repeat(offset.into()));
    for day in 1..=date.month().length(date.year()) {
        let style = match first.replace_day(day).map(|shown| shown.cmp(&today)) {
            _ if day == date.day() => Style::new().bold().fg(ACCENT).bg(SELECTED),
            Ok(Ordering::Less) => Style::new().fg(TERTIARY),
            Ok(Ordering::Equal) => Style::new().fg(DUE_TODAY),
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
    frame.render_widget(Paragraph::new(lines).block(floating(true).title(title)), area);
}

/// Draws the completion `names`, five rows at most, under the tag at `tag` or above it without room, `selected` highlighted.
fn draw_completions(frame: &mut Frame, names: &[(String, usize)], selected: usize, tag: Position, bounds: Rect) {
    if names.is_empty() {
        return;
    }
    let width = names.iter().map(|(name, _)| name.chars().count()).max().unwrap_or_default();
    let rows = names.iter().map(|(name, count)| {
        Line::from_iter([
            Span::styled(format!("{name:<width$}"), tag_style(name)),
            format!(" {count:>3}").fg(TERTIARY),
        ])
    });
    let area = drop_down(tag, width as u16 + 6, names.len().min(5) as u16 + 2, bounds);
    let list = List::new(rows).highlight_style(highlight(true)).block(floating(true));
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

/// The popup's text styled as a task is, one span per character, the one under the cursor, or a space past the end, reversed.
fn field(editor: &Editor, today: Date) -> Line<'static> {
    let mut chars: Vec<Span<'static>> = typed(&editor.text, today)
        .into_iter()
        .flat_map(|span| span.content.chars().map(|c| Span::styled(c.to_string(), span.style)).collect::<Vec<_>>())
        .collect();
    if editor.cursor >= chars.len() {
        chars.push(" ".into());
    }
    chars[editor.cursor].style = chars[editor.cursor].style.reversed();
    Line::from(chars)
}

/// Words of a line being typed, the priority and dates before the description styled as in the list, then its words.
fn typed(text: &str, today: Date) -> Vec<Span<'static>> {
    let description = Todo::from_line(text).description;
    let head = &text[..text.len() - description.len()];
    let mut spans = Vec::new();
    for word in head.split_inclusive(' ') {
        let letter = word.trim_end().strip_prefix('(').and_then(|rest| rest.strip_suffix(')'));
        let style = match letter.and_then(|letter| letter.parse::<char>().ok()) {
            Some(letter) => Style::new().bold().fg(priority(letter)),
            None if parse_date(word.trim_end()).is_some() => Style::new().fg(TERTIARY),
            None => Style::new(),
        };
        spans.push(Span::styled(word.to_string(), style));
    }
    spans.extend(words(&description, Some(today)));
    spans
}

/// Rows of the filter panel, entries under their section headers, and the row of the active filter when it is among them.
fn panel(app: &App) -> (Vec<Line<'static>>, Option<usize>) {
    let focused = matches!(app.focus, Focus::Panel);
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
            lines.extend([Line::default(), Line::from(header.fg(SECONDARY).bold())]);
        }
        let marker = if Some(i) == active {
            row = Some(lines.len());
            arrow(focused)
        } else {
            "  ".into()
        };
        let name = Span::styled(format!("{term:<13.13}"), style);
        lines.push(Line::from_iter([marker, name, " ".into(), format!("{count:>3}").fg(TERTIARY)]));
    }
    (lines, row)
}

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, message: &str) {
    let text = vec![
        Line::from(format!(" {message}")).fg(ALERT),
        Line::from(" press any key to quit").fg(TERTIARY),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
}

/// A listed task: its number then its todo.txt line, the priority as a badge, due dates against `today`, struck through when done.
pub fn line(number: usize, todo: &Todo, today: Date) -> Line<'static> {
    let number = format!("{number:>3}  ");
    if todo.done {
        return Line::from_iter([number.into(), todo.to_line().fg(TERTIARY).crossed_out()]);
    }
    let mut spans = vec![Span::raw(number)];
    spans.extend(todo.priority.map(badge));
    if todo.priority.is_some() {
        spans.push(" ".into());
    }
    if let Some(created) = todo.created {
        spans.extend([created.to_string().fg(TERTIARY), " ".into()]);
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
        Ordering::Less => Some(Style::new().fg(ALERT)),
        Ordering::Equal => Some(Style::new().fg(DUE_TODAY)),
        Ordering::Greater => None,
    }
}

/// Style of a word of a task: a `+project` mauve, an `@context` teal, a `key:value` greyed, anything else plain.
fn tag_style(word: &str) -> Style {
    match word.chars().next() {
        Some('+') if word.len() > 1 => Style::new().fg(PROJECT),
        Some('@') if word.len() > 1 => Style::new().fg(CONTEXT),
        _ if Todo::is_key_value(word) => Style::new().fg(TERTIARY),
        _ => Style::new(),
    }
}

/// Header of a group of the list: its count, its title, then a rule filling `width`, ended by ` ▸` when the group is folded.
fn header(group: Group, count: usize, folded: bool, width: usize) -> Line<'static> {
    let (title, style) = match group {
        Group::Priority(letter) => (format!("PRIORITY {letter}"), Style::new().bold().fg(priority(letter))),
        Group::Unprioritised => ("NO PRIORITY".to_string(), Style::new().bold().fg(SECONDARY)),
        Group::Done => ("DONE".to_string(), Style::new().bold().fg(TERTIARY)),
    };
    let count = format!(" ({count})");
    let end = if folded { " ▸" } else { "" };
    let rule = "─".repeat(width.saturating_sub(count.len() + title.len() + 4 + end.chars().count()));
    Line::from_iter([
        count.fg(TERTIARY),
        "  ".into(),
        Span::styled(title, style),
        "  ".into(),
        rule.fg(STRUCTURE),
        end.into(),
    ])
}

/// A zone's card: a rounded border, in the accent when the zone gets the keys.
fn card(focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { ACCENT } else { STRUCTURE }))
}

/// A floating window's frame: rounded on a raised background, its border in the accent when it gets the keys, its titles bold.
fn floating(focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { ACCENT } else { SEPARATOR }))
        .title_style(Style::new().fg(PRIMARY).bold())
        .style(Style::new().bg(RAISED))
}

/// The cursor's mark, in the accent when its zone gets the keys.
fn arrow(focused: bool) -> Span<'static> {
    "→ ".fg(if focused { ACCENT } else { TERTIARY })
}

/// Style of the row under the cursor: highlighted only when its zone gets the keys.
fn highlight(focused: bool) -> Style {
    if focused { Style::new().bg(SELECTED) } else { Style::new() }
}

/// Colour of a priority, `A` to `E`.
fn priority(letter: char) -> Color {
    PRIORITIES[(letter as usize).saturating_sub('A' as usize).min(PRIORITIES.len() - 1)]
}

/// A priority as a badge: its letter bold in its colour between two spaces, on a dark background.
fn badge(letter: char) -> Span<'static> {
    Span::styled(format!(" {letter} "), Style::new().bold().fg(priority(letter)).bg(BADGE))
}

#[cfg(test)]
mod tests;
