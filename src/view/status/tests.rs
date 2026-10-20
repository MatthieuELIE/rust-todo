use super::super::tests::{MOCHA, TODAY, app_of, render, render_in, rows};
use super::*;
use ratatui::style::Modifier;

#[test]
fn the_status_bar_shows_the_mode_and_filters_left_and_the_message_right_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.filter = Some("+rent".to_string());
    app.search = "pay".to_string();
    app.message = Some("reloaded".to_string());

    let buffer = render(&app);
    let status = rows(&buffer)[4].clone();
    assert!(status.starts_with(" LIST  +rent  /pay "), "{status}");
    assert!(status.ends_with(" reloaded"), "{status}");
    assert_eq!((buffer[(1, 4)].bg, buffer[(1, 4)].fg), (MOCHA.accent, MOCHA.on_accent));
    assert!(buffer[(1, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(7, 4)].bg, MOCHA.background);
    assert_eq!((buffer[(7, 4)].fg, buffer[(14, 4)].fg), (MOCHA.project, MOCHA.primary));
    assert_eq!(buffer[(45, 4)].fg, MOCHA.secondary);

    app.refused = true;
    assert_eq!(render(&app)[(45, 4)].fg, MOCHA.alert);
}

#[test]
fn the_mode_keys_follow_when_they_fit_whole_and_no_message_is_shown_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Panel;

    let buffer = render(&app);
    assert_eq!(rows(&buffer)[4], " PANEL  j/k filter · esc all tasks · tab back");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);
    assert!(buffer[(8, 4)].fg == MOCHA.primary && buffer[(8, 4)].modifier.contains(Modifier::BOLD));
    assert!(buffer[(12, 4)].fg == MOCHA.tertiary && !buffer[(12, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(19, 4)].fg, MOCHA.separator);

    app.message = Some("reloaded".to_string());
    assert!(!rows(&render(&app))[4].contains("j/k"));

    app.focus = Focus::List;
    app.message = None;
    assert_eq!(rows(&render(&app))[4], " LIST");
    assert_eq!(rows(&render_in(&app, 60, 5))[4], " LIST  ⏎ edit · o add · x done · p priority · ? help");
    app.in_history = true;
    assert_eq!(rows(&render_in(&app, 60, 5))[4], " LIST  r reopen · H back · ? help");
    app.in_history = false;

    app.focus = Focus::Search;
    app.search = "ca".to_string();
    let buffer = render(&app);
    assert_eq!(rows(&buffer)[4], " SEARCH  /ca▌  ⏎ keep · esc clear");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);
}

#[test]
fn the_keys_of_a_command_waiting_for_its_end_show_on_the_right_of_the_status_bar_in_mocha() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    let key = |app: &mut App, code: KeyCode| app.handle_key(KeyEvent::from(code), TODAY);
    let status = |app: &App| rows(&render_in(app, 70, 5))[4].clone();
    let mut app = app_of(&["Pay +rent"]);

    key(&mut app, KeyCode::Char('d'));
    assert!(status(&app).contains("? help") && status(&app).ends_with(" d"), "{}", status(&app));
    assert_eq!(render_in(&app, 70, 5)[(68, 4)].fg, MOCHA.secondary);
    let fits = |width| rows(&render_in(&app, width, 5))[4].contains("? help");
    assert!(!fits(54));
    assert!(fits(55));
    key(&mut app, KeyCode::Char('j'));
    assert!(status(&app).ends_with("? help"));

    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Char('c'));
    key(&mut app, KeyCode::Char('i'));
    assert!(status(&app).ends_with(" ci"), "{}", status(&app));
    key(&mut app, KeyCode::Char('w'));
    assert!(
        status(&app).ends_with("tab complete") || status(&app).ends_with("save"),
        "{}",
        status(&app)
    );
}
