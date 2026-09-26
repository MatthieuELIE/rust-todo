use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What a key did to the line being edited.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    /// Keep editing.
    Continue,
    /// The text is to be saved.
    Submit,
    /// The text is to be dropped.
    Cancel,
}

/// A one-line text field with a cursor, edited as in insert mode.
#[derive(Default)]
pub struct Editor {
    /// The text typed so far.
    pub text: String,
    /// Position of the cursor, in characters: the next character typed goes before the one at this position.
    pub cursor: usize,
}

impl Editor {
    /// Applies one key press to the text and tells whether editing goes on.
    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let len = self.text.chars().count();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Enter => return Outcome::Submit,
            KeyCode::Esc => return Outcome::Cancel,
            KeyCode::Char('w') if ctrl => {
                let before: Vec<char> = self.text.chars().take(self.cursor).collect();
                let spaces = before.iter().rev().take_while(|c| c.is_whitespace()).count();
                let word = before.iter().rev().skip(spaces).take_while(|c| !c.is_whitespace()).count();
                self.remove(self.cursor - spaces - word, self.cursor);
            }
            KeyCode::Char('u') if ctrl => self.remove(0, self.cursor),
            KeyCode::Char(c) if !ctrl => {
                self.text.insert(self.byte(self.cursor), c);
                self.cursor += 1;
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(len),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = len,
            KeyCode::Backspace if self.cursor > 0 => self.remove(self.cursor - 1, self.cursor),
            KeyCode::Delete if self.cursor < len => self.remove(self.cursor, self.cursor + 1),
            _ => {}
        }
        Outcome::Continue
    }

    /// Erases the characters from position `start` up to `end`, leaving the cursor at `start`.
    fn remove(&mut self, start: usize, end: usize) {
        let range = self.byte(start)..self.byte(end);
        self.text.replace_range(range, "");
        self.cursor = start;
    }

    /// Byte offset of the character at `position`, the text's length past its end.
    fn byte(&self, position: usize) -> usize {
        self.text.char_indices().nth(position).map_or(self.text.len(), |(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(text: &str, cursor: usize) -> Editor {
        Editor {
            text: text.to_string(),
            cursor,
        }
    }

    fn press(editor: &mut Editor, keys: &[KeyEvent]) {
        for key in keys {
            editor.handle_key(*key);
        }
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn state(editor: &Editor) -> (&str, usize) {
        (&editor.text, editor.cursor)
    }

    #[test]
    fn a_character_goes_in_at_the_cursor() {
        let mut ed = editor("Cal bank", 3);

        press(&mut ed, &[KeyEvent::from(KeyCode::Char('l'))]);

        assert_eq!(state(&ed), ("Call bank", 4));
    }

    #[test]
    fn left_and_right_move_one_character_without_leaving_the_text() {
        let mut ed = editor("ab", 0);
        let (left, right) = (KeyEvent::from(KeyCode::Left), KeyEvent::from(KeyCode::Right));

        press(&mut ed, &[left]);
        assert_eq!(ed.cursor, 0);
        press(&mut ed, &[right, right, right]);
        assert_eq!(ed.cursor, 2);
        press(&mut ed, &[left]);
        assert_eq!(ed.cursor, 1);
    }

    #[test]
    fn home_and_end_jump_to_either_end() {
        let mut ed = editor("Call bank", 4);

        press(&mut ed, &[KeyEvent::from(KeyCode::Home)]);
        assert_eq!(ed.cursor, 0);
        press(&mut ed, &[KeyEvent::from(KeyCode::End)]);
        assert_eq!(ed.cursor, 9);
    }

    #[test]
    fn backspace_erases_before_the_cursor_and_delete_under_it_each_stopping_at_the_edge() {
        let (backspace, delete) = (KeyEvent::from(KeyCode::Backspace), KeyEvent::from(KeyCode::Delete));

        let mut ed = editor("abc", 1);
        press(&mut ed, &[backspace, backspace]);
        assert_eq!(state(&ed), ("bc", 0));

        let mut ed = editor("abc", 1);
        press(&mut ed, &[delete]);
        assert_eq!(state(&ed), ("ac", 1));
        press(&mut ed, &[delete, delete]);
        assert_eq!(state(&ed), ("a", 1));
    }

    #[test]
    fn ctrl_w_erases_the_word_before_the_cursor_and_the_spaces_after_it() {
        let mut ed = editor("Call the bank  ", 15);
        press(&mut ed, &[ctrl('w')]);
        assert_eq!(state(&ed), ("Call the ", 9));

        let mut ed = editor("Call the bank", 11);
        press(&mut ed, &[ctrl('w')]);
        assert_eq!(state(&ed), ("Call the nk", 9));
    }

    #[test]
    fn ctrl_u_erases_from_the_start_to_the_cursor() {
        let mut ed = editor("Call the bank", 8);

        press(&mut ed, &[ctrl('u')]);

        assert_eq!(state(&ed), (" bank", 0));
    }

    #[test]
    fn the_cursor_counts_characters_so_accents_edit_like_any_letter() {
        let mut ed = editor("", 0);

        press(&mut ed, &"Été à".chars().map(|c| KeyEvent::from(KeyCode::Char(c))).collect::<Vec<_>>());
        press(&mut ed, &[KeyEvent::from(KeyCode::Left), KeyEvent::from(KeyCode::Backspace)]);

        assert_eq!(state(&ed), ("Étéà", 3));
    }

    #[test]
    fn enter_submits_esc_cancels_and_other_control_keys_are_not_typed() {
        let mut ed = editor("ab", 2);

        assert_eq!(ed.handle_key(ctrl('a')), Outcome::Continue);
        assert_eq!(state(&ed), ("ab", 2));
        assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Submit);
        assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Esc)), Outcome::Cancel);
    }
}
