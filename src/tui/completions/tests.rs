use super::super::tests::{app_of, history_of, key, names, popup, press, shown};
use super::super::*;

#[test]
fn a_tag_typed_in_the_popup_lists_the_names_of_the_file_starting_like_it_most_used_first() {
    let mut app = app_of(&[
        "Pay +bank",
        "Call +Bank @phone",
        "x 2026-09-20 Old +bank",
        "Read +books +bank",
        "Sell +rent",
    ]);

    press(&mut app, "oBuy +b");
    assert_eq!(names(&app), ["+bank 3", "+Bank 1", "+books 1"]);
    press(&mut app, "O");
    assert_eq!(names(&app), ["+books 1"]);
    press(&mut app, " @");
    assert_eq!(names(&app), ["@phone 1"]);
    press(&mut app, " ");
    assert!(names(&app).is_empty());

    press(&mut app, "+r");
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(names(&app), ["+bank 3", "+Bank 1", "+books 1", "+rent 1"]);
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert!(names(&app).is_empty());
    key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(names(&app).len(), 4);
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(names(&app).is_empty());
}

#[test]
fn wait_colon_typed_in_the_popup_lists_the_wait_values_of_the_file_most_used_first() {
    let mut app = app_of(&[
        "Design wait:figma",
        "x 2026-09-20 Mock wait:figma",
        "Mail wait:designer",
        "Note wait: later",
        "Pay +rent",
    ]);

    press(&mut app, "oDraw wait:");
    assert_eq!(names(&app), ["wait:figma 2", "wait:designer 1"]);
    press(&mut app, "d");
    assert_eq!(names(&app), ["wait:designer 1"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(popup(&app).editor.text, "Draw wait:designer ");
}

#[test]
fn the_names_of_done_txt_complete_a_tag_after_those_of_the_task_file_with_a_count_of_zero() {
    let mut app = history_of(
        &["Pay +bank", "Call +bank @phone", "Read +books"],
        &["x 2026-09-20 Old +bank +archive @Home wait:bob", "x 2026-09-21 Older +Attic +archive"],
    );

    press(&mut app, "o+");
    assert_eq!(names(&app), ["+bank 2", "+books 1", "+archive 0", "+Attic 0"]);
    press(&mut app, " @");
    assert_eq!(names(&app), ["@phone 1", "@Home 0"]);
    press(&mut app, " wait:");
    assert_eq!(names(&app), ["wait:bob 0"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(popup(&app).editor.text, "+ @ wait:bob ");
}

#[test]
fn arrows_and_ctrl_n_p_pick_a_name_that_tab_writes_in_place_of_the_tag() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Read +books"]);
    let pick = |app: &mut App, code: KeyCode, modifiers: KeyModifiers| {
        key(app, code, modifiers);
        app.completions().1
    };

    press(&mut app, "oCall +B");
    assert_eq!(app.completions().1, 0);
    assert_eq!(pick(&mut app, KeyCode::Down, KeyModifiers::NONE), 1);
    assert_eq!(pick(&mut app, KeyCode::Down, KeyModifiers::NONE), 1);
    assert_eq!(pick(&mut app, KeyCode::Char('p'), KeyModifiers::CONTROL), 0);
    assert_eq!(pick(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL), 1);
    assert_eq!(pick(&mut app, KeyCode::Up, KeyModifiers::NONE), 0);
    assert_eq!(pick(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL), 1);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!((popup(&app).editor.text.as_str(), popup(&app).editor.cursor), ("Call +books ", 12));
    assert!(names(&app).is_empty());

    press(&mut app, "+b now");
    (0..5).for_each(|_| key(&mut app, KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(names(&app), ["+bank 2", "+books 1"]);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(
        (popup(&app).editor.text.as_str(), popup(&app).editor.cursor),
        ("Call +books +bank now", 18)
    );

    press(&mut app, "+");
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(shown(&app).last().unwrap(), "2026-09-26 Call +books +bank +now");
}

#[test]
fn the_pick_goes_back_to_the_first_name_after_a_typed_key_and_after_tab() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Wash +bath"]);

    press(&mut app, "o+b");
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.completions().1, 1);
    press(&mut app, "a");
    assert_eq!(
        (names(&app), app.completions().1),
        (vec!["+bank 2".to_string(), "+bath 1".to_string()], 0)
    );

    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    app.paste("+b");
    assert_eq!((popup(&app).editor.text.as_str(), app.completions().1), ("+bath +b", 0));
}

#[test]
fn names_used_as_often_are_listed_alphabetically_whatever_their_case() {
    let mut app = app_of(&["Pay +Bank", "Eat +apple"]);

    press(&mut app, "o+");

    assert_eq!(names(&app), ["+apple 1", "+Bank 1"]);
}
