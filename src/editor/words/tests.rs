use super::super::tests::{LINE, TAGGED, after, keys, normal};

fn stops(text: &str, from: usize, key: char, count: usize) -> Vec<usize> {
    let mut ed = normal(text, from);
    (0..count)
        .map(|_| {
            keys(&mut ed, &key.to_string());
            ed.cursor
        })
        .collect()
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
fn iw_is_the_run_of_letters_punctuation_or_blanks_under_the_cursor_and_i_capital_w_the_blank_separated_word() {
    assert_eq!(after(TAGGED, 11, "ciwx"), ("Appeler +x due:2026-10-15".to_string(), 10));
    assert_eq!(after(TAGGED, 11, "ciWx"), ("Appeler x due:2026-10-15".to_string(), 9));
    assert_eq!(after(TAGGED, 26, "diw"), ("Appeler +banque due:2026--15".to_string(), 25));
    assert_eq!(after(TAGGED, 8, "diw"), ("Appeler banque due:2026-10-15".to_string(), 8));
    assert_eq!(after("Appeler  +banque", 7, "diw"), ("Appeler+banque".to_string(), 7));
    assert_eq!(after("", 0, "diw"), (String::new(), 0));
}

#[test]
fn aw_takes_the_blanks_after_the_word_too_or_those_before_it_when_none_follow() {
    assert_eq!(after(TAGGED, 11, "daw"), ("Appeler +due:2026-10-15".to_string(), 9));
    assert_eq!(after(TAGGED, 11, "daW"), ("Appeler due:2026-10-15".to_string(), 8));
    assert_eq!(after(TAGGED, 26, "daw"), ("Appeler +banque due:2026--15".to_string(), 25));
    assert_eq!(after(TAGGED, 26, "daW"), ("Appeler +banque".to_string(), 14));
    assert_eq!(after("Appeler  +banque", 7, "daw"), ("Appelerbanque".to_string(), 7));
    assert_eq!(after("Appeler  +banque", 8, "caWx"), ("Appelerx".to_string(), 8));
    assert_eq!(after("Appeler  ", 8, "daw"), ("Appeler".to_string(), 6));
}
