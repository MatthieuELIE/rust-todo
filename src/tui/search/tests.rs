use super::super::tests::{TODAY, app, app_of, press, shown};
use super::super::*;

#[test]
fn slash_filters_at_each_letter_from_the_top_and_enter_keeps_the_search() {
    let mut app = app_of(&["Call the bank", "Pay rent", "Call mom"]);

    press(&mut app, "jj/call");
    assert_eq!(shown(&app), ["Call the bank", "Call mom"]);
    assert_eq!(app.cursor, 0);
    app.cursor = 1;
    app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
    assert_eq!((app.search.as_str(), app.cursor), ("cal", 0));
    press(&mut app, "l");

    press(&mut app, " -mom");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "j");
    assert_eq!(shown(&app), ["Call the bank"]);
}

#[test]
fn esc_drops_the_search_being_typed_or_the_one_kept_in_the_list() {
    let mut app = app();
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "/one");
    app.handle_key(esc, TODAY);
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "/one");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app.handle_key(esc, TODAY);
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "jj");
    app.handle_key(esc, TODAY);
    assert_eq!(app.cursor, 0);

    press(&mut app, "/one");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "/");
    assert_eq!(app.search, "");
}

#[test]
fn a_control_key_in_the_search_types_nothing() {
    let mut app = app();

    press(&mut app, "/t");
    app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL), TODAY);

    assert_eq!(app.search, "t");
}
