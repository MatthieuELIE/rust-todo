use super::super::tests::{app, key, press, shown};
use super::super::*;

fn scroll(app: &App) -> u16 {
    match app.focus {
        Focus::Help(scroll) => scroll,
        _ => panic!("the help is closed"),
    }
}

fn help_app() -> App {
    let app = app();
    app.help_rows.set(26);
    app
}

#[test]
fn j_and_k_scroll_the_help_a_line_and_stop_at_the_top_and_the_bottom() {
    let mut app = help_app();
    let max = crate::view::HELP.lines().count() as u16 - 26;

    press(&mut app, "?");
    assert_eq!(scroll(&app), 0);
    press(&mut app, "k");
    assert_eq!(scroll(&app), 0);
    press(&mut app, "j");
    assert_eq!(scroll(&app), 1);
    press(&mut app, "k");
    assert_eq!(scroll(&app), 0);
    press(&mut app, &"j".repeat(max as usize + 5));
    assert_eq!(scroll(&app), max);
    key(&mut app, KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(scroll(&app), max - 1);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(scroll(&app), max);
}

#[test]
fn capital_g_scrolls_the_help_to_the_bottom_and_gg_back_to_the_top() {
    let mut app = help_app();
    let max = crate::view::HELP.lines().count() as u16 - 26;

    press(&mut app, "?G");
    assert_eq!(scroll(&app), max);
    press(&mut app, "g");
    assert_eq!(scroll(&app), max);
    press(&mut app, "g");
    assert_eq!(scroll(&app), 0);
}

#[test]
fn esc_enter_question_mark_and_q_each_close_the_help_with_no_other_effect() {
    for code in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char('?'), KeyCode::Char('q')] {
        let mut app = help_app();
        press(&mut app, "?");
        key(&mut app, code, KeyModifiers::NONE);
        assert!(matches!(app.focus, Focus::List), "{code:?}");
        assert!(!app.quit && app.moves.is_empty() && app.message.is_none(), "{code:?}");
        assert_eq!(shown(&app), ["one", "two", "three"]);
    }
}

#[test]
fn any_other_key_leaves_the_help_open_and_changes_nothing() {
    let mut app = help_app();
    press(&mut app, "?");

    for keys in ["x", "/", "dd", "o"] {
        assert!(!press(&mut app, keys));
        assert_eq!(scroll(&app), 0);
    }
    assert_eq!(shown(&app), ["one", "two", "three"]);
    assert!(app.moves.is_empty());
}

#[test]
fn ctrl_c_quits_from_the_help() {
    let mut app = help_app();
    press(&mut app, "?");
    key(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(app.quit);
}

#[test]
fn the_help_opened_again_starts_at_the_top() {
    let mut app = help_app();
    press(&mut app, "?jjj");
    assert_eq!(scroll(&app), 3);
    press(&mut app, "q?");
    assert_eq!(scroll(&app), 0);
}
