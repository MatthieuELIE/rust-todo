use super::super::tests::{MOCHA, app_of, app_with_every_entry, glyph_cells, panel_rows, render_in};
use super::*;
use ratatui::style::Modifier;

#[test]
fn a_filter_gone_from_the_panel_marks_no_entry_in_mocha() {
    let mut app = app_of(&["Pay +rent", "x 2026-09-20 Call +bank"]);
    app.filter = Some("+bank".to_string());
    app.focus = Focus::Panel;

    let buffer = render_in(&app, 50, 8);
    assert_eq!(panel_rows(&buffer)[1], "  All tasks       1");
    assert!(panel_rows(&buffer).iter().all(|row| !row.starts_with('→')));
    assert_eq!(buffer[(6, 1)].bg, MOCHA.background);
}

#[test]
fn the_panel_puts_coloured_projects_and_contexts_under_grey_headers_and_drops_an_empty_section_in_mocha() {
    let buffer = render_in(&app_of(&["Pay +rent", "Call +bank @phone"]), 50, 11);

    assert_eq!(
        panel_rows(&buffer)[1..9],
        [
            "→ All tasks       2",
            "",
            " PROJECTS",
            "  +bank           1",
            "  +rent           1",
            "",
            " CONTEXTS",
            "  @phone          1"
        ]
    );
    assert_eq!((buffer[(2, 3)].fg, buffer[(2, 7)].fg), (MOCHA.secondary, MOCHA.secondary));
    assert!(buffer[(2, 3)].modifier.contains(Modifier::BOLD));
    assert_eq!(
        (buffer[(3, 4)].fg, buffer[(3, 8)].fg, buffer[(3, 1)].fg),
        (MOCHA.project, MOCHA.context, MOCHA.primary)
    );
    assert_eq!(buffer[(19, 4)].fg, MOCHA.tertiary);

    let waiting = panel_rows(&render_in(&app_of(&["Design wait:figma +app"]), 50, 11));
    assert_eq!(waiting[1..5], ["→ All tasks       1", "  Waiting         1", "", " PROJECTS"]);

    let contexts_only = panel_rows(&render_in(&app_of(&["Call @phone"]), 50, 11));
    assert_eq!(contexts_only[3], " CONTEXTS");
    assert!(!contexts_only.iter().any(|row| row.contains("PROJECTS")));
}

#[test]
fn the_due_entry_is_red_with_an_overdue_task_and_yellow_with_tasks_of_the_day_only_in_mocha() {
    let late = render_in(&app_of(&["Pay rent due:2026-09-26", "Call due:2026-09-01"]), 50, 11);
    let today = render_in(&app_of(&["Pay rent due:2026-09-26"]), 50, 11);

    assert_eq!(panel_rows(&late)[1..3], ["→ All tasks       2", "  Due             2"]);
    assert_eq!((late[(3, 2)].fg, today[(3, 2)].fg), (MOCHA.alert, MOCHA.due_today));

    let mut picked = app_of(&["Pay rent due:2026-09-26", "Call due:2026-09-01"]);
    picked.filter = Some(DUE_NOW.to_string());
    let picked = render_in(&picked, 50, 11);
    assert_eq!(panel_rows(&picked)[2], "→ Due             2");
    assert_eq!(picked[(3, 2)].fg, MOCHA.alert);
}

#[test]
fn the_panel_cuts_a_long_name_at_its_column() {
    let buffer = render_in(&app_of(&["Pay +abcdefghijklmnopqrst"]), 50, 8);

    assert!(panel_rows(&buffer).contains(&"  +abcdefghijkl   1".to_string()));
}

#[test]
fn with_icons_the_section_headers_icons_stand_in_the_column_of_the_entries_icons() {
    let mut app = app_with_every_entry();
    app.icons = true;

    let found = glyph_cells(&render_in(&app, 109, 36));

    let column = |glyph| found.iter().find(|(found, ..)| *found == glyph).unwrap().1;
    for header in ['\u{f07b}', '\u{f1fa}'] {
        assert_eq!(column(header), column('\u{f0ca}'), "{header:?}");
    }
}

#[test]
fn in_the_history_the_panel_icon_is_grey() {
    let mut app = app_with_every_entry();
    app.icons = true;
    app.in_history = true;

    let buffer = render_in(&app, 109, 36);

    let all = glyph_cells(&buffer).into_iter().find(|(glyph, ..)| *glyph == '\u{f0ca}').unwrap();
    assert_eq!(buffer[(all.1, all.2)].fg, MOCHA.secondary);
}

#[test]
fn icons_do_not_change_the_width_of_the_panel_entries() {
    let mut app = app_with_every_entry();
    let plain = panel_rows(&render_in(&app, 109, 36));
    app.icons = true;

    let iconed = panel_rows(&render_in(&app, 109, 36));

    let entries = |rows: &[String]| {
        rows.iter()
            .filter(|row| row.ends_with(char::is_numeric))
            .map(|row| row.chars().count())
            .collect::<Vec<_>>()
    };
    assert_eq!(entries(&plain), entries(&iconed));
    assert!(entries(&plain).len() >= 5);
}
