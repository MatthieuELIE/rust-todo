use super::super::tests::{LINE, TAGGED, ctrl, editor, keys, normal, press, state};
use super::*;

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

    let mut ed = normal("Call  +bank", 4);
    keys(&mut ed, "cw");
    assert_eq!(state(&ed), ("Call+bank", 4));
}

#[test]
fn d_deletes_up_to_where_a_motion_goes_the_end_of_a_word_included() {
    let after = |from: usize, typed: &str| {
        let mut ed = normal("Call the bank", from);
        keys(&mut ed, typed);
        (ed.text, ed.cursor)
    };

    assert_eq!(after(6, "db"), ("Call he bank".to_string(), 5));
    assert_eq!(after(5, "de"), ("Call  bank".to_string(), 5));
    assert_eq!(after(7, "d0"), ("e bank".to_string(), 0));
    assert_eq!(after(5, "d$"), ("Call ".to_string(), 4));
    assert_eq!(after(5, "dh"), ("Callthe bank".to_string(), 4));
    assert_eq!(after(5, "dl"), ("Call he bank".to_string(), 5));
    assert_eq!(after(0, "dh"), ("Call the bank".to_string(), 0));
    assert_eq!(after(9, "dB"), ("Call bank".to_string(), 5));
    assert_eq!(after(5, "dE"), ("Call  bank".to_string(), 5));
}

#[test]
fn c_deletes_as_d_does_then_inserts() {
    let mut ed = normal("Call the bank", 7);
    keys(&mut ed, "cba");
    assert_eq!(state(&ed), ("Call ae bank", 6));
    assert_eq!(ed.mode, Mode::Insert);

    let mut ed = normal("Call the bank", 5);
    keys(&mut ed, "c$x");
    assert_eq!(state(&ed), ("Call x", 6));

    let mut ed = normal("Call the bank", 0);
    keys(&mut ed, "ch");
    assert_eq!(state(&ed), ("Call the bank", 0));
    assert_eq!(ed.mode, Mode::Insert);
}

#[test]
fn a_key_completing_no_command_after_d_or_c_cancels_it_and_does_nothing_esc_and_enter_included() {
    let mut ed = normal("abc", 1);

    keys(&mut ed, "dxcA");
    assert_eq!(state(&ed), ("abc", 1));
    assert_eq!(ed.mode, Mode::Normal);
    keys(&mut ed, "x");
    assert_eq!(state(&ed), ("ac", 1));
    keys(&mut ed, "du");
    assert_eq!(state(&ed), ("ac", 1));

    keys(&mut ed, "d");
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Esc)), Outcome::Continue);
    keys(&mut ed, "c");
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Continue);
    keys(&mut ed, "l");
    assert_eq!(state(&ed), ("ac", 1));
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Esc)), Outcome::Cancel);
}

#[test]
fn a_key_that_is_not_w_after_di_or_da_cancels_the_command_and_does_nothing() {
    let mut ed = normal(TAGGED, 11);

    keys(&mut ed, "dixcaidildihdidci$da");
    assert_eq!(state(&ed), (TAGGED, 11));
    assert_eq!(ed.mode, Mode::Normal);
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Esc)), Outcome::Continue);
    keys(&mut ed, "x");
    assert_eq!(state(&ed), ("Appeler +baque due:2026-10-15", 11));
}

#[test]
fn dd_empties_the_text_and_cc_then_inserts() {
    let mut ed = normal("Call the bank", 6);
    keys(&mut ed, "dd");
    assert_eq!(state(&ed), ("", 0));
    assert_eq!(ed.mode, Mode::Normal);

    let mut ed = normal("Call the bank", 6);
    keys(&mut ed, "ccPay");
    assert_eq!(state(&ed), ("Pay", 3));
    assert_eq!(ed.mode, Mode::Insert);
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
fn p_then_a_letter_writes_or_replaces_the_priority_keeping_the_cursor_on_its_character() {
    let mut ed = normal("Call +bank", 5);

    keys(&mut ed, "pb");
    assert_eq!(state(&ed), ("(B) Call +bank", 9));
    assert_eq!(ed.mode, Mode::Normal);
    keys(&mut ed, "pa");
    assert_eq!(state(&ed), ("(A) Call +bank", 9));
    keys(&mut ed, "p ");
    assert_eq!(state(&ed), ("Call +bank", 5));

    let mut ed = normal(LINE, 1);
    keys(&mut ed, "pe");
    assert_eq!(state(&ed), ("(E) 2026-09-26 Call +bank due:2026-10-01", 1));
    keys(&mut ed, "p ");
    assert_eq!(state(&ed), ("2026-09-26 Call +bank due:2026-10-01", 0));
}

#[test]
fn p_leaves_a_done_line_alone_and_is_a_letter_in_insert_mode() {
    let mut ed = normal("x 2026-09-26 Call", 3);
    keys(&mut ed, "pa");
    assert_eq!(state(&ed), ("x 2026-09-26 Call", 3));
    assert_eq!(ed.mode, Mode::Normal);

    let mut ed = editor("", 0);
    keys(&mut ed, "pa");
    assert_eq!(state(&ed), ("pa", 2));
}

#[test]
fn a_control_key_in_normal_mode_is_not_its_letter() {
    let mut ed = normal("Call the bank", 0);
    keys(&mut ed, "x");

    press(&mut ed, &[ctrl('w'), ctrl('x'), ctrl('u')]);
    assert_eq!(state(&ed), ("all the bank", 0));

    keys(&mut ed, "d");
    press(&mut ed, &[ctrl('w')]);
    keys(&mut ed, "d");
    assert_eq!(state(&ed), ("all the bank", 0));

    ed.handle_key(KeyEvent::from(KeyCode::Esc));
    keys(&mut ed, "p");
    assert_eq!(ed.handle_key(ctrl('a')), Outcome::NotPriority);
}
