use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;

use crate::store::Store;
use crate::todo::Todo;
use crate::view;

/// State of the interactive list: the tasks and where the user stands in them.
pub struct App {
    /// The tasks being browsed.
    store: Store,
    /// Position of the selected row among the tasks on screen.
    pub cursor: usize,
    /// First key of a two-key command (`gg`) waiting for its second key.
    pending: Option<char>,
    /// Set once the user asked to leave.
    pub quit: bool,
}

impl App {
    /// Opens the list on its first row.
    pub fn new(store: Store) -> Self {
        App {
            store,
            cursor: 0,
            pending: None,
            quit: false,
        }
    }

    /// Tasks on screen, numbered and in display order.
    pub fn tasks(&self) -> Vec<(usize, &Todo)> {
        self.store.list(false, &[])
    }

    /// Applies one key press to the state.
    pub fn handle_key(&mut self, key: KeyEvent) {
        let last = self.tasks().len().saturating_sub(1);
        let pending = self.pending.take();
        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.quit = true,
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.cursor = (self.cursor + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Char('g') if pending == Some('g') => self.cursor = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => self.cursor = last,
            _ => {}
        }
    }
}

/// Runs the interactive list until the user quits.
pub fn run(store: Store) -> io::Result<()> {
    let mut app = App::new(store);
    let mut scroll = ListState::default();
    ratatui::run(|terminal| {
        while !app.quit {
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key);
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(Store::new(["one", "two", "three"].into_iter().map(Todo::from_line).collect()))
    }

    fn press(app: &mut App, keys: &str) {
        for c in keys.chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
    }

    #[test]
    fn j_and_k_move_the_cursor_without_leaving_the_list() {
        let mut app = app();

        press(&mut app, "k");
        assert_eq!(app.cursor, 0);
        press(&mut app, "jjj");
        assert_eq!(app.cursor, 2);
        app.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn gg_jumps_to_the_top_and_g_is_dropped_when_another_key_follows() {
        let mut app = app();

        press(&mut app, "G");
        assert_eq!(app.cursor, 2);
        press(&mut app, "gkg");
        assert_eq!(app.cursor, 1);
        press(&mut app, "gg");
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn q_and_ctrl_c_quit() {
        let mut by_q = app();
        press(&mut by_q, "q");
        assert!(by_q.quit);

        let mut by_ctrl_c = app();
        by_ctrl_c.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(by_ctrl_c.quit);
    }
}
