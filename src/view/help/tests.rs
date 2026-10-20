use super::super::tests::{MOCHA, app_of, render_in, rows};
use crate::tui::Focus;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;

#[test]
fn the_help_shows_one_column_of_modes_under_their_names_keys_bold_actions_and_slashes_greyed_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(0);

    let buffer = render_in(&app, 109, 36);
    let rows = rows(&buffer);
    let top = rows.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    let y = top + 2;

    assert_eq!(
        rows[top..]
            .iter()
            .position(|row| row.contains('╯') && row.contains("esc close"))
            .expect("bottom")
            + 1,
        30
    );
    let corner = rows[top].chars().position(|c| c == '╭').expect("corner") as u16;
    assert_eq!((corner, buffer[(corner, top as u16)].fg), (32, MOCHA.accent));
    assert!(rows[top + 29].contains("─ j/k scroll · esc close ╯"), "{}", rows[top + 29]);
    assert!(rows[y].contains(" LIST "), "{}", rows[y]);
    assert!(rows[y + 1].contains("j/k/↓/↑      move"), "{}", rows[y + 1]);
    assert!(rows[y + 21].chars().skip(33).take(44).all(|c| c == ' ' || c == '▐'), "{}", rows[y + 21]);
    assert!(rows[y + 22].contains(" PANEL "), "{}", rows[y + 22]);
    assert!(rows[y + 25].contains("Tab/Enter    back to the list"), "{}", rows[y + 25]);
    assert!(!rows.iter().any(|row| row.contains(" DATE ")));
    assert!(rows.iter().any(|row| row.contains("▐")));

    let y = y as u16;
    assert_eq!(buffer[(36, y)].fg, MOCHA.secondary);
    assert!(buffer[(36, y)].modifier.contains(Modifier::BOLD));
    let (key, slash, action) = (&buffer[(36, y + 1)], &buffer[(37, y + 1)], &buffer[(49, y + 1)]);
    assert!(key.fg == MOCHA.primary && key.modifier.contains(Modifier::BOLD));
    assert!(slash.fg == MOCHA.separator && !slash.modifier.contains(Modifier::BOLD));
    assert!(action.fg == MOCHA.tertiary && !action.modifier.contains(Modifier::BOLD));
}

#[test]
fn the_help_scrolled_to_the_bottom_ends_on_the_date_keys_and_a_low_screen_shrinks_the_box() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(u16::MAX);

    let rows_tall = rows(&render_in(&app, 109, 36));
    assert!(rows_tall.iter().any(|row| row.contains("Enter/Esc    pick, close")));
    assert!(rows_tall.iter().any(|row| row.contains(" DATE ")));

    let rows_low = rows(&render_in(&app, 109, 20));
    let top = rows_low.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    assert_eq!(top, 0);
    assert!(rows_low[18].contains("─ j/k scroll · esc close ╯"), "{}", rows_low[18]);
    assert!(rows_low[16].contains("Enter/Esc    pick, close"), "{}", rows_low[16]);
}

#[test]
fn the_help_has_a_scrollbar_in_its_right_margin_when_its_lines_overflow_the_box_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(0);

    let bar = |buffer: &Buffer| {
        buffer
            .content
            .iter()
            .filter(|cell| cell.symbol() == "▐")
            .map(|cell| cell.fg)
            .collect::<Vec<_>>()
    };
    let low = bar(&render_in(&app, 109, 20));
    assert!(low.contains(&MOCHA.tertiary) && low.contains(&MOCHA.separator));

    app.focus = Focus::Help(u16::MAX);
    let buffer = render_in(&app, 109, 20);
    let thumb = (0..20)
        .filter(|&y| buffer[(75, y)].symbol() == "▐" && buffer[(75, y)].fg == MOCHA.tertiary)
        .max();
    assert_eq!(thumb, Some(16));
}

#[test]
fn the_help_says_tab_completes_wait_values_too() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(10);

    assert!(
        rows(&render_in(&app, 109, 36))
            .iter()
            .any(|row| row.contains("Tab          complete +, @ or wait:  "))
    );
}
