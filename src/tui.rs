use std::io;
use std::path::Path;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use time::Date;

use crate::editor::{Editor, Mode, Outcome};
use crate::repository;
use crate::store::Store;
use crate::todo::Todo;
use crate::view;

/// What the popup's text becomes once submitted.
pub enum Target {
    /// A new task.
    Add,
    /// A new line for the task with this number.
    Edit(usize),
}

/// The centred field where a task is typed.
pub struct Popup {
    /// The text being typed.
    pub editor: Editor,
    /// What the text is for.
    pub target: Target,
}

/// State of the interactive list: the tasks and where the user stands in them.
pub struct App {
    /// The tasks being browsed.
    store: Store,
    /// Position of the selected row among the tasks on screen.
    pub cursor: usize,
    /// First key of a two-key command (`gg`, `dd`, `p` and a letter) waiting for its second key.
    pending: Option<char>,
    /// Whether done tasks are listed too.
    pub show_done: bool,
    /// Set once the user asked to leave.
    pub quit: bool,
    /// Last message for the status bar, cleared by the next key.
    pub message: Option<String>,
    /// Set while the status bar is the search input line.
    pub searching: bool,
    /// Set while a task is typed in the popup.
    pub popup: Option<Popup>,
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
            searching: false,
            popup: None,
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
        if let Some(popup) = &mut self.popup {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                self.quit = true;
                return false;
            }
            let outcome = popup.editor.handle_key(key);
            if outcome == Outcome::Continue {
                return false;
            }
            let popup = self.popup.take().expect("the popup is open");
            let write = match (outcome, popup.target) {
                (Outcome::Submit, Target::Add) => self.add(&popup.editor.text, today),
                (Outcome::Submit, Target::Edit(number)) => self.edit(number, &popup.editor.text),
                _ => false,
            };
            self.cursor = self.cursor.min(self.tasks().len().saturating_sub(1));
            return write;
        }
        if self.searching {
            match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.quit = true,
                KeyCode::Char(c) => self.search.push(c),
                KeyCode::Backspace => _ = self.search.pop(),
                KeyCode::Esc => {
                    self.search.clear();
                    self.searching = false;
                }
                KeyCode::Enter => self.searching = false,
                _ => {}
            }
            self.cursor = 0;
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
            KeyCode::Char(c @ ('a'..='e' | ' ')) if pending == Some('p') => {
                let priority = (c != ' ').then(|| c.to_ascii_uppercase());
                if let Some(number) = selected
                    && let todo = &mut self.store.todos[number - 1]
                    && !todo.done
                    && todo.priority != priority
                {
                    todo.priority = priority;
                    self.follow(number, "priority set, hidden by the filter");
                    write = true;
                }
            }
            KeyCode::Char('p') => self.pending = Some('p'),
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
            KeyCode::Char('o') => {
                self.popup = Some(Popup {
                    editor: Editor::default(),
                    target: Target::Add,
                })
            }
            KeyCode::Char('/') => {
                self.search.clear();
                self.searching = true;
                self.cursor = 0;
            }
            KeyCode::Esc => {
                self.search.clear();
                self.filter = None;
                self.cursor = 0;
            }
            KeyCode::Enter => {
                if let Some(number) = selected {
                    self.popup = Some(Popup {
                        editor: Editor::new(self.store.todos[number - 1].to_line(), Mode::Normal),
                        target: Target::Edit(number),
                    })
                }
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
                self.follow(self.store.todos.len(), "added, hidden by the filter");
                true
            }
            Err(e) => {
                self.message = Some(e);
                false
            }
        }
    }

    /// Replaces task `number` with the line `text` as typed, with the cursor on it; an empty description only leaves a message,
    /// and an unchanged line changes nothing.
    fn edit(&mut self, number: usize, text: &str) -> bool {
        let todo = Todo::from_line(text);
        if todo.description.trim().is_empty() {
            self.message = Some("a task needs a description".to_string());
            return false;
        }
        if text == self.store.todos[number - 1].to_line() {
            return false;
        }
        self.store.todos[number - 1] = todo;
        self.follow(number, "edited, hidden by the filter");
        true
    }

    /// Puts the cursor on task `number`, or says `hidden` when it is not on screen.
    fn follow(&mut self, number: usize, hidden: &str) {
        match self.tasks().iter().position(|(n, _)| *n == number) {
            Some(row) => self.cursor = row,
            None => self.message = Some(hidden.to_string()),
        }
    }

    /// Replaces the tasks with a fresh read of the file, keeping the cursor on its row.
    /// An edit is cancelled, since its task number may now name another task.
    pub fn reload(&mut self, store: Store) {
        self.store = store;
        self.cursor = self.cursor.min(self.tasks().len().saturating_sub(1));
        self.message = Some("reloaded".to_string());
        if let Some(Popup { target: Target::Edit(_), .. }) = self.popup {
            self.popup = None;
            self.message = Some("reloaded, edit cancelled".to_string());
        }
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
        assert!(app.popup.is_none());
    }

    #[test]
    fn backspace_edits_the_input_and_esc_twice_drops_it() {
        let mut app = app();
        let esc = KeyEvent::from(KeyCode::Esc);

        press(&mut app, "oab");
        app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
        press(&mut app, "c");
        assert_eq!(app.popup.as_ref().unwrap().editor.text, "ac");

        assert!(!app.handle_key(esc, TODAY));
        assert_eq!(app.popup.as_ref().unwrap().editor.mode, Mode::Normal);
        assert!(!app.handle_key(esc, TODAY));
        assert!(app.popup.is_none());
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }

    #[test]
    fn enter_in_normal_mode_adds_the_task_too() {
        let mut app = app();

        press(&mut app, "oCall bank");
        app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
        press(&mut app, "bithe ");
        app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

        assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));
        assert_eq!(app.store.todos[3].to_line(), "2026-09-26 Call the bank");
    }

    #[test]
    fn list_keys_typed_in_the_popup_are_text() {
        let mut app = app();

        assert!(!press(&mut app, "oqxdd"));
        assert!(!app.quit);
        assert_eq!(shown(&app), ["one", "two", "three"]);
        assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

        assert_eq!(app.store.todos[3].to_line(), "2026-09-26 qxdd");
    }

    #[test]
    fn enter_opens_the_task_line_in_normal_mode_and_the_cursor_follows_the_edited_task() {
        let mut app = app_of(&["one", "two", "three"]);
        let enter = KeyEvent::from(KeyCode::Enter);

        assert!(!press(&mut app, "jj"));
        app.handle_key(enter, TODAY);
        let editor = &app.popup.as_ref().unwrap().editor;
        assert_eq!((editor.text.as_str(), editor.cursor, editor.mode), ("three", 0, Mode::Normal));

        press(&mut app, "i(A) ");
        assert!(app.handle_key(enter, TODAY));

        assert_eq!(shown(&app), ["(A) three", "one", "two"]);
        assert_eq!(app.cursor, 0);
        assert!(app.popup.is_none());
    }

    #[test]
    fn an_edited_line_is_saved_as_typed_and_one_leaving_the_list_is_said_hidden() {
        let mut app = app_of(&["one", "two"]);
        let enter = KeyEvent::from(KeyCode::Enter);

        press(&mut app, "j");
        app.handle_key(enter, TODAY);
        press(&mut app, "ix 2026-09-20 ");
        assert!(app.handle_key(enter, TODAY));

        assert_eq!(app.store.todos[1].to_line(), "x 2026-09-20 two");
        assert_eq!(app.cursor, 0);
        assert_eq!(app.message.as_deref(), Some("edited, hidden by the filter"));
    }

    #[test]
    fn an_edit_hidden_by_the_search_leaves_the_cursor_where_it_was() {
        let mut app = app_of(&["one", "two", "three"]);
        let enter = KeyEvent::from(KeyCode::Enter);

        press(&mut app, "/t");
        app.handle_key(enter, TODAY);
        press(&mut app, "j");
        app.handle_key(enter, TODAY);
        press(&mut app, "Cfree");
        assert!(app.handle_key(enter, TODAY));

        assert_eq!(shown(&app), ["two"]);
        assert_eq!(app.cursor, 0);
        assert_eq!(app.store.todos[2].to_line(), "free");
        assert_eq!(app.message.as_deref(), Some("edited, hidden by the filter"));
    }

    #[test]
    fn an_edit_emptied_or_left_unchanged_writes_nothing() {
        let mut app = app_of(&["(A) one"]);
        let enter = KeyEvent::from(KeyCode::Enter);

        app.handle_key(enter, TODAY);
        assert!(!app.handle_key(enter, TODAY));
        assert_eq!(app.message, None);

        app.handle_key(enter, TODAY);
        press(&mut app, "WD");
        assert_eq!(app.popup.as_ref().unwrap().editor.text, "(A) ");
        assert!(!app.handle_key(enter, TODAY));
        assert_eq!(app.message.as_deref(), Some("a task needs a description"));
        assert!(app.popup.is_none());
        assert_eq!(shown(&app), ["(A) one"]);
    }

    #[test]
    fn enter_on_an_empty_list_opens_nothing() {
        let mut app = app_of(&[]);

        app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

        assert!(app.popup.is_none());
    }

    #[test]
    fn a_reload_cancels_an_edit_but_keeps_an_add_being_typed() {
        let mut app = app();
        let reread = || Store::new(vec![Todo::from_line("one"), Todo::from_line("two")]);

        app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
        app.reload(reread());
        assert!(app.popup.is_none());
        assert_eq!(app.message.as_deref(), Some("reloaded, edit cancelled"));

        press(&mut app, "onew");
        app.reload(reread());
        assert_eq!(app.popup.as_ref().unwrap().editor.text, "new");
        assert_eq!(app.message.as_deref(), Some("reloaded"));
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
    fn p_then_a_letter_sets_the_priority_and_the_cursor_follows_the_task() {
        let mut app = app_of(&["one", "(A) two", "three"]);

        assert!(press(&mut app, "jjpb"));
        assert_eq!(shown(&app), ["(A) two", "(B) three", "one"]);
        assert_eq!(app.cursor, 1);

        assert!(press(&mut app, "kp "));
        assert_eq!(shown(&app), ["(B) three", "one", "two"]);
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn p_does_nothing_with_another_key_the_same_priority_or_a_done_task() {
        let mut app = app_of(&["(A) one", "x 2026-09-20 two"]);

        assert!(!press(&mut app, "pz"));
        assert!(!press(&mut app, "pa"));
        assert!(!press(&mut app, "Hjpb"));

        assert_eq!(shown(&app), ["(A) one", "x 2026-09-20 two"]);
    }

    #[test]
    fn x_and_dd_do_nothing_on_an_empty_list() {
        let mut app = app_of(&[]);

        assert!(!press(&mut app, "xddpa"));
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
