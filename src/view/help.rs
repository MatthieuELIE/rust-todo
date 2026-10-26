use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use super::style::{floating, mode_block};
use crate::editor::Mode;
use crate::theme::Theme;
use crate::tui::{App, Focus};

/// Width of a column of keys: the longest line of `HELP`.
const COLUMN_WIDTH: u16 = 40;

/// Columns left between the two columns.
const COLUMN_GAP: u16 = 4;

/// Borders and padding on both sides of the box.
const BOX_MARGIN: u16 = 8;

/// Narrowest area that holds the two columns.
const TWO_COLUMNS_WIDTH: u16 = 2 * COLUMN_WIDTH + COLUMN_GAP + BOX_MARGIN;

/// Sections `LIST`, `PANEL` and `SEARCH` in the left column.
const LEFT_SECTIONS: usize = 3;

/// Tallest the help box grows, its borders and padding included.
const HELP_MAX_HEIGHT: u16 = 36;

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
?            help; PANEL NORMAL DATE too
q            quit

PANEL
j/k          pick a filter
Esc          all tasks
Tab/Enter    back to the list

SEARCH
Enter        keep the search
Esc          clear the search

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
Enter        save
Esc          cancel

DATE
h/l/←/→      day
k/j/↑/↓      week
H/L          month
Enter/Esc    pick, close";

/// Draws the key help in a rounded box, centred in `area`: two columns of sections when the width allows, else one, scrolling when the height falls short, and tells the app how far it can scroll.
pub(super) fn draw_help(frame: &mut Frame, app: &App, scroll: u16, area: Rect) {
    let theme = &app.theme;
    let lit = opened_from(&app.under_help);
    let sections = sections();
    let columns: Vec<Vec<Line<'static>>> = if area.width >= TWO_COLUMNS_WIDTH {
        let (left, right) = sections.split_at(LEFT_SECTIONS);
        vec![column_lines(theme, left, lit), column_lines(theme, right, lit)]
    } else {
        vec![column_lines(theme, &sections, lit)]
    };
    let lines = columns.iter().map(Vec::len).max().unwrap_or_default() as u16;
    let width = columns.len() as u16 * COLUMN_WIDTH + (columns.len() as u16 - 1) * COLUMN_GAP + BOX_MARGIN;
    let height = (lines + 4).min(HELP_MAX_HEIGHT).min(area.height);
    let area = area.centered(Constraint::Length(width), Constraint::Length(height));
    let max = lines.saturating_sub(height.saturating_sub(4));
    let hint = if max > 0 { " j/k scroll · esc close " } else { " esc close " };
    let block = floating(theme, true)
        .title(" HELP ")
        .title_bottom(Line::from(hint.fg(theme.tertiary)).right_aligned())
        .padding(Padding::new(3, 3, 1, 1));
    let inner = block.inner(area);
    app.help_max.set(max);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let slots = Layout::horizontal(vec![Constraint::Length(COLUMN_WIDTH); columns.len()])
        .spacing(COLUMN_GAP)
        .split(inner);
    for (column, slot) in columns.into_iter().zip(slots.iter()) {
        frame.render_widget(Paragraph::new(column).scroll((scroll.min(max), 0)), *slot);
    }
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

/// The name of the mode the status bar shows for `focus`, which the help lights.
fn opened_from(focus: &Focus) -> &'static str {
    match focus {
        Focus::Search => "SEARCH",
        Focus::Panel => "PANEL",
        Focus::Popup(popup) if popup.picker.is_some() => "DATE",
        Focus::Popup(popup) if popup.editor.mode == Mode::Insert => "INSERT",
        Focus::Popup(_) => "NORMAL",
        Focus::List | Focus::Help(_) => "LIST",
    }
}

/// `HELP` cut into its sections, in order, each a name line then its key lines.
fn sections() -> Vec<&'static str> {
    HELP.split("\n\n").collect()
}

/// The lines of `sections` one under the other, a blank line between two sections, the one named `lit` lit.
fn column_lines(theme: &Theme, sections: &[&'static str], lit: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, section) in sections.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        let (name, keys) = section.split_once('\n').unwrap_or((section, ""));
        lines.push(Line::from(mode_block(theme, name, name == lit)));
        lines.extend(help_lines(theme, keys));
    }
    lines
}

/// Key lines of the help: keys bold, the `/` between alternatives and actions greyed.
fn help_lines(theme: &Theme, text: &'static str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| {
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

#[cfg(test)]
mod tests;
