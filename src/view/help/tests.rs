use super::super::tests::{MOCHA, app_of, render_in, rows};
use crate::tui::{App, Focus};
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;

fn help_app(focus: Focus) -> App {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(0);
    app.under_help = focus;
    app
}

fn slot(row: &str, column: usize) -> String {
    row.chars().skip(13 + 44 * column).take(40).collect()
}

fn header(rows: &[String], column: usize, name: &str) -> Option<usize> {
    rows.iter().position(|row| slot(row, column).trim() == name)
}

#[test]
fn the_help_draws_two_columns_on_a_wide_area_each_section_whole_in_its_own() {
    let buffer = render_in(&help_app(Focus::List), 109, 36);
    let rows = rows(&buffer);
    let top = rows.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    let slots = |column: usize| rows[top..].iter().map(|row| slot(row, column)).collect::<Vec<_>>();
    let (left, right) = (slots(0), slots(1));
    let at = |column: &[String], text: &str| column.iter().position(|row| row.trim() == text);
    let order = |column: &[String], names: &[&str]| {
        let found: Vec<usize> = names.iter().map(|name| at(column, name).expect(name)).collect();
        assert!(found.windows(2).all(|pair| pair[0] < pair[1]), "{names:?}");
    };

    order(
        &left,
        &[
            "LIST",
            "q            quit",
            "PANEL",
            "Tab/Enter    back to the list",
            "SEARCH",
            "Esc          clear the search",
        ],
    );
    order(
        &right,
        &[
            "INSERT",
            "↓/↑/Ctrl-n/p pick a completion",
            "NORMAL",
            "Esc          cancel",
            "DATE",
            "Enter/Esc    pick, close",
        ],
    );
    for name in ["INSERT", "NORMAL", "DATE"] {
        assert_eq!(at(&left, name), None, "{name}");
    }
    for name in ["LIST", "PANEL", "SEARCH"] {
        assert_eq!(at(&right, name), None, "{name}");
    }
}

#[test]
fn the_help_draws_one_column_in_the_order_of_help_on_a_narrow_area() {
    let rows = rows(&render_in(&help_app(Focus::List), 80, 36));
    let top = rows.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    let names = [" LIST ", " PANEL ", " SEARCH "];

    let at: Vec<usize> = names.iter().map(|name| rows.iter().position(|row| row.contains(name)).unwrap()).collect();
    assert!(top < at[0] && at[0] < at[1] && at[1] < at[2]);
    assert!(rows.iter().all(|row| !row.contains(" INSERT ")), "the first screen stops after SEARCH");
    let box_width = rows[top].chars().filter(|c| *c == '─').count() + 2 + " HELP ".chars().count();
    assert!(box_width <= 80, "{}", rows[top]);

    let mut app = help_app(Focus::List);
    app.focus = Focus::Help(u16::MAX);
    let rows = rows_of(&app, 80, 36);
    let order: Vec<usize> = [" INSERT ", " NORMAL ", " DATE "]
        .iter()
        .map(|name| rows.iter().position(|row| row.contains(name)).unwrap())
        .collect();
    assert!(order[0] < order[1] && order[1] < order[2]);
}

fn rows_of(app: &App, width: u16, height: u16) -> Vec<String> {
    rows(&render_in(app, width, height))
}

#[test]
fn the_help_lights_the_mode_it_was_opened_from_in_accent_and_the_others_in_structure_in_mocha() {
    let buffer = render_in(&help_app(Focus::Panel), 109, 36);
    let rows = rows(&buffer);
    let cell_of = |name: &str| {
        let column = usize::from(["INSERT", "NORMAL", "DATE"].contains(&name));
        let y = header(&rows, column, name).expect(name);
        buffer[(13 + 44 * column as u16, y as u16)].clone()
    };

    let lit = cell_of("PANEL");
    assert_eq!((lit.fg, lit.bg), (MOCHA.on_accent, MOCHA.accent));
    assert!(lit.modifier.contains(Modifier::BOLD));
    for name in ["LIST", "SEARCH", "INSERT", "NORMAL", "DATE"] {
        let cell = cell_of(name);
        assert_eq!((cell.fg, cell.bg), (MOCHA.primary, MOCHA.structure), "{name}");
        assert!(cell.modifier.contains(Modifier::BOLD), "{name}");
    }
}

#[test]
fn the_help_bottom_title_says_j_k_scroll_only_when_the_content_overflows() {
    let app = help_app(Focus::List);

    let tall = rows_of(&app, 109, 36);
    assert!(tall.iter().any(|row| row.contains("─ esc close ╯")), "two columns fit");
    assert!(tall.iter().all(|row| !row.contains("j/k scroll")));
    assert!(tall.iter().all(|row| !row.contains('▐')));
    assert_eq!(app.help_max.get(), 0);

    let low = rows_of(&app, 109, 20);
    assert!(low.iter().any(|row| row.contains("─ j/k scroll · esc close ╯")));
    assert!(low.iter().any(|row| row.contains('▐')));
}

#[test]
fn the_help_tells_the_app_how_far_the_last_drawing_left_it_to_scroll() {
    let app = help_app(Focus::List);

    render_in(&app, 109, 20);
    assert_eq!(app.help_max.get(), 30 - 15);
    render_in(&app, 80, 36);
    assert_eq!(app.help_max.get(), 61 - 31);
    render_in(&app, 109, 36);
    assert_eq!(app.help_max.get(), 0);
}

#[test]
fn the_help_scrolled_to_the_bottom_ends_on_the_date_keys_and_a_low_screen_shrinks_the_box() {
    let mut app = help_app(Focus::List);
    app.focus = Focus::Help(u16::MAX);

    let rows_low = rows_of(&app, 109, 20);
    let top = rows_low.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    assert_eq!(top, 0);
    assert!(rows_low[18].contains("─ j/k scroll · esc close ╯"), "{}", rows_low[18]);
    assert!(rows_low[16].contains("Enter/Esc    pick, close"), "{}", rows_low[16]);
}

#[test]
fn the_help_has_a_scrollbar_in_its_right_margin_when_its_lines_overflow_the_box_in_mocha() {
    let mut app = help_app(Focus::List);

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
        .filter(|&y| buffer[(98, y)].symbol() == "▐" && buffer[(98, y)].fg == MOCHA.tertiary)
        .max();
    assert_eq!(thumb, Some(16));
}

#[test]
fn the_help_keys_are_bold_their_slashes_greyed_and_their_actions_tertiary_in_mocha() {
    let buffer = render_in(&help_app(Focus::List), 109, 36);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains("j/k/↓/↑      move")).expect("line") as u16;
    let x = rows[y as usize].find("j/k").map(|byte| rows[y as usize][..byte].chars().count()).unwrap() as u16;

    let (key, slash, action) = (&buffer[(x, y)], &buffer[(x + 1, y)], &buffer[(x + 13, y)]);
    assert!(key.fg == MOCHA.primary && key.modifier.contains(Modifier::BOLD));
    assert!(slash.fg == MOCHA.separator && !slash.modifier.contains(Modifier::BOLD));
    assert!(action.fg == MOCHA.tertiary && !action.modifier.contains(Modifier::BOLD));
}

#[test]
fn the_help_says_tab_completes_wait_values_too() {
    assert!(
        rows_of(&help_app(Focus::List), 109, 36)
            .iter()
            .any(|row| row.contains("Tab          complete +, @ or wait:"))
    );
}

#[test]
fn no_line_of_help_is_wider_than_a_column() {
    assert!(super::super::HELP.lines().all(|line| line.chars().count() <= 40));
}
