use std::cmp::Ordering;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListState, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use time::Date;

use crate::editor::{Editor, Mode};
use crate::todo::{DUE, Todo, parse_date};
use crate::tui::{App, DUE_NOW, Focus, Group, Popup, Shown, Target, WAITING};

const PROJECTS_ICON: char = '\u{f07b}';
const CONTEXTS_ICON: char = '\u{f1fa}';
const DETAILS_ICON: char = '\u{f05a}';
const ALL_TASKS_ICON: char = '\u{f0ca}';
const DUE_ICON: char = '\u{f073}';
const WAITING_ICON: char = '\u{f252}';

/// The glyph and the space after it when `icons` is on, else nothing.
fn icon(icons: bool, glyph: char) -> String {
    if icons { format!("{glyph} ") } else { String::new() }
}

/// The colour of each role of the list; a role's colour is the same wherever it is drawn.
#[derive(PartialEq)]
pub struct Theme {
    /// Painted on every unset cell so the list and the popup's frame read as one.
    background: Color,
    /// Where the keys go: the mode block, the border of the focused card or floating window.
    accent: Color,
    /// Text on the accent.
    on_accent: Color,
    /// Text read first, painted on every unset cell.
    primary: Color,
    /// Text read second: labels, section names, a message, the history.
    secondary: Color,
    /// Labels of the detail zone, a step greyer than the values they name.
    label: Color,
    /// Text read only when looked for: dates, `key:value` words, counts, key actions, done tasks.
    tertiary: Color,
    /// Rules and borders.
    structure: Color,
    /// Background of a floating window, a step above the screen's.
    raised: Color,
    /// Background of the row under the cursor in the zone that gets the keys.
    selected: Color,
    /// Separators between keys, border of a floating window without the keys.
    separator: Color,
    /// Background of a priority badge.
    badge: Color,
    /// A `+project`.
    project: Color,
    /// An `@context`.
    context: Color,
    /// A due date past, or something refused.
    alert: Color,
    /// A due date on the day.
    due_today: Color,
    /// Priorities A to E.
    priorities: [Color; 5],
}

impl Theme {
    /// Catppuccin Mocha, the background being its mantle, herdr's popup colour.
    pub const MOCHA: Theme = Theme {
        background: Color::Rgb(0x18, 0x18, 0x25),
        accent: Color::Rgb(0xfa, 0xb3, 0x87),
        on_accent: Color::Rgb(0x11, 0x11, 0x1b),
        primary: Color::Rgb(0xcd, 0xd6, 0xf4),
        secondary: Color::Rgb(0xa6, 0xad, 0xc8),
        label: Color::Rgb(0x93, 0x99, 0xb2),
        tertiary: Color::Rgb(0x7f, 0x84, 0x9c),
        structure: Color::Rgb(0x45, 0x47, 0x5a),
        raised: Color::Rgb(0x1e, 0x1e, 0x2e),
        selected: Color::Rgb(0x31, 0x32, 0x44),
        separator: Color::Rgb(0x58, 0x5b, 0x70),
        badge: Color::Rgb(0x45, 0x47, 0x5a),
        project: Color::Rgb(0xcb, 0xa6, 0xf7),
        context: Color::Rgb(0x94, 0xe2, 0xd5),
        alert: Color::Rgb(0xf3, 0x8b, 0xa8),
        due_today: Color::Rgb(0xf9, 0xe2, 0xaf),
        priorities: [
            Color::Rgb(0xf5, 0xc2, 0xe7),
            Color::Rgb(0xa6, 0xe3, 0xa1),
            Color::Rgb(0x89, 0xb4, 0xfa),
            Color::Rgb(0xb4, 0xbe, 0xfe),
            Color::Rgb(0x89, 0xdc, 0xeb),
        ],
    };

    /// Catppuccin Latte, with the roles of Mocha.
    pub const LATTE: Theme = Theme {
        background: Color::Rgb(0xe6, 0xe9, 0xef),
        accent: Color::Rgb(0xfe, 0x64, 0x0b),
        on_accent: Color::Rgb(0xdc, 0xe0, 0xe8),
        primary: Color::Rgb(0x4c, 0x4f, 0x69),
        secondary: Color::Rgb(0x6c, 0x6f, 0x85),
        label: Color::Rgb(0x7c, 0x7f, 0x93),
        tertiary: Color::Rgb(0x8c, 0x8f, 0xa1),
        structure: Color::Rgb(0xbc, 0xc0, 0xcc),
        raised: Color::Rgb(0xef, 0xf1, 0xf5),
        selected: Color::Rgb(0xcc, 0xd0, 0xda),
        separator: Color::Rgb(0xac, 0xb0, 0xbe),
        badge: Color::Rgb(0xbc, 0xc0, 0xcc),
        project: Color::Rgb(0x88, 0x39, 0xef),
        context: Color::Rgb(0x17, 0x92, 0x99),
        alert: Color::Rgb(0xd2, 0x0f, 0x39),
        due_today: Color::Rgb(0xdf, 0x8e, 0x1d),
        priorities: [
            Color::Rgb(0xea, 0x76, 0xcb),
            Color::Rgb(0x40, 0xa0, 0x2b),
            Color::Rgb(0x1e, 0x66, 0xf5),
            Color::Rgb(0x72, 0x87, 0xfd),
            Color::Rgb(0x04, 0xa5, 0xe5),
        ],
    };

    /// Catppuccin Frappé, with the roles of Mocha.
    pub const FRAPPE: Theme = Theme {
        background: Color::Rgb(0x29, 0x2c, 0x3c),
        accent: Color::Rgb(0xef, 0x9f, 0x76),
        on_accent: Color::Rgb(0x23, 0x26, 0x34),
        primary: Color::Rgb(0xc6, 0xd0, 0xf5),
        secondary: Color::Rgb(0xa5, 0xad, 0xce),
        label: Color::Rgb(0x94, 0x9c, 0xbb),
        tertiary: Color::Rgb(0x83, 0x8b, 0xa7),
        structure: Color::Rgb(0x51, 0x57, 0x6d),
        raised: Color::Rgb(0x30, 0x34, 0x46),
        selected: Color::Rgb(0x41, 0x45, 0x59),
        separator: Color::Rgb(0x62, 0x68, 0x80),
        badge: Color::Rgb(0x51, 0x57, 0x6d),
        project: Color::Rgb(0xca, 0x9e, 0xe6),
        context: Color::Rgb(0x81, 0xc8, 0xbe),
        alert: Color::Rgb(0xe7, 0x82, 0x84),
        due_today: Color::Rgb(0xe5, 0xc8, 0x90),
        priorities: [
            Color::Rgb(0xf4, 0xb8, 0xe4),
            Color::Rgb(0xa6, 0xd1, 0x89),
            Color::Rgb(0x8c, 0xaa, 0xee),
            Color::Rgb(0xba, 0xbb, 0xf1),
            Color::Rgb(0x99, 0xd1, 0xdb),
        ],
    };

    /// Catppuccin Macchiato, with the roles of Mocha.
    pub const MACCHIATO: Theme = Theme {
        background: Color::Rgb(0x1e, 0x20, 0x30),
        accent: Color::Rgb(0xf5, 0xa9, 0x7f),
        on_accent: Color::Rgb(0x18, 0x19, 0x26),
        primary: Color::Rgb(0xca, 0xd3, 0xf5),
        secondary: Color::Rgb(0xa5, 0xad, 0xcb),
        label: Color::Rgb(0x93, 0x9a, 0xb7),
        tertiary: Color::Rgb(0x80, 0x87, 0xa2),
        structure: Color::Rgb(0x49, 0x4d, 0x64),
        raised: Color::Rgb(0x24, 0x27, 0x3a),
        selected: Color::Rgb(0x36, 0x3a, 0x4f),
        separator: Color::Rgb(0x5b, 0x60, 0x78),
        badge: Color::Rgb(0x49, 0x4d, 0x64),
        project: Color::Rgb(0xc6, 0xa0, 0xf6),
        context: Color::Rgb(0x8b, 0xd5, 0xca),
        alert: Color::Rgb(0xed, 0x87, 0x96),
        due_today: Color::Rgb(0xee, 0xd4, 0x9f),
        priorities: [
            Color::Rgb(0xf5, 0xbd, 0xe6),
            Color::Rgb(0xa6, 0xda, 0x95),
            Color::Rgb(0x8a, 0xad, 0xf4),
            Color::Rgb(0xb7, 0xbd, 0xf8),
            Color::Rgb(0x91, 0xd7, 0xe3),
        ],
    };

    /// No colour at all, for `NO_COLOR`: every role is the terminal's own.
    pub const NONE: Theme = Theme {
        background: Color::Reset,
        accent: Color::Reset,
        on_accent: Color::Reset,
        primary: Color::Reset,
        secondary: Color::Reset,
        label: Color::Reset,
        tertiary: Color::Reset,
        structure: Color::Reset,
        raised: Color::Reset,
        selected: Color::Reset,
        separator: Color::Reset,
        badge: Color::Reset,
        project: Color::Reset,
        context: Color::Reset,
        alert: Color::Reset,
        due_today: Color::Reset,
        priorities: [Color::Reset; 5],
    };

    /// The terminal's own ANSI colours: default text and background, no fill for the selection or the badge.
    pub const TERMINAL: Theme = Theme {
        background: Color::Reset,
        accent: Color::Blue,
        on_accent: Color::Black,
        primary: Color::Reset,
        secondary: Color::Reset,
        label: Color::Reset,
        tertiary: Color::DarkGray,
        structure: Color::DarkGray,
        raised: Color::Reset,
        selected: Color::Reset,
        separator: Color::DarkGray,
        badge: Color::Reset,
        project: Color::Magenta,
        context: Color::Cyan,
        alert: Color::Red,
        due_today: Color::Yellow,
        priorities: [Color::Green; 5],
    };
}

impl Default for Theme {
    fn default() -> Self {
        Theme::TERMINAL
    }
}

/// Width of the help box, its borders included.
const HELP_WIDTH: u16 = 46;

/// Tallest the help box grows, its borders and padding included.
const HELP_MAX_HEIGHT: u16 = 30;

/// Height of the detail card under the list when its text fits one line: its borders and five rows; `Text` wraps and the card grows with it.
const DETAILS_HEIGHT: u16 = 7;

/// Width of the filter panel's card, its borders included.
const PANEL_WIDTH: u16 = 22;

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

/// Draws the tasks on screen under their group headers, or says there is none; `scroll` keeps the offset between frames.
fn draw_list(frame: &mut Frame, app: &App, area: Rect, scroll: &mut ListState, today: Date) {
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

/// Draws the status bar: mode block, active filters, then the mode's keys when they fit; on the right the message, or the keys of a command under way.
fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let (mode, keys) = match &app.focus {
        Focus::Search => ("SEARCH", "⏎ keep · esc clear"),
        Focus::Popup(popup) if popup.picker.is_some() => ("DATE", "hjkl move · H/L month · ⏎ pick · esc close"),
        Focus::Popup(popup) => match popup.editor.mode {
            Mode::Insert if app.completions().0.is_empty() => ("INSERT", "esc normal · ⏎ save"),
            Mode::Insert => ("INSERT", "esc normal · ⏎ save · tab complete"),
            Mode::Normal => ("NORMAL", "i insert · p priority · ⏎ save · esc cancel"),
        },
        Focus::Panel => ("PANEL", "j/k filter · esc all tasks · tab back"),
        Focus::List | Focus::Help(_) if app.in_history => ("LIST", "r reopen · H back · ? help"),
        Focus::List | Focus::Help(_) => ("LIST", "⏎ edit · o add · x done · p priority · ? help"),
    };
    let filters = if matches!(app.focus, Focus::Search) {
        format!("/{}▌", app.search)
    } else {
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        [app.filter.clone(), search].into_iter().flatten().collect::<Vec<_>>().join("  ")
    };
    let mut status = Line::from_iter([mode_block(theme, mode), " ".into()]);
    for (i, word) in filters.split(' ').enumerate() {
        status.extend([
            if i > 0 { " " } else { "" }.into(),
            Span::styled(word.to_string(), tag_style(theme, word)),
        ]);
    }
    let mut hints = Line::from(if filters.is_empty() { "" } else { "  " });
    for (i, hint) in keys.split(" · ").enumerate() {
        let (key, action) = hint.split_once(' ').unwrap_or_default();
        hints.extend([
            if i > 0 { " · " } else { "" }.fg(theme.separator),
            key.bold(),
            format!(" {action}").fg(theme.tertiary),
        ]);
    }
    let pending = app.pending();
    let reserved = if pending.is_empty() { 0 } else { pending.len() + 2 };
    if app.message.is_none() && status.width() + hints.width() + reserved <= area.width as usize {
        status.extend(hints);
    }
    frame.render_widget(Paragraph::new(status), area);
    if let Some(right) = app.message.as_deref().or((!pending.is_empty()).then_some(&pending)) {
        let colour = if app.refused { theme.alert } else { theme.secondary };
        frame.render_widget(Paragraph::new(format!("{right} ").fg(colour)).right_aligned(), area);
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

/// A mode's name in bold on the accent, as the status bar shows it.
fn mode_block(theme: &Theme, mode: &str) -> Span<'static> {
    format!(" {mode} ").bold().fg(theme.on_accent).bg(theme.accent)
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

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, theme: &Theme, message: &str) {
    let text = vec![
        Line::from(format!(" {message}")).fg(theme.alert),
        Line::from(" press any key to quit").fg(theme.tertiary),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
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

/// Words of `text`, a `due:` date styled against `today` when given, any other word by `tag_style`, with the spaces between them kept.
fn words(theme: &Theme, text: &str, today: Option<Date>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            spans.push(" ".into());
        }
        let due = word.strip_prefix(DUE).and_then(|value| due_style(theme, value, today));
        spans.push(Span::styled(word.to_string(), due.unwrap_or_else(|| tag_style(theme, word))));
    }
    spans
}

/// Style of a due date: red once past `today`, yellow on the day, none when later, not a date, or with no `today`.
fn due_style(theme: &Theme, value: &str, today: Option<Date>) -> Option<Style> {
    let (due, today) = (parse_date(value)?, today?);
    match due.cmp(&today) {
        Ordering::Less => Some(Style::new().fg(theme.alert)),
        Ordering::Equal => Some(Style::new().fg(theme.due_today)),
        Ordering::Greater => None,
    }
}

/// Style of a word of a task: a `+project` mauve, an `@context` teal, a `key:value` greyed, anything else plain.
fn tag_style(theme: &Theme, word: &str) -> Style {
    match word.chars().next() {
        Some('+') if word.len() > 1 => Style::new().fg(theme.project),
        Some('@') if word.len() > 1 => Style::new().fg(theme.context),
        _ if Todo::is_key_value(word) => Style::new().fg(theme.tertiary),
        _ => Style::new(),
    }
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

/// A zone's card: a rounded border, in the accent when the zone gets the keys.
fn card(theme: &Theme, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { theme.accent } else { theme.structure }))
}

/// A floating window's frame: rounded on a raised background, its border in the accent when it gets the keys, its titles bold.
fn floating(theme: &Theme, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { theme.accent } else { theme.separator }))
        .title_style(Style::new().fg(theme.primary).bold())
        .style(Style::new().bg(theme.raised))
}

/// The cursor's mark, in the accent when its zone gets the keys.
fn arrow(theme: &Theme, focused: bool) -> Span<'static> {
    "→ ".fg(if focused { theme.accent } else { theme.tertiary })
}

/// Style of the row under the cursor: highlighted only when its zone gets the keys.
fn highlight(theme: &Theme, focused: bool) -> Style {
    if focused { Style::new().bg(theme.selected) } else { Style::new() }
}

/// Colour of a priority, `A` to `E`.
fn priority(theme: &Theme, letter: char) -> Color {
    theme.priorities[(letter as usize).saturating_sub('A' as usize).min(theme.priorities.len() - 1)]
}

/// A priority as a badge: its letter bold in its colour between two spaces, on a dark background.
fn badge(theme: &Theme, letter: char) -> Span<'static> {
    Span::styled(format!(" {letter} "), Style::new().bold().fg(priority(theme, letter)).bg(theme.badge))
}

#[cfg(test)]
mod tests;
