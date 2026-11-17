use std::collections::HashMap;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::widgets::ListState;

use crate::theme::Theme;
use crate::todo::Todo;
use crate::tui::{App, Opening};
use crate::{repository, today, view};

/// Runs the list until the user quits, saving to `path` on each change, reloading when the file no longer matches `text`, and opening a key:value with `programs`.
pub fn run(todos: Vec<Todo>, mut text: String, path: &Path, theme: Theme, icons: bool, programs: HashMap<String, PathBuf>) -> io::Result<()> {
    let mut app = App::new(todos);
    app.theme = theme;
    app.icons = icons;
    app.programs = programs;
    app.today = Some(today());
    let mut scroll = ListState::default();

    on_terminal(|terminal| {
        let looped = (|| {
            execute!(io::stdout(), EnableBracketedPaste)?;

            let mut last = None;
            let mut draw = true;

            while !app.quit {
                if draw {
                    app.refresh_history(path);
                    terminal.draw(|frame| view::draw(frame, &app, &mut scroll, today()))?;
                    execute!(io::stdout(), app.cursor_shape())?;
                }

                let event = if event::poll(Duration::from_millis(250))? {
                    Some(event::read()?)
                } else {
                    None
                };

                let toast_shown = app.toast.is_some();
                app.tick(Instant::now());

                let now = idle::stamp_of(path);
                let reread = idle::must_read(&last, &now);
                let read = reread.then(|| {
                    now.as_ref()
                        .map_err(|e| io::Error::new(e.kind(), e.to_string()))
                        .and_then(|_| repository::read(path))
                });
                last = idle::remembered(now, read.as_ref().is_some_and(|read| read.is_err()));
                let day = today();

                draw = idle::must_draw(event.is_some(), reread, toast_shown && app.toast.is_none(), app.today != Some(day));

                if draw && app.turn(read.unwrap_or_else(|| Ok(text.clone())), &mut text, event, day) {
                    app.save(path, &mut text);
                }

                if let Some(clip) = app.copy.take() {
                    app.copied(&clip, pbcopy(clip.text()));
                }

                if let Some(opening) = app.open.take() {
                    app.opened(&opening, open(&opening));
                }
            }

            Ok(())
        })();

        reset_after(&mut io::stdout(), looped)
    })
}

/// Puts `line` in the clipboard through `pbcopy`, with a UTF-8 locale so accents arrive whole.
fn pbcopy(line: &str) -> io::Result<()> {
    let mut child = Command::new("pbcopy").env("LC_CTYPE", "UTF-8").stdin(Stdio::piped()).spawn()?;
    let written = child.stdin.take().expect("stdin is piped").write_all(line.as_bytes());
    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(status.to_string()));
    }

    written
}

/// Starts the program of `opening` on its value, without a shell, its standard streams cut, in a session of its own so it outlives a popup that closes, and does not wait for it.
fn open(opening: &Opening) -> io::Result<()> {
    let mut command = Command::new(&opening.program);
    command
        .arg(&opening.value)
        .env("TODO_PROJECTS", &opening.projects)
        .env("TODO_CONTEXTS", &opening.contexts)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: setsid is async-signal-safe, so it may run between fork and exec.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    command.spawn().map(drop)
}

/// Runs `f` on the terminal set up for the full screen and puts it back, with an error when it cannot be set up.
fn on_terminal(f: impl FnOnce(&mut DefaultTerminal) -> io::Result<()>) -> io::Result<()> {
    let mut terminal = ratatui::try_init().map_err(|e| io::Error::other(format!("could not set up the terminal: {e}")))?;
    let result = f(&mut terminal);
    ratatui::restore();
    result
}

/// Writes the sequences that undo the list's terminal modes to `out`, whatever `looped` holds, and gives back `looped`'s error before the write's.
fn reset_after(out: &mut impl Write, looped: io::Result<()>) -> io::Result<()> {
    let reset = execute!(out, DisableBracketedPaste, SetCursorStyle::DefaultUserShape);

    looped.and(reset)
}

/// Shows `message` full screen until a key is pressed, so a popup that closes when the program exits does not swallow it.
pub fn show_error(message: &str, theme: &Theme) -> io::Result<()> {
    on_terminal(|terminal| {
        terminal.draw(|frame| view::draw_error(frame, theme, message))?;

        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                return Ok(());
            }
        }
    })
}

mod idle;

#[cfg(test)]
mod tests;
