use std::cmp::Ordering;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListState, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use time::Date;

use crate::editor::Editor;
use crate::theme::Theme;
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, Focus, Popup, Target, WAITING};
use list::draw_list;
use panel::{PANEL_WIDTH, panel};
use status::draw_status;
use style::{badge, card, due_style, floating, highlight, icon, priority, tag_style, words};

const DETAILS_ICON: char = '\u{f05a}';

/// Width of the help box, its borders included.
const HELP_WIDTH: u16 = 46;

/// Tallest the help box grows, its borders and padding included.
const HELP_MAX_HEIGHT: u16 = 30;

/// Height of the detail card under the list when its text fits one line: its borders and five rows; `Text` wraps and the card grows with it.
const DETAILS_HEIGHT: u16 = 7;

/// Fewest rows the list keeps inside its card; on a screen too low for them and the detail card, that card is hidden.
const MIN_LIST_ROWS: u16 = 5;

/// Keys of every screen `?` can show, one section per mode under its name, one key per line, a key's alternatives separated by `/`.
pub const HELP: &str = "\
LIST
j/k/↓/↑      move
gg/G         top, bottom
Enter        edit
o            add
x            done: moves to done.txt
dd           delete
yy           copy the task
p a…e        priority
p Space      no priority
u/Ctrl-r     undo, redo
zM/zR        fold, unfold all
za           fold, unfold group
/            search
D            due tasks
H            show, hide history
r            reopen, in the history
Tab          panel
Esc          drop filter and search
?            these keys
q            quit

PANEL
j/k          pick a filter
Esc          all tasks
Tab/Enter    back to the list

INSERT
Enter        save
Esc          normal mode
Ctrl-w       erase a word
Ctrl-u       erase to the start
Tab          complete +, @ or wait:
↓/↑/Ctrl-n/p pick a completion

NORMAL
h/l/0/$      move
w/b/e        word
W/B/E        blank-separated word
f/F/t/T x    to, before a char
;/,          repeat, reverse
x/D/C        delete, to end, change
d/c + move   delete, change up to
dd/cc        delete, change the line
iw/aw/iW/aW  after d/c: a word
i/a/I/A      insert
u/Ctrl-r     undo, redo
p a…e        priority
p Space      no priority
Esc          cancel

DATE
h/l/←/→      day
k/j/↑/↓      week
H/L          month
Enter/Esc    pick, close";

/// Draws the filter panel and the task list above the status bar, due dates against `today`; `scroll` keeps the list's offset between frames.
pub fn draw(frame: &mut Frame, app: &App, scroll: &mut ListState, today: Date) {
    let theme = &app.theme;
    let [main_area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let [panel_area, list_area] = Layout::horizontal([Constraint::Length(PANEL_WIDTH), Constraint::Fill(1)])
        .spacing(1)
        .areas(main_area);

    let (entries, row) = panel(app);
    let panel_focused = matches!(app.focus, Focus::Panel);
    let panel = List::new(entries)
        .highlight_style(highlight(theme, panel_focused))
        .block(card(theme, panel_focused));
    frame.render_stateful_widget(panel, panel_area, &mut ListState::default().with_selected(row));

    if list_area.height >= MIN_LIST_ROWS + 2 + 1 + DETAILS_HEIGHT {
        let spare = list_area.height - (MIN_LIST_ROWS + 2 + 1 + DETAILS_HEIGHT);
        let text_width = list_area.width.saturating_sub(2 + label(theme, "Text").width() as u16);
        let text_rows = app.selected_task().map_or(1, |todo| {
            text_of(theme, todo, (!todo.done).then_some(today)).line_count(text_width) as u16
        });
        let height = DETAILS_HEIGHT + text_rows.saturating_sub(1).min(spare);
        let [list_area, details_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(height)])
            .spacing(1)
            .areas(list_area);
        draw_list(frame, app, list_area, scroll, today);
        draw_details(frame, theme, app.icons, app.selected_task(), details_area, today);
    } else {
        draw_list(frame, app, list_area, scroll, today);
    }
    draw_status(frame, app, status_area);
    match &app.focus {
        Focus::Help(scroll) => draw_help(frame, app, *scroll, main_area),
        Focus::Popup(popup) => draw_popup(frame, app, popup, main_area, today),
        _ => {}
    }
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = theme.background;
        }
        if cell.fg == Color::Reset {
            cell.fg = theme.primary;
        }
    }
}

/// Draws the detail zone: a ` DETAILS ` rule, then the priority or completion, text, dates, projects, contexts and key:values of `todo`.
/// The text wraps under its value over the rows `area` leaves it, its last row ended by `…` when it is cut; the other rows are cut by `…`.
fn draw_details(frame: &mut Frame, theme: &Theme, icons: bool, todo: Option<&Todo>, area: Rect, today: Date) {
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

/// Draws the key help in one scrolling column in a rounded box, centred in `area`, and tells the app how many lines it shows.
fn draw_help(frame: &mut Frame, app: &App, scroll: u16, area: Rect) {
    let theme = &app.theme;
    let lines = HELP.lines().count() as u16;
    let height = (lines + 4).min(HELP_MAX_HEIGHT).min(area.height);
    let area = area.centered(Constraint::Length(HELP_WIDTH), Constraint::Length(height));
    let block = floating(theme, true)
        .title(" HELP ")
        .title_bottom(Line::from(" j/k scroll · esc close ".fg(theme.tertiary)).right_aligned())
        .padding(Padding::new(3, 3, 1, 1));
    let inner = block.inner(area);
    app.help_rows.set(inner.height);
    let max = lines.saturating_sub(inner.height);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(help_lines(theme, HELP)).scroll((scroll.min(max), 0)), inner);
    if max > 0 {
        let bar = Rect::new(area.right() - 3, inner.y, 1, inner.height);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("▐"))
            .thumb_symbol("▐")
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::new().fg(theme.separator))
            .thumb_style(Style::new().fg(theme.tertiary));
        let mut state = ScrollbarState::new(max as usize + 1)
            .viewport_content_length(inner.height as usize)
            .position(scroll.min(max) as usize);
        frame.render_stateful_widget(scrollbar, bar, &mut state);
    }
}

/// Lines of the help: a mode name bold, keys bold, the `/` between alternatives and actions greyed.
fn help_lines(theme: &Theme, text: &'static str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| {
            if !line.contains(' ') {
                return Line::from(line.fg(theme.secondary).bold());
            }
            let (key, action) = line.split_at(line.char_indices().nth(13).map_or(line.len(), |(i, _)| i));
            let alternatives: Vec<&str> = if key.trim_end() == "/" { vec![key] } else { key.split('/').collect() };
            let mut spans = Vec::new();
            for (i, alternative) in alternatives.into_iter().enumerate() {
                if i > 0 {
                    spans.push("/".fg(theme.separator));
                }
                spans.push(alternative.bold());
            }
            spans.push(action.fg(theme.tertiary));
            Line::from(spans)
        })
        .collect()
}

/// Draws the popup centred in `bounds`, titled by the task edited or the term an add gets, with the tag's completions or the date picker.
fn draw_popup(frame: &mut Frame, app: &App, popup: &Popup, bounds: Rect, today: Date) {
    let theme = &app.theme;
    let title = match (&popup.target, app.filter.as_deref()) {
        (Target::Edit(number), _) => Line::from(format!(" EDIT {number} ")),
        (Target::Add, Some(term)) if term != WAITING => {
            Line::from_iter([" ADD (".into(), Span::styled(term.to_string(), tag_style(theme, term)), ") ".into()])
        }
        (Target::Add, _) => Line::from(" ADD "),
    };
    let field = Paragraph::new(field(theme, &popup.editor, today))
        .wrap(Wrap { trim: false })
        .block(floating(theme, popup.picker.is_none()).title(title).padding(Padding::right(1)));
    let width = bounds.width * 4 / 5;
    let height = field.line_count(width.saturating_sub(3)) as u16;
    let area = bounds.centered(Constraint::Length(width), Constraint::Length(height));
    frame.render_widget(Clear, area);
    frame.render_widget(field, area);
    // The cursor's cell is marked reversed then read back from the buffer, since the paragraph does not tell where it wrapped
    // the text; the marks are taken off and the terminal's own cursor stands there. A blank at the end of a wrapped row is not
    // drawn: the cursor then stands after the last cell drawn before it, marked hidden, in the column kept free on the right.
    let marked = |frame: &mut Frame, mark| area.positions().filter(|&cell| frame.buffer_mut()[cell].modifier.contains(mark)).last();
    let after = marked(frame, Modifier::HIDDEN).map(|cell| {
        let width = Span::raw(frame.buffer_mut()[cell].symbol()).width() as u16;
        Position::new(cell.x + width, cell.y)
    });
    let cursor = marked(frame, Modifier::REVERSED).or(after);
    for cell in area.positions() {
        frame.buffer_mut()[cell].modifier.remove(Modifier::REVERSED | Modifier::HIDDEN);
    }
    if let Some(cursor) = cursor
        && popup.picker.is_none()
    {
        frame.set_cursor_position(cursor);
    }
    if let (Some(tag), Some(cursor)) = (popup.editor.tag(), cursor) {
        let (names, selected) = app.completions();
        let start = Position::new(cursor.x.saturating_sub(tag.chars().count() as u16), cursor.y);
        draw_completions(frame, theme, &names, selected, start, bounds);
    }
    if let (Some(date), Some(cursor)) = (popup.picker, cursor) {
        draw_picker(
            frame,
            theme,
            date,
            today,
            Position::new(cursor.x.saturating_sub(DUE.len() as u16), cursor.y),
            bounds,
        );
    }
}

/// Draws the date picker under the word at `word`: `date`'s month from Monday, `date` in the accent, `today` yellow, past days greyed.
fn draw_picker(frame: &mut Frame, theme: &Theme, date: Date, today: Date, word: Position, bounds: Rect) {
    let first = date.replace_day(1).expect("every month has a first day");
    let offset = first.weekday().number_days_from_monday();
    let mut lines = vec![Line::from("Mo Tu We Th Fr Sa Su".fg(theme.tertiary))];
    let mut week = Line::from("   ".repeat(offset.into()));
    for day in 1..=date.month().length(date.year()) {
        let style = match first.replace_day(day).map(|shown| shown.cmp(&today)) {
            _ if day == date.day() => Style::new().bold().fg(theme.accent).bg(theme.selected),
            Ok(Ordering::Less) => Style::new().fg(theme.tertiary),
            Ok(Ordering::Equal) => Style::new().fg(theme.due_today),
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
    frame.render_widget(Paragraph::new(lines).block(floating(theme, true).title(title)), area);
}

/// Draws the completion `names`, five rows at most, under the tag at `tag` or above it without room, `selected` highlighted.
fn draw_completions(frame: &mut Frame, theme: &Theme, names: &[(String, usize)], selected: usize, tag: Position, bounds: Rect) {
    if names.is_empty() {
        return;
    }
    let width = names.iter().map(|(name, _)| name.chars().count()).max().unwrap_or_default();
    let rows = names.iter().map(|(name, count)| {
        Line::from_iter([
            Span::styled(format!("{name:<width$}"), tag_style(theme, name)),
            format!(" {count:>3}").fg(theme.tertiary),
        ])
    });
    let area = drop_down(tag, width as u16 + 6, names.len().min(5) as u16 + 2, bounds);
    let list = List::new(rows)
        .highlight_style(highlight(theme, true).bold())
        .block(floating(theme, true));
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

/// The popup's text styled as a task is, one span per character, the one under the cursor, or a space past the end, reversed, those before it hidden.
fn field(theme: &Theme, editor: &Editor, today: Date) -> Line<'static> {
    let mut chars: Vec<Span<'static>> = typed(theme, &editor.text, today)
        .into_iter()
        .flat_map(|span| span.content.chars().map(|c| Span::styled(c.to_string(), span.style)).collect::<Vec<_>>())
        .collect();
    if editor.cursor >= chars.len() {
        chars.push(" ".into());
    }
    chars[editor.cursor].style = chars[editor.cursor].style.reversed();
    for before in &mut chars[..editor.cursor] {
        before.style = before.style.hidden();
    }
    Line::from(chars)
}

/// Words of a line being typed, the priority and dates before the description styled as in the list, then its words.
fn typed(theme: &Theme, text: &str, today: Date) -> Vec<Span<'static>> {
    let description = Todo::from_line(text).description;
    let head = &text[..text.len() - description.len()];
    let mut spans = Vec::new();
    for word in head.split_inclusive(' ') {
        let letter = word.trim_end().strip_prefix('(').and_then(|rest| rest.strip_suffix(')'));
        let style = match letter.and_then(|letter| letter.parse::<char>().ok()) {
            Some(letter) => Style::new().bold().fg(priority(theme, letter)),
            None if parse_date(word.trim_end()).is_some() => Style::new().fg(theme.tertiary),
            None => Style::new(),
        };
        spans.push(Span::styled(word.to_string(), style));
    }
    spans.extend(words(theme, &description, Some(today)));
    spans
}

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, theme: &Theme, message: &str) {
    let text = vec![
        Line::from(format!(" {message}")).fg(theme.alert),
        Line::from(" press any key to quit").fg(theme.tertiary),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
}

mod list;
mod panel;
mod status;
mod style;

pub use list::{line, number};

#[cfg(test)]
mod tests;
