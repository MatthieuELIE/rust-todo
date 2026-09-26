use std::io;
use std::path::Path;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use time::Date;

use crate::repository;
use crate::store::Store;
use crate::todo::Todo;
use crate::view;

/// What the status bar input line is typing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Prompt {
    /// A task to add.
    Add,
    /// The search, applied at each letter.
    Search,
}

/// State of the interactive list: the tasks and where the user stands in them.
pub struct App {
    /// The tasks being browsed.
    store: Store,
    /// Position of the selected row among the tasks on screen.
    pub cursor: usize,
    /// First key of a two-key command (`gg`, `dd`) waiting for its second key.
    pending: Option<char>,
    /// Whether done tasks are listed too.
    pub show_done: bool,
    /// Set once the user asked to leave.
    pub quit: bool,
    /// Last message for the status bar, cleared by the next key.
    pub message: Option<String>,
    /// Set while the status bar is an input line.
    pub prompt: Option<Prompt>,
    /// Text of the task being typed.
    pub input: String,
    /// Search terms, separated by whitespace, as `todo list` takes them.
    pub search: String,
    /// Term picked in the panel, `None` for `all`.
    pub filter: Option<String>,
    /// Set while keys go to the panel rather than the list.
    pub panel: bool,
    /// Set while the key help is shown over the list.
    pub help: bool,
}

impl App {
    /// Opens the list on its first row.
    pub fn new(store: Store) -> Self {
        App {
            store,
            cursor: 0,
            pending: None,
            show_done: false,
            quit: false,
            message: None,
            prompt: None,
            input: String::new(),
            search: String::new(),
            filter: None,
            panel: false,
            help: false,
        }
    }

    /// Tasks on screen, numbered and in display order.
    pub fn tasks(&self) -> Vec<(usize, &Todo)> {
        let mut terms: Vec<String> = self.search.split_whitespace().map(String::from).collect();
        terms.extend(self.filter.clone());
        self.store.list(self.show_done, &terms)
    }

    /// Panel entries with how many tasks each shows, the search left out: `all`, then the `+projects` and `@contexts` of the
    /// tasks shown, each alphabetical.
    pub fn filters(&self) -> Vec<(String, usize)> {
        let shown = self.store.list(self.show_done, &[]);
        let terms = |sigil: char, names: fn(&Todo) -> Vec<&str>| {
            let mut terms: Vec<String> = shown
                .iter()
                .flat_map(|(_, todo)| names(todo))
                .map(|name| format!("{sigil}{name}"))
                .collect();
            terms.sort_by_key(|term| (term.to_lowercase(), term.clone()));
            terms.dedup();
            terms
        };
        let mut filters = vec![("all".to_string(), shown.len())];
        for term in terms('+', Todo::projects).into_iter().chain(terms('@', Todo::contexts)) {
            let count = self.store.list(self.show_done, std::slice::from_ref(&term)).len();
            filters.push((term, count));
        }
        filters
    }

    /// Row of the active filter among the panel `filters`, `all` when it is not among them.
    pub fn filter_row(&self, filters: &[(String, usize)]) -> usize {
        filters.iter().position(|(term, _)| Some(term) == self.filter.as_ref()).unwrap_or(0)
    }

    /// Applies one key press to the state, dated `today`, and tells whether the file must be rewritten.
    pub fn handle_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let tasks = self.tasks();
        let last = tasks.len().saturating_sub(1);
        let selected = tasks.get(self.cursor).map(|(number, _)| *number);
        let pending = self.pending.take();
        self.message = None;
        if self.help {
            self.help = false;
            return false;
        }
        if let Some(prompt) = self.prompt {
            let text = match prompt {
                Prompt::Add => &mut self.input,
                Prompt::Search => &mut self.search,
            };
            match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.quit = true,
                KeyCode::Char(c) => text.push(c),
                KeyCode::Backspace => _ = text.pop(),
                KeyCode::Esc => {
                    text.clear();
                    self.prompt = None;
                }
                KeyCode::Enter if prompt == Prompt::Add => {
                    let text = std::mem::take(text);
                    self.prompt = None;
                    return self.add(&text, today);
                }
                KeyCode::Enter => self.prompt = None,
                _ => {}
            }
            if prompt == Prompt::Search {
                self.cursor = 0;
            }
            return false;
        }
        if self.panel {
            let filters = self.filters();
            let row = self.filter_row(&filters);
            let pick = |row: usize| (row > 0).then(|| filters[row].0.clone());
            let before = self.filter.clone();
            match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.quit = true,
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.filter = pick((row + 1).min(filters.len() - 1)),
                KeyCode::Char('k') | KeyCode::Up => self.filter = pick(row.saturating_sub(1)),
                KeyCode::Esc => self.filter = None,
                KeyCode::Tab | KeyCode::Enter => self.panel = false,
                _ => {}
            }
            if self.filter != before {
                self.cursor = 0;
            }
            return false;
        }
        let mut write = false;
        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.quit = true,
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.cursor = (self.cursor + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Char('g') if pending == Some('g') => self.cursor = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => self.cursor = last,
            KeyCode::Char('x') => {
                if let Some(number) = selected {
                    let todo = &mut self.store.todos[number - 1];
                    if todo.done {
                        todo.reopen()
                    } else {
                        todo.complete(today)
                    }
                    write = true;
                }
            }
            KeyCode::Char('d') if pending == Some('d') => write = selected.is_some_and(|number| self.store.remove(number)),
            KeyCode::Char('d') => self.pending = Some('d'),
            KeyCode::Char('o') => self.prompt = Some(Prompt::Add),
            KeyCode::Char('/') => {
                self.search.clear();
                self.prompt = Some(Prompt::Search);
                self.cursor = 0;
            }
            KeyCode::Esc => {
                self.search.clear();
                self.filter = None;
                self.cursor = 0;
            }
            KeyCode::Tab => self.panel = true,
            KeyCode::Char('?') => self.help = true,
            KeyCode::Char('H') => {
                self.show_done = !self.show_done;
                self.cursor = 0;
            }
            _ => {}
        }
        self.cursor = self.cursor.min(self.tasks().len().saturating_sub(1));
        write
    }

    /// Adds the task typed as `text`, dated `today`, with the cursor on it; a rejected text only leaves a message.
    /// Under a panel filter the term is appended when the task would not match it.
    fn add(&mut self, text: &str, today: Date) -> bool {
        match Todo::new_from_input(text, today) {
            Ok(mut todo) => {
                if let Some(term) = &self.filter
                    && !todo.to_line().to_lowercase().contains(&term.to_lowercase())
                {
                    todo.description = format!("{} {term}", todo.description);
                }
                self.store.add(todo);
                let number = self.store.todos.len();
                match self.tasks().iter().position(|(n, _)| *n == number) {
                    Some(row) => self.cursor = row,
                    None => self.message = Some("added, hidden by the filter".to_string()),
                }
                true
            }
            Err(e) => {
                self.message = Some(e);
                false
            }
        }
    }

    /// Replaces the tasks with a fresh read of the file, keeping the cursor on its row.
    pub fn reload(&mut self, store: Store) {
        self.store = store;
        self.cursor = self.cursor.min(self.tasks().len().saturating_sub(1));
        self.message = Some("reloaded".to_string());
    }
}

/// Runs the interactive list until the user quits, saving to `path` after every change.
/// `text` is the file as last read or written: when the file no longer matches it, the list is reloaded and the key ignored.
pub fn run(store: Store, mut text: String, path: &Path) -> io::Result<()> {
    let mut app = App::new(store);
    let mut scroll = ListState::default();
    ratatui::run(|terminal| {
        while !app.quit {
            terminal.draw(|frame| view::draw(frame, &app, &mut scroll))?;
            let event = if event::poll(Duration::from_millis(250))? {
                Some(event::read()?)
            } else {
                None
            };
            if let Ok((current, todos)) = repository::load(path)
                && current != text
            {
                text = current;
                app.reload(Store::new(todos));
            } else if let Some(Event::Key(key)) = event
                && key.kind == KeyEventKind::Press
                && app.handle_key(key, crate::today())
            {
                match repository::save(path, &app.store.todos) {
                    Ok(written) => text = written,
                    Err(e) => {
                        if let Ok((_, todos)) = repository::load(path) {
                            app.reload(Store::new(todos));
                        }
                        app.message = Some(format!("could not save: {e} (file left unchanged)"));
                    }
                }
            }
        }
        Ok(())
    })
}

/// Shows `message` full screen until a key is pressed, so a popup that closes when the program exits does not swallow it.
pub fn show_error(message: &str) -> io::Result<()> {
    ratatui::run(|terminal| {
        terminal.draw(|frame| view::draw_error(frame, message))?;
        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                return Ok(());
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 09 - 26);

    fn app_of(lines: &[&str]) -> App {
        App::new(Store::new(lines.iter().map(|l| Todo::from_line(l)).collect()))
    }

    fn app() -> App {
        app_of(&["one", "two", "three"])
    }

    fn press(app: &mut App, keys: &str) -> bool {
        keys.chars()
            .fold(false, |write, c| app.handle_key(KeyEvent::from(KeyCode::Char(c)), TODAY) | write)
    }

    fn shown(app: &App) -> Vec<String> {
        app.tasks().into_iter().map(|(_, todo)| todo.to_line()).collect()
    }

    #[test]
    fn x_completes_the_selected_task_and_the_next_one_takes_its_row() {
        let mut app = app();

        assert!(press(&mut app, "jx"));

        assert_eq!(shown(&app), ["one", "three"]);
        assert_eq!(app.cursor, 1);
        assert_eq!(app.store.todos[1].to_line(), "x 2026-09-26 two");
    }

    #[test]
    fn o_types_a_task_that_enter_adds_with_the_cursor_on_it() {
        let mut app = app();

        assert!(!press(&mut app, "jjo(A) Ask for a quote"));
        assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

        assert_eq!(shown(&app), ["(A) 2026-09-26 Ask for a quote", "one", "two", "three"]);
        assert_eq!(app.cursor, 0);
        assert!(!app.quit);
        assert_eq!(app.prompt, None);
    }

    #[test]
    fn backspace_edits_the_input_and_esc_drops_it() {
        let mut app = app();

        press(&mut app, "oab");
        app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
        press(&mut app, "c");
        assert_eq!(app.input, "ac");

        app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
        assert_eq!(app.prompt, None);
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }

    #[test]
    fn a_rejected_input_adds_nothing_and_says_why() {
        let mut app = app();

        press(&mut app, "o");
        assert!(!app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

        assert_eq!(app.message.as_deref(), Some("a task needs a description"));
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }

    #[test]
    fn slash_filters_at_each_letter_from_the_top_and_enter_keeps_the_search() {
        let mut app = app_of(&["Call the bank", "Pay rent", "Call mom"]);

        press(&mut app, "jj/call");
        assert_eq!(shown(&app), ["Call the bank", "Call mom"]);
        assert_eq!(app.cursor, 0);

        press(&mut app, " -mom");
        app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
        press(&mut app, "j");
        assert_eq!(shown(&app), ["Call the bank"]);
    }

    #[test]
    fn esc_drops_the_search_being_typed_or_the_one_kept_in_the_list() {
        let mut app = app();
        let esc = KeyEvent::from(KeyCode::Esc);

        press(&mut app, "/one");
        app.handle_key(esc, TODAY);
        assert_eq!(shown(&app), ["one", "two", "three"]);

        press(&mut app, "/one");
        app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
        app.handle_key(esc, TODAY);
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }

    #[test]
    fn a_task_added_under_a_search_it_does_not_match_is_saved_but_said_hidden() {
        let mut app = app();
        let enter = KeyEvent::from(KeyCode::Enter);

        press(&mut app, "/t");
        app.handle_key(enter, TODAY);
        press(&mut app, "jofour");
        assert!(app.handle_key(enter, TODAY));

        assert_eq!(shown(&app), ["two", "three"]);
        assert_eq!(app.cursor, 1);
        assert_eq!(app.message.as_deref(), Some("added, hidden by the filter"));
    }

    #[test]
    fn the_panel_lists_all_then_projects_then_contexts_alphabetically_with_their_shown_count() {
        let mut app = app_of(&["Pay +rent @home", "Call +bank @phone +rent", "x Old +archive", "Read +Books"]);

        let expected = [("all", 3), ("+bank", 1), ("+Books", 1), ("+rent", 2), ("@home", 1), ("@phone", 1)];
        assert_eq!(app.filters(), expected.map(|(name, count)| (name.to_string(), count)));

        press(&mut app, "H/call");
        assert_eq!(app.filters()[..2], [("all".to_string(), 4), ("+archive".to_string(), 1)]);
    }

    #[test]
    fn in_the_panel_j_and_k_filter_the_list_from_the_top_and_tab_goes_back_keeping_the_filter() {
        let mut app = app_of(&["Pay +rent", "Call +bank", "Buy milk +rent"]);
        let tab = KeyEvent::from(KeyCode::Tab);

        press(&mut app, "j");
        app.handle_key(tab, TODAY);
        press(&mut app, "jj");
        assert_eq!(app.filter.as_deref(), Some("+rent"));
        assert_eq!(shown(&app), ["Pay +rent", "Buy milk +rent"]);
        assert_eq!(app.cursor, 0);

        app.handle_key(tab, TODAY);
        press(&mut app, "j");
        assert_eq!(app.cursor, 1);
        app.handle_key(tab, TODAY);
        press(&mut app, "k");
        assert_eq!(app.filter.as_deref(), Some("+bank"));
    }

    #[test]
    fn esc_in_the_panel_goes_back_to_all_and_esc_in_the_list_drops_filter_and_search() {
        let mut app = app_of(&["Pay +rent", "Call +bank"]);
        let (tab, esc) = (KeyEvent::from(KeyCode::Tab), KeyEvent::from(KeyCode::Esc));

        app.handle_key(tab, TODAY);
        press(&mut app, "j");
        app.handle_key(esc, TODAY);
        assert_eq!(app.filter, None);
        assert!(app.panel);

        press(&mut app, "j");
        app.handle_key(tab, TODAY);
        press(&mut app, "/pay");
        app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
        app.handle_key(esc, TODAY);
        assert_eq!(shown(&app), ["Pay +rent", "Call +bank"]);
    }

    #[test]
    fn a_task_added_under_a_panel_filter_gets_its_term_unless_it_already_matches() {
        let mut app = app_of(&["Pay +rent"]);
        let enter = KeyEvent::from(KeyCode::Enter);
        app.filter = Some("+rent".to_string());

        press(&mut app, "o(A) Call landlord");
        app.handle_key(enter, TODAY);
        press(&mut app, "oFix +Rent form");
        app.handle_key(enter, TODAY);
        press(&mut app, "o");
        app.handle_key(enter, TODAY);

        let lines: Vec<String> = app.store.todos.iter().map(Todo::to_line).collect();
        assert_eq!(lines, ["Pay +rent", "(A) 2026-09-26 Call landlord +rent", "2026-09-26 Fix +Rent form"]);
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn question_mark_opens_the_help_and_the_next_key_only_closes_it() {
        let mut app = app();

        press(&mut app, "?");
        assert!(app.help);

        assert!(!press(&mut app, "x"));
        assert!(!app.help);
        assert_eq!(shown(&app), ["one", "two", "three"]);

        press(&mut app, "?q");
        assert!(!app.quit);
    }

    #[test]
    fn a_reload_keeps_the_cursor_row_and_says_so_until_the_next_key() {
        let mut app = app();
        press(&mut app, "G");

        app.reload(Store::new(vec![Todo::from_line("one"), Todo::from_line("four")]));

        assert_eq!(shown(&app), ["one", "four"]);
        assert_eq!(app.cursor, 1);
        assert_eq!(app.message.as_deref(), Some("reloaded"));
        press(&mut app, "k");
        assert_eq!(app.message, None);
    }

    #[test]
    fn h_shows_done_tasks_from_the_top_where_x_reopens_them() {
        let mut app = app_of(&["x 2026-09-20 one", "two"]);

        assert!(!press(&mut app, "H"));
        assert_eq!(shown(&app), ["two", "x 2026-09-20 one"]);
        assert!(press(&mut app, "jx"));
        assert_eq!(shown(&app), ["one", "two"]);

        assert_eq!(app.cursor, 1);
        press(&mut app, "H");
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn dd_removes_the_selected_task_and_a_d_followed_by_another_key_does_nothing() {
        let mut app = app();

        assert!(!press(&mut app, "djd"));
        assert_eq!(app.cursor, 1);
        assert!(press(&mut app, "jdd"));

        assert_eq!(shown(&app), ["one", "two"]);
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn x_and_dd_do_nothing_on_an_empty_list() {
        let mut app = app_of(&[]);

        assert!(!press(&mut app, "xdd"));
    }

    #[test]
    fn j_and_k_move_the_cursor_without_leaving_the_list() {
        let mut app = app();

        press(&mut app, "k");
        assert_eq!(app.cursor, 0);
        press(&mut app, "jjj");
        assert_eq!(app.cursor, 2);
        app.handle_key(KeyEvent::from(KeyCode::Up), TODAY);
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
        by_ctrl_c.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), TODAY);
        assert!(by_ctrl_c.quit);
    }
}
