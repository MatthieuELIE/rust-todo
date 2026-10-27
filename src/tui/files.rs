use super::*;

impl App {
    /// Says how copying the line `yy` asked for went.
    pub fn copied(&mut self, copied: io::Result<()>) {
        match copied {
            Ok(()) => self.message = Some("copied".to_string()),
            Err(e) => self.refuse(&format!("could not copy: {e}")),
        }
    }

    /// One turn of the loop: reloads when the file, `read` just now, no longer matches `text`, else applies `event`; tells whether to save.
    pub fn turn(&mut self, read: io::Result<String>, text: &mut String, event: Option<Event>, today: Date) -> bool {
        self.today = Some(today);
        match (read, event) {
            (Ok(current), _) if current != *text => {
                self.reload(repository::parse(&current));
                *text = current;
            }
            (read, Some(Event::Key(key))) if key.kind == KeyEventKind::Press && self.handle_key(key, today) => match read {
                Ok(_) => return true,
                Err(e) => self.refuse(&format!("could not read the file: {e} (file left unchanged)")),
            },
            (_, Some(Event::Paste(pasted))) => self.paste(&pasted),
            _ => {}
        }
        false
    }

    /// The save `turn` asked for, destination first: the tasks completed go to the end of `done.txt`, the list is saved to `path`, then the tasks `u` or `r` brought back leave `done.txt`.
    pub fn save(&mut self, path: &Path, text: &mut String) {
        let done_path = path.with_file_name("done.txt");
        let (mut gone, mut back) = (Vec::new(), Vec::new());
        for moved in std::mem::take(&mut self.moves) {
            match moved {
                Move::Append(todo) => gone.push(todo),
                Move::Remove(todo, number) => back.push((todo, number)),
            }
        }
        self.history_read &= gone.is_empty() && back.is_empty();
        let appended = if gone.is_empty() && back.is_empty() {
            Ok(())
        } else if repository::same_file(path, &done_path) {
            Err(io::Error::other("the task file is done.txt itself"))
        } else {
            let writable = |path: &Path| std::fs::OpenOptions::new().write(true).open(path).map(drop);
            let reopens = back.iter().any(|(_, number)| number.is_some());
            let checked = if reopens { writable(&done_path) } else { Ok(()) };
            if gone.is_empty() {
                checked
            } else {
                checked.and_then(|()| writable(path)).and_then(|()| repository::append(&done_path, &gone))
            }
        };
        let moved = !gone.is_empty() && appended.is_ok();
        let written = appended.and_then(|()| repository::save(path, &self.todos));
        let failed = written.as_ref().err().map(|e| e.to_string());
        let saved = failed.is_none();
        self.saved(written, text, repository::read(path));
        if let Some(e) = failed
            && moved
        {
            self.refuse(&format!("could not save: {e} (done.txt already holds the task)"));
        }
        let mut held = false;
        for (todo, number) in back.iter().filter(|_| saved) {
            let verb = if number.is_some() { "reopened" } else { "undone" };
            match repository::remove_task(&done_path, *number, &todo.to_line()) {
                Ok(true) => {}
                Ok(false) if !held => self.message = Some(format!("{verb}, the task was no longer in done.txt")),
                Ok(false) => {}
                Err(e) => {
                    self.undo.clear();
                    self.redo.clear();
                    if !held {
                        self.refuse(&format!("{verb}, but done.txt still holds the task: {e}"));
                    }
                    held = true;
                }
            }
        }
    }

    /// What follows the save `turn` asked for: `text` takes what was `written`, or the tasks go back to the file, `reread` just now, and the failure is told.
    pub fn saved(&mut self, written: io::Result<String>, text: &mut String, reread: io::Result<String>) {
        match written {
            Ok(written) => *text = written,
            Err(e) => {
                if let Ok(current) = reread {
                    self.reload(repository::parse(&current));
                }
                self.refuse(&format!("could not save: {e} (file left unchanged)"));
            }
        }
    }

    /// Reads `done.txt`, next to the task file at `path`, when it was not read since the list opened, the last `H`, the last reload or the last save that moved a task.
    pub fn refresh_history(&mut self, path: &Path) {
        if !std::mem::replace(&mut self.history_read, true) {
            self.set_history(repository::read(&path.with_file_name("done.txt")));
        }
    }

    /// Takes the tasks of `done.txt`, `read` just now, as the history; a failed read empties it and is told, unless a refusal is already shown.
    pub fn set_history(&mut self, read: io::Result<String>) {
        self.history = repository::parse(&read.unwrap_or_else(|e| {
            if !self.refused {
                self.refuse(&format!("could not read done.txt: {e}"));
            }
            String::new()
        }));
    }

    /// Replaces the tasks with a fresh read, cursor on its row, history dropped; an edit and a command under way are cancelled as their task may shift.
    pub fn reload(&mut self, todos: Vec<Todo>) {
        self.todos = todos;
        self.pending = None;
        self.undo.clear();
        self.redo.clear();
        self.moves.clear();
        self.history_read = false;
        self.refused = false;
        self.forget_gone_folds();
        self.clamp_cursor();
        self.message = Some("reloaded".to_string());
        let focus = if matches!(self.focus, Focus::Help(_)) {
            &mut self.under_help
        } else {
            &mut self.focus
        };
        if let Focus::Popup(Popup { target: Target::Edit(_), .. }) = focus {
            *focus = Focus::List;
            self.message = Some("reloaded, edit cancelled".to_string());
        }
    }
}

#[cfg(test)]
mod tests;
