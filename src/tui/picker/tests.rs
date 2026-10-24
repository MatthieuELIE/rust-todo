use super::super::tests::{TODAY, app, key, popup, press};
use super::super::*;
use super::shift;
use time::macros::date;

#[test]
fn a_month_step_in_the_date_picker_crosses_the_year_and_keeps_to_the_month_length() {
    assert_eq!(shift(date!(2026 - 12 - 15), KeyCode::Char('L')), date!(2027 - 01 - 15));
    assert_eq!(shift(date!(2027 - 01 - 15), KeyCode::Char('H')), date!(2026 - 12 - 15));
    assert_eq!(shift(date!(2028 - 01 - 31), KeyCode::Char('L')), date!(2028 - 02 - 29));
}

#[test]
fn due_colon_typed_in_the_popup_opens_a_date_picker_on_today_whose_keys_move_the_date_and_enter_writes_it() {
    let mut app = app();
    let picked = |app: &App| popup(app).picker;

    press(&mut app, "oShip due:");
    assert_eq!(picked(&app), Some(TODAY));
    press(&mut app, "lj");
    assert_eq!(picked(&app), Some(date!(2026 - 10 - 04)));
    press(&mut app, "hk");
    assert_eq!(picked(&app), Some(TODAY));
    key(&mut app, KeyCode::Right, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Left, KeyModifiers::NONE);
    key(&mut app, KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(picked(&app), Some(TODAY));
    press(&mut app, "L");
    assert_eq!(picked(&app), Some(date!(2026 - 10 - 26)));
    press(&mut app, "HHlllll");
    assert_eq!(picked(&app), Some(date!(2026 - 08 - 31)));
    press(&mut app, "Lx?");
    app.paste("2026-01-01");
    assert_eq!(
        (picked(&app), popup(&app).editor.text.as_str()),
        (Some(date!(2026 - 09 - 30)), "Ship due:")
    );

    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(picked(&app), None);
    assert_eq!(
        (popup(&app).editor.text.as_str(), popup(&app).editor.mode),
        ("Ship due:2026-09-30 ", Mode::Insert)
    );
}

#[test]
fn esc_closes_the_date_picker_on_due_colon_and_a_key_leaving_due_colon_again_reopens_it() {
    let mut app = app();

    press(&mut app, "oShip due:");
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!((popup(&app).picker, popup(&app).editor.text.as_str()), (None, "Ship due:"));
    assert_eq!(popup(&app).editor.mode, Mode::Insert);

    press(&mut app, "2");
    assert_eq!(popup(&app).picker, None);
    key(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(popup(&app).picker, Some(TODAY));
}
