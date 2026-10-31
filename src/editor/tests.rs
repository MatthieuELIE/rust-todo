use super::*;

pub(super) const LINE: &str = "(A) 2026-09-26 Call +bank due:2026-10-01";

pub(super) fn editor(text: &str, cursor: usize) -> Editor {
    Editor {
        text: text.to_string(),
        cursor,
        rest: (text.to_string(), cursor),
        ..Editor::default()
    }
}

pub(super) fn normal(text: &str, cursor: usize) -> Editor {
    Editor {
        mode: Mode::Normal,
        ..editor(text, cursor)
    }
}

pub(super) fn keys(editor: &mut Editor, keys: &str) {
    for c in keys.chars() {
        editor.handle_key(KeyEvent::from(KeyCode::Char(c)));
    }
}

pub(super) fn press(editor: &mut Editor, keys: &[KeyEvent]) {
    for key in keys {
        editor.handle_key(*key);
    }
}

pub(super) fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

pub(super) fn state(editor: &Editor) -> (&str, usize) {
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
fn esc_goes_from_insert_to_normal_one_character_back_and_cancels_from_normal() {
    let esc = KeyEvent::from(KeyCode::Esc);

    let mut ed = editor("abc", 3);
    assert_eq!(ed.handle_key(esc), Outcome::Continue);
    assert_eq!((ed.mode, ed.cursor), (Mode::Normal, 2));
    assert_eq!(ed.handle_key(esc), Outcome::Cancel);

    let mut ed = editor("abc", 0);
    ed.handle_key(esc);
    assert_eq!(ed.cursor, 0);
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Submit);
}

pub(super) const TAGGED: &str = "Appeler +banque due:2026-10-15";

pub(super) fn after(text: &str, from: usize, typed: &str) -> (String, usize) {
    let mut ed = normal(text, from);
    keys(&mut ed, typed);
    (ed.text, ed.cursor)
}

#[test]
fn u_undoes_the_normal_commands_one_by_one_and_ctrl_r_redoes_them_until_a_new_change() {
    let u = KeyEvent::from(KeyCode::Char('u'));
    let mut ed = normal("Call the bank", 5);
    keys(&mut ed, "dwlx");
    assert_eq!(state(&ed), ("Call bnk", 6));

    assert_eq!(ed.handle_key(u), Outcome::Notice(Notice::Undone));
    assert_eq!(state(&ed), ("Call bank", 6));
    ed.handle_key(u);
    assert_eq!(state(&ed), ("Call the bank", 5));
    assert_eq!(ed.handle_key(u), Outcome::Notice(Notice::NothingToUndo));

    assert_eq!(ed.handle_key(ctrl('r')), Outcome::Notice(Notice::Redone));
    assert_eq!(state(&ed), ("Call bank", 6));
    keys(&mut ed, "x");
    assert_eq!(ed.handle_key(ctrl('r')), Outcome::Notice(Notice::NothingToRedo));
    assert_eq!(state(&ed), ("Call bnk", 6));
}

#[test]
fn the_text_typed_from_entering_insert_mode_to_esc_is_undone_at_once() {
    let esc = KeyEvent::from(KeyCode::Esc);
    let mut ed = normal("Call the bank", 5);
    keys(&mut ed, "cwmy");
    ed.handle_key(esc);
    assert_eq!(state(&ed), ("Call my bank", 6));
    keys(&mut ed, "u");
    assert_eq!(state(&ed), ("Call the bank", 5));

    let mut added = Editor::default();
    keys(&mut added, "Pay rent");
    added.handle_key(esc);
    keys(&mut added, "u");
    assert_eq!(state(&added), ("", 0));

    keys(&mut added, "r");
    assert_eq!(state(&added), ("", 0));

    let mut opened = Editor::new("Pay rent".to_string());
    keys(&mut opened, "l");
    assert_eq!(
        opened.handle_key(KeyEvent::from(KeyCode::Char('u'))),
        Outcome::Notice(Notice::NothingToUndo)
    );
    keys(&mut opened, "xu");
    assert_eq!(state(&opened), ("Pay rent", 1));
}

#[test]
fn enter_submits_and_other_control_keys_are_not_typed() {
    let mut ed = editor("ab", 2);

    assert_eq!(ed.handle_key(ctrl('a')), Outcome::Continue);
    assert_eq!(state(&ed), ("ab", 2));
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Submit);
}

#[test]
fn a_paste_in_normal_mode_leaves_the_cursor_on_a_character() {
    let mut ed = normal("", 0);

    ed.paste("Call");

    assert_eq!(state(&ed), ("Call", 3));
    keys(&mut ed, "u");
    assert_eq!(state(&ed), ("", 0));
}

#[test]
fn the_tag_is_the_word_before_the_cursor_when_it_starts_with_a_plus_an_at_or_wait_colon_in_insert_mode() {
    assert_eq!(editor("Call +ba", 8).tag(), Some("+ba"));
    assert_eq!(editor("Call +bank now", 7).tag(), Some("+b"));
    assert_eq!(editor("Call @", 6).tag(), Some("@"));
    assert_eq!(editor("Call +bank ", 11).tag(), None);
    assert_eq!(editor("Call +bank", 5).tag(), None);
    assert_eq!(editor("a+b", 3).tag(), None);
    assert_eq!(normal("Call +bank", 7).tag(), None);
    assert_eq!(editor("Design wait:", 12).tag(), Some("wait:"));
    assert_eq!(editor("Design wait:fi", 14).tag(), Some("wait:fi"));
    assert_eq!(editor("Design wai", 10).tag(), None);
}

#[test]
fn a_bare_due_before_the_cursor_wants_a_date_in_insert_mode_only() {
    assert!(editor("Pay due:", 8).wants_date());
    assert!(!editor("Pay due:2026", 12).wants_date());
    assert!(!normal("Pay due: rent", 8).wants_date());
}

#[test]
fn completing_writes_the_name_over_the_whole_word_then_one_space_the_cursor_after_it() {
    let mut ed = editor("Call +b", 7);
    ed.complete("+bank");
    assert_eq!(state(&ed), ("Call +bank ", 11));

    let mut ed = editor("Call +bnk now", 7);
    ed.complete("+Bank");
    assert_eq!(state(&ed), ("Call +Bank now", 11));
}
