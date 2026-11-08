use super::super::tests::{keys, normal, state};

use super::*;

#[test]
fn the_selection_runs_between_the_anchor_and_the_cursor_both_ends_included_either_way() {
    let mut ed = normal("one two three", 4);
    assert_eq!(ed.selection(), None);

    keys(&mut ed, "ve");
    assert_eq!(ed.selection(), Some(4..7));

    keys(&mut ed, "bb");
    assert_eq!(ed.selection(), Some(0..5));

    keys(&mut ed, "v");
    assert_eq!(ed.selection(), None);
}

#[test]
fn the_cursor_stays_on_a_character_whatever_the_motion() {
    let mut ed = normal("one two", 0);

    keys(&mut ed, "vllllllllll");
    assert_eq!(ed.cursor, 6);

    keys(&mut ed, "$");
    assert_eq!(ed.cursor, 6);
}

#[test]
fn y_leaves_the_selection_to_copy_and_the_cursor_at_its_start_in_normal_mode() {
    let mut ed = normal("one two", 4);

    keys(&mut ed, "vhh");
    let outcome = ed.handle_key(KeyEvent::from(KeyCode::Char('y')));

    assert_eq!(outcome, Outcome::Copy("e t".to_string()));
    assert_eq!((state(&ed), ed.mode), (("one two", 2), Mode::Normal));
}
