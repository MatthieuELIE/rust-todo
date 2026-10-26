use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use super::style::floating;
use crate::theme::Theme;
use crate::tui::App;

/// Width of the help box, its borders included.
const HELP_WIDTH: u16 = 46;

/// Tallest the help box grows, its borders and padding included.
const HELP_MAX_HEIGHT: u16 = 30;

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

/// Draws the key help in one scrolling column in a rounded box, centred in `area`, and tells the app how many lines it shows.
pub(super) fn draw_help(frame: &mut Frame, app: &App, scroll: u16, area: Rect) {
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

#[cfg(test)]
mod tests;
