use super::*;
use crate::view::HELP;

impl App {
    /// Opens the help from the focus it covers, kept in `under_help` for the help to give back.
    pub(super) fn open_help(&mut self) {
        self.under_help = std::mem::take(&mut self.focus);
        self.focus = Focus::Help(0);
    }

    /// Applies a key typed in the help: `j`/`k` and `gg`/`G` scroll within the lines the last drawing showed, `Esc`, `Enter`, `?` and `q` close it, back to the focus it covered.
    pub(super) fn help_key(&mut self, key: KeyEvent, pending: Option<char>) {
        let Focus::Help(scroll) = &mut self.focus else {
            unreachable!("the help is open");
        };
        let max = (HELP.lines().count() as u16).saturating_sub(self.help_rows.get());
        match key.code {
            _ if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {}
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?' | 'q') => self.focus = std::mem::take(&mut self.under_help),
            KeyCode::Char('j') | KeyCode::Down => *scroll = (*scroll + 1).min(max),
            KeyCode::Char('k') | KeyCode::Up => *scroll = scroll.saturating_sub(1).min(max),
            KeyCode::Char('g') if pending == Some('g') => *scroll = 0,
            KeyCode::Char('g') => self.pending = Some('g'),
            KeyCode::Char('G') => *scroll = max,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
