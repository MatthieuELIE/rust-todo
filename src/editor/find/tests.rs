use super::super::tests::{keys, normal, press, state};

use super::*;

#[test]
fn f_and_t_go_to_the_next_character_and_capital_ones_to_the_previous() {
    let mut ed = normal("a x b x c x", 4);

    keys(&mut ed, "fx");
    assert_eq!(ed.cursor, 6);
    keys(&mut ed, "Fx");
    assert_eq!(ed.cursor, 2);
    keys(&mut ed, "tx");
    assert_eq!(ed.cursor, 5);
    keys(&mut ed, "Tx");
    assert_eq!(ed.cursor, 3);
}

#[test]
fn a_missing_character_leaves_the_cursor() {
    let mut ed = normal("a x b", 2);

    keys(&mut ed, "fzFztzTz");

    assert_eq!(state(&ed), ("a x b", 2));
}

#[test]
fn d_takes_t_and_f_up_to_and_including_the_arrival() {
    let mut ed = normal("due:2026-10-15", 0);
    keys(&mut ed, "dt:");
    assert_eq!(state(&ed), (":2026-10-15", 0));

    let mut ed = normal("due:2026-10-15", 0);
    keys(&mut ed, "df:");
    assert_eq!(state(&ed), ("2026-10-15", 0));
}

#[test]
fn d_with_a_capital_find_keeps_the_starting_character() {
    let mut ed = normal("a x bc", 5);

    keys(&mut ed, "dFx");

    assert_eq!(state(&ed), ("a c", 2));
}

#[test]
fn c_takes_a_find_and_a_miss_changes_nothing() {
    let mut ed = normal("a b c", 0);
    keys(&mut ed, "cfz");
    assert_eq!((state(&ed), ed.mode), (("a b c", 0), Mode::Normal));

    keys(&mut ed, "cf ");
    assert_eq!((state(&ed), ed.mode), (("b c", 0), Mode::Insert));
}

#[test]
fn a_key_that_is_not_a_character_drops_the_search() {
    let mut ed = normal("a x b", 0);

    keys(&mut ed, "f");
    assert_eq!(ed.pending(), "f");
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Esc)), Outcome::Continue);
    assert_eq!(ed.pending(), "");

    keys(&mut ed, "f");
    assert_eq!(ed.handle_key(KeyEvent::from(KeyCode::Enter)), Outcome::Continue);
    press(&mut ed, &[KeyEvent::from(KeyCode::Tab)]);
    keys(&mut ed, "d");
    press(&mut ed, &[KeyEvent::from(KeyCode::Left)]);
    keys(&mut ed, ";");
    assert_eq!(state(&ed), ("a x b", 0));
}

#[test]
fn semicolon_repeats_and_comma_reverses() {
    let mut ed = normal("a x b x c x", 0);

    keys(&mut ed, "fx;");
    assert_eq!(ed.cursor, 6);
    keys(&mut ed, ",");
    assert_eq!(ed.cursor, 2);
}

#[test]
fn a_repeated_t_skips_the_character_next_to_the_cursor() {
    let mut ed = normal("a x b x c", 0);

    keys(&mut ed, "tx");
    assert_eq!(ed.cursor, 1);
    keys(&mut ed, ";");
    assert_eq!(ed.cursor, 5);
}

#[test]
fn semicolon_without_a_search_or_after_a_miss_does_nothing() {
    let mut ed = normal("a x b", 2);
    keys(&mut ed, ";,");
    assert_eq!(ed.cursor, 2);

    keys(&mut ed, "fz;");
    assert_eq!(ed.cursor, 2);
}

#[test]
fn d_semicolon_deletes_like_df() {
    let mut ed = normal("a x b x c", 0);

    keys(&mut ed, "fxd;");

    assert_eq!(state(&ed), ("a  c", 2));
}
