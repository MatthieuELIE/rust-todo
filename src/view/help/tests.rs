use super::super::tests::{MOCHA, TODAY, add_popup, app_of, render_in, rows};
use crate::editor::Mode;
use crate::tui::{App, Focus};
use ratatui::buffer::{Buffer, Cell};
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
    let buffer = render_in(&help_app(Focus::List), 109, 40);
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
        &["INSERT", "↓/↑/Ctrl-n/p pick a completion", "NORMAL", "Esc          cancel", "VISUAL"],
    );
    let mut scrolled = help_app(Focus::List);
    scrolled.focus = Focus::Help(u16::MAX);
    let bottom = rows_of(&scrolled, 109, 40);
    let top = bottom.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    let right: Vec<String> = bottom[top..].iter().map(|row| slot(row, 1)).collect();
    order(&right, &["VISUAL", "y            copy the selection", "DATE", "Enter/Esc    pick, close"]);
    for name in ["INSERT", "NORMAL", "VISUAL", "DATE"] {
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

    let titled = |rows: &[String], name: &str| {
        rows.iter()
            .position(|row| row.chars().skip(20).take(40).collect::<String>().trim() == name)
    };
    let order = |names: [&str; 2]| {
        let seen = (0..=80)
            .filter(|&scroll| {
                let mut app = help_app(Focus::List);
                app.focus = Focus::Help(scroll);
                let rows = rows_of(&app, 80, 36);
                match (titled(&rows, names[0]), titled(&rows, names[1])) {
                    (Some(first), Some(second)) => {
                        assert!(first < second, "{names:?}");
                        true
                    }
                    _ => false,
                }
            })
            .count();
        assert!(seen > 0, "{names:?} never show together");
    };
    order(["INSERT", "NORMAL"]);
    order(["NORMAL", "VISUAL"]);
    order(["VISUAL", "DATE"]);
}

fn rows_of(app: &App, width: u16, height: u16) -> Vec<String> {
    rows(&render_in(app, width, height))
}

fn box_width(rows: &[String]) -> usize {
    let top = rows.iter().find(|row| row.contains("╭ HELP ─")).expect("box");
    let from = top.split("╭ HELP ─").next().expect("row").chars().count();
    top.chars().skip(from).position(|c| c == '╮').expect("corner") + 1
}

#[test]
fn the_help_needs_92_columns_for_its_two_columns() {
    let app = help_app(Focus::List);

    assert_eq!(box_width(&rows_of(&app, 91, 36)), 48);
    assert_eq!(box_width(&rows_of(&app, 92, 36)), 92);
}

#[test]
fn the_help_box_stops_at_its_tallest_on_a_taller_screen_and_scrolls_the_rest() {
    let rows = rows_of(&help_app(Focus::List), 109, 40);
    let top = rows.iter().position(|row| row.contains("╭ HELP ─")).expect("box");
    let bottom = rows.iter().position(|row| row.contains("─ j/k scroll · esc close ╯")).expect("bottom");

    assert_eq!(bottom - top + 1, 32 + 4);
}

fn header_cell(app: &mut App, name: &str) -> Cell {
    let column = usize::from(["INSERT", "NORMAL", "VISUAL", "DATE"].contains(&name));
    for scroll in [0, u16::MAX] {
        app.focus = Focus::Help(scroll);
        let buffer = render_in(app, 109, 36);
        if let Some(y) = header(&rows(&buffer), column, name) {
            return buffer[(13 + 44 * column as u16, y as u16)].clone();
        }
    }
    panic!("{name} is in no screen of the help");
}

#[test]
fn the_help_lights_the_mode_it_was_opened_from_in_accent_and_the_others_in_structure_in_mocha() {
    let mut app = help_app(Focus::Panel);
    let mut cell_of = |name: &str| header_cell(&mut app, name);

    let lit = cell_of("PANEL");
    assert_eq!((lit.fg, lit.bg), (MOCHA.on_accent, MOCHA.accent));
    assert!(lit.modifier.contains(Modifier::BOLD));
    for name in ["LIST", "SEARCH", "INSERT", "NORMAL", "VISUAL", "DATE"] {
        let cell = cell_of(name);
        assert_eq!((cell.fg, cell.bg), (MOCHA.primary, MOCHA.structure), "{name}");
        assert!(cell.modifier.contains(Modifier::BOLD), "{name}");
    }
}

#[test]
fn the_help_opened_from_the_popup_lights_normal_visual_or_date_when_the_date_picker_is_open() {
    let lit = |mode: Mode, picker: bool| {
        let mut app = app_of(&["Pay +rent"]);
        add_popup(&mut app, "Ship due:", 0);
        if let Focus::Popup(popup) = &mut app.focus {
            popup.editor.mode = mode;
            popup.picker = picker.then_some(TODAY);
        }
        let mut app = help_app(std::mem::take(&mut app.focus));
        ["INSERT", "NORMAL", "VISUAL", "DATE"]
            .into_iter()
            .filter(|name| header_cell(&mut app, name).bg == MOCHA.accent)
            .collect::<Vec<_>>()
    };

    assert_eq!(lit(Mode::Normal, false), ["NORMAL"]);
    assert_eq!(lit(Mode::Visual, false), ["VISUAL"]);
    assert_eq!(lit(Mode::Normal, true), ["DATE"]);
}

#[test]
fn the_help_bottom_title_says_j_k_scroll_only_when_the_content_overflows() {
    let app = help_app(Focus::List);

    let tall = rows_of(&app, 109, 40);
    assert!(tall.iter().any(|row| row.contains("─ j/k scroll · esc close ╯")), "two columns overflow");
    assert!(tall.iter().any(|row| row.contains('▐')));
    assert_eq!(app.help_max.get(), 43 - 32);

    let low = rows_of(&app, 109, 20);
    assert!(low.iter().any(|row| row.contains("─ j/k scroll · esc close ╯")));
    assert!(low.iter().any(|row| row.contains('▐')));
}

#[test]
fn the_help_tells_the_app_how_far_the_last_drawing_left_it_to_scroll() {
    let app = help_app(Focus::List);

    render_in(&app, 109, 20);
    assert_eq!(app.help_max.get(), 43 - 15);
    render_in(&app, 80, 36);
    assert_eq!(app.help_max.get(), 75 - 31);
    render_in(&app, 109, 40);
    assert_eq!(app.help_max.get(), 43 - 32);
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
fn the_help_scrollbar_thumb_is_the_share_of_the_lines_the_box_shows() {
    let buffer = render_in(&help_app(Focus::List), 109, 25);
    let thumb = (0..25)
        .filter(|&y| buffer[(98, y)].symbol() == "▐" && buffer[(98, y)].fg == MOCHA.tertiary)
        .count();

    assert_eq!(thumb, 9);
}

#[test]
fn the_help_keys_are_bold_their_slashes_greyed_and_their_actions_tertiary_in_mocha() {
    let buffer = render_in(&help_app(Focus::List), 109, 40);
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
        rows_of(&help_app(Focus::List), 109, 40)
            .iter()
            .any(|row| row.contains("Tab          complete +, @ or wait:"))
    );
}

#[test]
fn the_normal_section_lists_y_and_gives_the_word_objects_after_d_c_and_y() {
    let rows = rows_of(&help_app(Focus::List), 109, 40);
    let has = |text: &str| rows.iter().any(|row| row.contains(text));

    assert!(has("y + move     copy up to"));
    assert!(has("yy           copy the line"));
    assert!(has("iw/aw/iW/aW  after d/c/y: a word"));
    assert!(has("v            visual mode"));
}

#[test]
fn the_visual_section_lists_the_motions_y_the_way_out_and_enter_after_normal() {
    let mut app = help_app(Focus::List);
    app.focus = Focus::Help(u16::MAX);
    let rows = rows_of(&app, 109, 40);
    let column: Vec<String> = rows.iter().map(|row| slot(row, 1)).collect();
    let start = header(&rows, 1, "VISUAL").expect("VISUAL");

    let section: Vec<&str> = column[start + 1..]
        .iter()
        .map(|row| row.trim_end())
        .take_while(|row| !row.is_empty())
        .collect();
    assert_eq!(
        section,
        [
            "h/l/0/$      move",
            "w/b/e        word",
            "W/B/E        blank-separated word",
            "f/F/t/T x    to, before a char",
            ";/,          repeat, reverse",
            "y            copy the selection",
            "Esc/v        normal mode",
            "Enter        save",
        ]
    );
    assert!(header(&rows, 1, "NORMAL").is_none_or(|normal| normal < start));
}

#[test]
fn the_question_mark_line_of_list_names_the_modes_whose_keys_the_help_shows_too() {
    let rows = rows_of(&help_app(Focus::List), 109, 40);
    let line = rows.iter().position(|row| slot(row, 0).starts_with("?            help;")).expect("line");

    assert_eq!(slot(&rows[line], 0).trim_end(), "?            help; PANEL, NORMAL,");
    assert_eq!(slot(&rows[line + 1], 0).trim_end(), "             VISUAL, DATE too");
}

#[test]
fn no_line_of_help_is_wider_than_a_column() {
    assert!(super::super::HELP.lines().all(|line| line.chars().count() <= 40));
}
