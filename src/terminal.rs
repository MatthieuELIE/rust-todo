use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::widgets::ListState;

use crate::theme::Theme;
use crate::todo::Todo;
use crate::tui::App;
use crate::{repository, today, view};

/// Runs the list until the user quits, saving to `path` on each change and reloading when the file no longer matches `text`.
pub fn run(todos: Vec<Todo>, mut text: String, path: &Path, theme: Theme, icons: bool) -> io::Result<()> {
    let mut app = App::new(todos);
    app.theme = theme;
    app.icons = icons;
    app.today = Some(today());
    let mut scroll = ListState::default();
    on_terminal(|terminal| {
        execute!(io::stdout(), EnableBracketedPaste)?;
        while !app.quit {
            app.refresh_history(path);
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll, today()))?;
            execute!(io::stdout(), app.cursor_shape())?;
            let event = if event::poll(Duration::from_millis(250))? {
                Some(event::read()?)
            } else {
                None
            };
            app.tick(Instant::now());
            if app.turn(repository::read(path), &mut text, event, today()) {
                app.save(path, &mut text);
            }
            if let Some(line) = app.copy.take() {
                app.copied(pbcopy(&line));
            }
        }
        execute!(io::stdout(), DisableBracketedPaste, SetCursorStyle::DefaultUserShape)
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

/// Runs `f` on the terminal set up for the full screen and puts it back, with an error when it cannot be set up.
fn on_terminal(f: impl FnOnce(&mut DefaultTerminal) -> io::Result<()>) -> io::Result<()> {
    let mut terminal = ratatui::try_init().map_err(|e| io::Error::other(format!("could not set up the terminal: {e}")))?;
    let result = f(&mut terminal);
    ratatui::restore();
    result
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
