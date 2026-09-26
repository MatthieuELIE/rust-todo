use super::*;

const LINE: &str = "(A) 2026-09-26 Call +bank due:2026-10-01";

fn editor(text: &str, cursor: usize) -> Editor {
    Editor {
        text: text.to_string(),
        cursor,
        ..Editor::default()
    }
}

fn normal(text: &str, cursor: usize) -> Editor {
    Editor {
        mode: Mode::Normal,
        ..editor(text, cursor)
    }
}

fn keys(editor: &mut Editor, keys: &str) {
    for c in keys.chars() {
        editor.handle_key(KeyEvent::from(KeyCode::Char(c)));
    }
}

fn stops(text: &str, from: usize, key: char, count: usize) -> Vec<usize> {
    let mut ed = normal(text, from);
    (0..count)
        .map(|_| {
            keys(&mut ed, &key.to_string());
            ed.cursor
        })
        .collect()
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

#[test]
fn h_l_0_and_dollar_move_from_character_to_character_without_leaving_the_text() {
    let mut ed = normal("abc", 1);

    keys(&mut ed, "hh");
    assert_eq!(ed.cursor, 0);
    keys(&mut ed, "lll");
    assert_eq!(ed.cursor, 2);
    keys(&mut ed, "0");
    assert_eq!(ed.cursor, 0);
    keys(&mut ed, "$");
    assert_eq!(ed.cursor, 2);
}

#[test]
fn w_stops_at_each_run_of_letters_or_punctuation_and_stays_on_the_last_character_at_the_end() {
    assert_eq!(stops(LINE, 0, 'w', 6), [1, 2, 4, 8, 9, 11]);
    assert_eq!(stops(LINE, 12, 'w', 4), [15, 20, 21, 26]);
    assert_eq!(stops(LINE, 37, 'w', 2), [38, 39]);
}

#[test]
fn capital_w_jumps_from_blank_separated_word_to_word() {
    assert_eq!(stops(LINE, 0, 'W', 5), [4, 15, 20, 26, 39]);
}

#[test]
fn e_goes_to_the_end_of_the_word_or_of_the_next_one_when_already_there() {
    assert_eq!(stops(LINE, 0, 'e', 4), [1, 2, 7, 8]);
    assert_eq!(stops(LINE, 15, 'e', 3), [18, 20, 24]);
    assert_eq!(stops(LINE, 38, 'e', 2), [39, 39]);
    assert_eq!(stops(LINE, 15, 'E', 3), [18, 24, 39]);
}

#[test]
fn b_goes_to_the_start_of_the_word_or_of_the_previous_one_when_already_there() {
    assert_eq!(stops(LINE, 23, 'b', 4), [21, 20, 15, 12]);
    assert_eq!(stops(LINE, 2, 'b', 3), [1, 0, 0]);
    assert_eq!(stops(LINE, 33, 'B', 2), [26, 20]);
}

#[test]
fn x_deletes_under_the_cursor_which_stays_on_the_text() {
    let mut ed = normal("abc", 2);

    keys(&mut ed, "x");
    assert_eq!(state(&ed), ("ab", 1));
    keys(&mut ed, "xxx");
    assert_eq!(state(&ed), ("", 0));
}

#[test]
fn capital_d_deletes_to_the_end_and_capital_c_then_inserts() {
    let mut ed = normal(LINE, 15);
    keys(&mut ed, "D");
    assert_eq!(state(&ed), ("(A) 2026-09-26 ", 14));
    assert_eq!(ed.mode, Mode::Normal);

    let mut ed = normal(LINE, 15);
    keys(&mut ed, "C");
    assert_eq!(state(&ed), ("(A) 2026-09-26 ", 15));
    assert_eq!(ed.mode, Mode::Insert);
}

#[test]
fn dw_stops_at_punctuation_and_d_capital_w_deletes_the_whole_blank_separated_word() {
    let mut ed = normal(LINE, 26);
    keys(&mut ed, "dw");
    assert_eq!(state(&ed), ("(A) 2026-09-26 Call +bank :2026-10-01", 26));

    let mut ed = normal(LINE, 26);
    keys(&mut ed, "dW");
    assert_eq!(state(&ed), ("(A) 2026-09-26 Call +bank ", 25));

    let mut ed = normal(LINE, 15);
    keys(&mut ed, "dW");
    assert_eq!(state(&ed), ("(A) 2026-09-26 +bank due:2026-10-01", 15));
}

#[test]
fn cw_changes_up_to_the_end_of_the_word_keeping_the_spaces_after_it() {
    let mut ed = normal("Call  +bank", 1);

    keys(&mut ed, "cwheck");

    assert_eq!(state(&ed), ("Check  +bank", 5));
    assert_eq!(ed.mode, Mode::Insert);

    let mut ed = normal(LINE, 21);
    keys(&mut ed, "cW");
    assert_eq!(ed.text, "(A) 2026-09-26 Call + due:2026-10-01");
}

#[test]
fn a_d_followed_by_another_key_is_dropped_and_that_key_acts() {
    let mut ed = normal("abc", 0);

    keys(&mut ed, "dl");
    assert_eq!(state(&ed), ("abc", 1));
    keys(&mut ed, "x");
    assert_eq!(state(&ed), ("ac", 1));
}

#[test]
fn i_a_capital_i_and_capital_a_enter_insert_mode_at_their_place() {
    let insert_at = |from: usize, key: &str| {
        let mut ed = normal("  abc", from);
        keys(&mut ed, key);
        assert_eq!(ed.mode, Mode::Insert);
        ed.cursor
    };

    assert_eq!(insert_at(3, "i"), 3);
    assert_eq!(insert_at(3, "a"), 4);
    assert_eq!(insert_at(4, "I"), 2);
    assert_eq!(insert_at(0, "A"), 5);
    assert_eq!(insert_at(0, "i"), 0);
}

#[test]
fn enter_submits_and_other_control_keys_are_not_typed() {
    let mut ed = editor("ab", 2);

    assert_eq!(ed.handle_key(ctrl('a')), Outcome::Continue);
    assert_eq!(state(&ed), ("ab", 2));
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Submit);
}
