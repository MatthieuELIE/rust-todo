use std::io::IsTerminal;

use ratatui::backend::IntoCrossterm;
use ratatui::text::Line;
use time::Date;

use crate::theme::Theme;
use crate::todo::Todo;
use crate::view;

/// `text` with each control character replaced, so that a line of the file cannot drive the terminal.
fn clean(text: &str) -> String {
    text.replace(char::is_control, "\u{fffd}")
}

/// A task as `list` prints it, without its number: styled on a terminal, its plain line otherwise.
pub fn shown(theme: &Theme, todo: &Todo, today: Date) -> String {
    if std::io::stdout().is_terminal() {
        styled(&view::line(theme, 0, todo, today))
    } else {
        clean(&todo.to_line())
    }
}

/// A listed line, without its number, as text carrying its styles for a terminal.
fn styled(line: &Line) -> String {
    line.iter()
        .skip(1)
        .map(|span| span.style.into_crossterm().apply(clean(&span.content)).to_string())
        .collect()
}

#[cfg(test)]
mod tests;
