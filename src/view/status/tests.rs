use super::super::tests::{MOCHA, TODAY, app_of, render_in, rows};
use super::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::Modifier;

const WIDTH: u16 = 109;

fn status_of(app: &App) -> String {
    rows(&render_in(app, WIDTH, 5))[4].clone()
}

fn popup_in(mode: Mode, picker: bool) -> App {
    let mut app = app_of(&["Pay +rent"]);
    app.handle_key(KeyEvent::from(KeyCode::Char('o')), TODAY);
    if let Focus::Popup(popup) = &mut app.focus {
        popup.editor.mode = mode;
        popup.picker = picker.then_some(TODAY);
    }
    app
}

#[test]
fn the_help_hint_shows_at_the_right_edge_in_list_history_panel_normal_and_date_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    let buffer = render_in(&app, WIDTH, 5);
    assert!(rows(&buffer)[4].ends_with("? help"));
    assert!(buffer[(WIDTH - 7, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(WIDTH - 7, 4)].fg, buffer[(WIDTH - 5, 4)].fg), (MOCHA.primary, MOCHA.tertiary));
    app.in_history = true;
    assert!(status_of(&app).ends_with("? help"));
    app.in_history = false;
    app.focus = Focus::Panel;
    assert!(status_of(&app).contains(" PANEL") && status_of(&app).ends_with("? help"));
    let normal = status_of(&popup_in(Mode::Normal, false));
    assert!(normal.contains(" NORMAL") && normal.ends_with("? help"), "{normal}");
    let date = status_of(&popup_in(Mode::Normal, true));
    assert!(date.contains(" DATE") && date.ends_with("? help"), "{date}");
}

#[test]
fn nothing_shows_on_the_right_in_insert_and_search_and_the_help_shows_its_block_in_mocha() {
    let insert = status_of(&popup_in(Mode::Insert, false));
    assert!(insert.contains("INSERT") && !insert.contains("help"), "{insert}");

    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Search;
    app.search = "ca".to_string();
    let buffer = render_in(&app, WIDTH, 5);
    assert_eq!(rows(&buffer)[4], " SEARCH  /ca▌");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);

    app.search = String::new();
    app.focus = Focus::Help(0);
    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    assert!(status.starts_with(" HELP") && !status.contains("? help"), "{status}");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);
}

#[test]
fn the_message_is_centred_in_accent_and_a_refusal_in_alert_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.message = Some("reloaded".to_string());

    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    let start = status.find("reloaded").unwrap();
    assert_eq!(start, (WIDTH as usize - "reloaded".len()) / 2, "{status}");
    assert!(status.ends_with("? help"), "{status}");
    assert_eq!(buffer[(start as u16, 4)].fg, MOCHA.accent);

    app.refused = true;
    assert_eq!(render_in(&app, WIDTH, 5)[(start as u16, 4)].fg, MOCHA.alert);
}

#[test]
fn the_mode_and_filters_stay_on_the_left_with_no_mode_keys_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.filter = Some("+rent".to_string());
    app.search = "pay".to_string();

    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    assert!(status.starts_with(" LIST  +rent  /pay "), "{status}");
    assert!(!status.contains("edit") && !status.contains("done"), "{status}");
    assert_eq!((buffer[(1, 4)].bg, buffer[(1, 4)].fg), (MOCHA.accent, MOCHA.on_accent));
    assert!(buffer[(1, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(7, 4)].fg, buffer[(14, 4)].fg), (MOCHA.project, MOCHA.primary));

    app.focus = Focus::Panel;
    assert!(!status_of(&app).contains("filter"));
}

#[test]
fn the_keys_of_a_command_waiting_for_its_end_show_just_left_of_the_help_hint_in_mocha() {
    let key = |app: &mut App, code: KeyCode| app.handle_key(KeyEvent::from(code), TODAY);
    let mut app = app_of(&["Pay +rent"]);

    key(&mut app, KeyCode::Char('d'));
    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    assert!(status.ends_with(" d ? help"), "{status}");
    assert_eq!(buffer[(WIDTH - 9, 4)].fg, MOCHA.accent);

    key(&mut app, KeyCode::Char('j'));
    assert!(!status_of(&app).contains(" d "));
}
