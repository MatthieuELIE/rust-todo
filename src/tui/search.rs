use super::*;

impl App {
    /// Applies a key typed in the search line, the list back on its first row.
    pub(super) fn search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => self.search.push(c),
            KeyCode::Backspace => _ = self.search.pop(),
            KeyCode::Esc => {
                self.search.clear();
                self.focus = Focus::List;
            }
            KeyCode::Enter => self.focus = Focus::List,
            _ => {}
        }
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests;
