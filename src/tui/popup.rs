use super::picker::shift;
use super::*;

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
    /// Row picked among the completions of the tag being typed.
    pub selected: usize,
    /// Date picked in the date picker, open while set.
    pub picker: Option<Date>,
}

impl App {
    /// Applies a key typed in the popup, to the date picker when open, arrows and `Tab` to the completions when shown; tells whether to write.
    pub(super) fn popup_key(&mut self, key: KeyEvent, today: Date) -> bool {
        let (names, selected) = self.completions();
        let Focus::Popup(popup) = &mut self.focus else {
            unreachable!("the popup is open");
        };
        let opens_help = key.code == KeyCode::Char('?')
            && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            && (popup.picker.is_some() || (popup.editor.mode == Mode::Normal && popup.editor.pending().is_empty()));
        if opens_help {
            self.open_help();
            return false;
        }
        if let Some(date) = popup.picker {
            popup.picker = match key.code {
                _ if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => Some(date),
                KeyCode::Enter => {
                    popup.editor.complete(&format!("{DUE}{date}"));
                    None
                }
                KeyCode::Esc => None,
                code => Some(shift(date, code)),
            };
            return false;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            _ if names.is_empty() => {}
            (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
                popup.selected = (selected + 1).min(names.len() - 1);
                return false;
            }
            (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
                popup.selected = selected.saturating_sub(1);
                return false;
            }
            (KeyCode::Tab, _) => {
                popup.editor.complete(&names[selected].0);
                popup.selected = 0;
                return false;
            }
            _ => {}
        }
        popup.selected = 0;
        let outcome = popup.editor.handle_key(key);
        if popup.editor.wants_date() {
            popup.picker = Some(today);
        }
        match outcome {
            Outcome::Continue => false,
            Outcome::NotPriority => {
                self.refuse(NOT_PRIORITY);
                false
            }
            Outcome::Message(message) => {
                self.message = Some(message.to_string());
                false
            }
            outcome => self.close_popup(outcome, today),
        }
    }

    /// Closes the popup, adding or editing the task when its text was submitted, and tells whether the file must be rewritten; a refused line keeps the popup open.
    fn close_popup(&mut self, outcome: Outcome, today: Date) -> bool {
        let Focus::Popup(popup) = std::mem::replace(&mut self.focus, Focus::List) else {
            unreachable!("the popup is open");
        };
        let write = match (outcome, &popup.target) {
            (Outcome::Submit, Target::Add) => self.add(&popup.editor.text, today),
            (Outcome::Submit, Target::Edit(number)) => self.edit(*number, &popup.editor.text, today),
            _ => false,
        };
        if self.refused {
            self.focus = Focus::Popup(popup);
        }
        write
    }
}

#[cfg(test)]
mod tests;
