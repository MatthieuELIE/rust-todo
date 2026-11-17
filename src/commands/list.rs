use std::io::IsTerminal;
use std::process::ExitCode;

use ratatui::backend::IntoCrossterm;
use ratatui::text::Line;
use time::Date;

use crate::cli::Listing;
use crate::filter;
use crate::theme::Theme;
use crate::today;
use crate::todo::Todo;
use crate::view;

/// `text` with each control character replaced, so that a line of the file cannot drive the terminal.
fn clean(text: &str) -> String {
    text.replace(crate::todo::is_control, "\u{fffd}")
}

/// A task as `list` prints it, without its number: styled on a terminal, its plain line otherwise.
pub(super) fn shown(theme: &Theme, todo: &Todo, today: Date) -> String {
    if std::io::stdout().is_terminal() {
        styled(&view::line(theme, 0, todo, today))
    } else {
        clean(&todo.shown_line())
    }
}

/// A listed line, without its number, as text carrying its styles for a terminal.
fn styled(line: &Line) -> String {
    line.iter()
        .skip(1)
        .map(|span| span.style.into_crossterm().apply(clean(&span.content)).to_string())
        .collect()
}

/// Whether `listing` shows `todo`.
fn shows(listing: Listing, todo: &Todo, today: Date) -> bool {
    match listing {
        Listing::Pending => !todo.done && !todo.is_sleeping(today),
        Listing::Due => !todo.done && !todo.is_sleeping(today) && todo.is_due(today),
        Listing::Done => true,
        Listing::Sleeping => !todo.done && todo.is_sleeping(today),
    }
}

/// Prints the tasks `list` shows and returns its exit code.
pub(super) fn list(todos: &[Todo], listing: Listing, terms: &[String], theme: &Theme) -> ExitCode {
    let today = today();
    let mut tasks = filter::list(todos, terms);
    tasks.retain(|(_, todo)| shows(listing, todo, today));

    if listing == Listing::Done {
        tasks.sort_by_key(|(number, _)| *number);
    }

    if tasks.is_empty() {
        let nothing = match listing {
            Listing::Pending => "nothing to do",
            Listing::Due => "nothing due",
            Listing::Done => "nothing done",
            Listing::Sleeping => "nothing sleeping",
        };

        eprintln!("{}", if terms.is_empty() { nothing } else { "no matching task" });
    }

    for (number, todo) in tasks {
        println!("{}{}", view::number(number), shown(theme, todo, today));
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests;
