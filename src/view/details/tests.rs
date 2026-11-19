use super::super::tests::{MOCHA, app_of, fold_all, list_rows, render_in, rows};
use ratatui::style::Modifier;

const DEMO: [&str; 11] = [
    "(A) 2026-09-17 Send the invoices +finance +finance-tax @desk due:2026-09-23",
    "(A) 2026-09-17 Call the accountant +finance-tax @phone due:2026-09-26",
    "(B) 2026-09-17 Book the offsite venue +offsite @phone due:2026-09-30",
    "(B) 2026-09-17 Review the pull request for the export command +work @computer",
    "(C) 2026-09-17 Draft the agenda for the offsite +offsite @desk due:2026-10-08",
    "(C) 2026-09-17 Reply about the new logo +work @computer wait:designer",
    "2026-09-17 Write the release notes for the next version, listing every change since the last tag, the breaking ones first, then the fixes and the documentation updates, and thank the people who reported issues +work @computer",
    "2026-09-17 Buy a birthday present @errands",
    "2026-09-17 Water the plants @home",
    "2026-09-17 Renew the gym membership @errands due:2026-10-08",
    "2026-09-17 Fix the squeaky door @home",
];

/// How many of the eleven demo tasks have their number on screen.
fn demo_tasks_on_screen(rows: &[String]) -> usize {
    (1..=11)
        .filter(|number| rows.iter().any(|row| row.contains(&super::super::number(*number))))
        .count()
}

#[test]
fn at_80_by_24_the_details_hide_and_the_list_shows_the_eleven_demo_tasks_and_their_four_groups() {
    let rows = list_rows(&render_in(&app_of(&DEMO), 80, 24));

    assert!(rows.iter().all(|row| !row.contains("DETAILS")));
    assert_eq!(demo_tasks_on_screen(&rows), 11);
    assert_eq!(rows.iter().filter(|row| row.contains("PRIORITY")).count(), 4);
}

#[test]
fn at_80_by_30_a_long_text_grows_the_details_only_into_the_row_the_list_leaves_empty_and_ends_on_an_ellipsis() {
    let mut app = app_of(&DEMO);
    app.cursor = 6;

    let rows = list_rows(&render_in(&app, 80, 30));

    assert_eq!(demo_tasks_on_screen(&rows), 11);
    assert!(rows[19].starts_with('─'), "{}", rows[19]);
    assert!(rows[21].starts_with(" DETAILS"), "{}", rows[21]);
    assert!(rows[24].starts_with("           ") && rows[24].ends_with('…'), "{}", rows[24]);
    assert!(rows[25].starts_with(" Created"), "{}", rows[25]);
}

#[test]
fn at_109_by_36_the_long_text_of_the_demo_shows_whole_in_the_details() {
    let mut app = app_of(&DEMO);
    app.cursor = 6;

    let rows = list_rows(&render_in(&app, 109, 36));

    assert_eq!(demo_tasks_on_screen(&rows), 11);
    let text: Vec<&str> = rows
        .iter()
        .skip_while(|row| !row.starts_with(" Text"))
        .take(4)
        .map(|row| row[10..].trim())
        .collect();
    assert_eq!(
        text.join(" "),
        "Write the release notes for the next version, listing every change since the last tag, the breaking ones first, then the fixes and the documentation updates, and thank the people who reported issues +work @computer"
    );
}

#[test]
fn the_details_card_under_the_list_shows_the_task_under_the_cursor_in_mocha() {
    let app = app_of(&["(A) 2026-09-01 Appeler +client pour le devis @work wait:figma"]);

    let buffer = render_in(&app, 75, 32);

    assert_eq!(
        list_rows(&buffer)[23..30],
        [
            String::new(),
            format!(" DETAILS {}", "─".repeat(41)),
            " Priority   A".to_string(),
            " Text      Appeler +client pour le devis @work".to_string(),
            format!("{:<25} Due", " Created   2026-09-01"),
            format!("{:<25} Contexts  @work", " Projects  +client"),
            " wait:figma".to_string(),
        ]
    );
    assert_eq!(rows(&buffer)[30].chars().nth(23), Some('╰'));
    assert_eq!(
        (buffer[(29, 24)].fg, buffer[(49, 24)].fg, buffer[(23, 24)].fg, buffer[(25, 25)].fg),
        (MOCHA.secondary, MOCHA.structure, MOCHA.structure, MOCHA.label)
    );
    assert!(buffer[(29, 24)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(36, 25)].fg, buffer[(35, 25)].bg), (MOCHA.priorities[0], MOCHA.badge));
    assert_eq!(
        (buffer[(43, 26)].fg, buffer[(35, 28)].fg, buffer[(60, 28)].fg),
        (MOCHA.project, MOCHA.project, MOCHA.context)
    );
    assert_eq!(buffer[(35, 26)].fg, MOCHA.primary);
}

#[test]
fn a_project_ending_the_description_is_shown_coloured_in_the_text_row_in_mocha() {
    let buffer = render_in(&app_of(&["Appeler pour le devis +client"]), 75, 32);

    assert_eq!(list_rows(&buffer)[26], " Text      Appeler pour le devis +client");
    assert_eq!((buffer[(35, 26)].fg, buffer[(57, 26)].fg), (MOCHA.primary, MOCHA.project));
}

#[test]
fn the_details_show_from_a_screen_of_30_rows_and_hide_below_it() {
    let app = app_of(&["Pay rent"]);

    let buffer = render_in(&app, 75, 30);
    assert!(list_rows(&buffer)[20].starts_with('─'));
    assert_eq!(list_rows(&buffer)[22], format!(" DETAILS {}", "─".repeat(41)));
    let buffer = render_in(&app, 75, 29);
    assert!(rows(&buffer).iter().all(|row| !row.contains("DETAILS")));
    assert!(list_rows(&buffer)[27].starts_with('─'));
}

#[test]
fn a_text_too_long_for_its_place_wraps_under_the_value_and_grows_the_card_in_mocha() {
    let app = app_of(&["Appeler le client pour le devis du printemps prochain +client"]);

    let buffer = render_in(&app, 75, 32);

    assert_eq!(
        list_rows(&buffer)[23..29],
        [
            format!(" DETAILS {}", "─".repeat(41)),
            " Priority".to_string(),
            " Text      Appeler le client pour le devis du".to_string(),
            "           printemps prochain +client".to_string(),
            format!("{:<25} Due", " Created"),
            format!("{:<25} Contexts", " Projects  +client"),
        ]
    );
    assert_eq!(rows(&buffer)[30].chars().nth(23), Some('╰'));
    assert_eq!((buffer[(54, 26)].fg, buffer[(35, 26)].fg), (MOCHA.project, MOCHA.primary));
}

#[test]
fn a_short_text_keeps_the_card_of_seven_rows_and_the_list_its_height() {
    let buffer = render_in(&app_of(&["Pay rent"]), 75, 32);

    assert!(list_rows(&buffer)[24].starts_with(" DETAILS"));
    assert_eq!(rows(&buffer)[30].chars().nth(23), Some('╰'));
}

#[test]
fn a_text_too_long_for_the_one_row_the_list_leaves_empty_takes_it_and_ends_its_last_row_with_an_ellipsis() {
    let text = "A rather long description that will not fit here at all because it is far too long for the card";
    let lines: Vec<&str> = std::iter::once(text).chain(["Pay rent"; 17]).collect();

    let buffer = render_in(&app_of(&lines), 65, 30);

    assert!(list_rows(&buffer)[19].starts_with('─'));
    assert!(list_rows(&buffer)[21].starts_with(" DETAILS"));
    assert_eq!(list_rows(&buffer)[23], " Text      A rather long description");
    assert!(list_rows(&buffer)[24].ends_with('…'));
}

#[test]
fn a_text_too_long_for_a_list_that_leaves_no_row_empty_keeps_the_card_of_seven_rows_with_its_text_on_one() {
    let text = "A rather long description that will not fit here at all because it is far too long for the card";
    let lines: Vec<&str> = std::iter::once(text).chain(["Pay rent"; 20]).collect();

    let rows = list_rows(&render_in(&app_of(&lines), 75, 30));

    assert!(rows[22].starts_with(" DETAILS"), "{}", rows[22]);
    assert!(
        rows[24].starts_with(" Text      A rather long") && rows[24].ends_with('…'),
        "{}",
        rows[24]
    );
    assert!(rows[25].starts_with(" Created"), "{}", rows[25]);
}

#[test]
fn the_details_stay_empty_when_the_cursor_is_on_no_task() {
    let mut app = app_of(&["(A) a"]);
    fold_all(&mut app);

    assert_eq!(
        list_rows(&render_in(&app, 75, 30))[22..28],
        [
            format!(" DETAILS {}", "─".repeat(41)),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new()
        ]
    );
}

#[test]
fn the_details_show_the_due_date_beside_the_creation_date_and_the_other_key_values_on_the_last_row_in_mocha() {
    let app = app_of(&["2026-09-01 Ship it due:2026-10-15 wait:figma rec:1w"]);

    let buffer = render_in(&app, 75, 32);
    let zone = &list_rows(&buffer)[25..30];

    assert_eq!(zone[1], " Text      Ship it");
    assert_eq!(zone[2], format!("{:<25} Due       2026-10-15", " Created   2026-09-01"));
    assert_eq!(zone[4], " wait:figma  rec:1w");
    assert_eq!((buffer[(25, 29)].fg, buffer[(30, 29)].fg), (MOCHA.tertiary, MOCHA.primary));
}

#[test]
fn a_detail_row_as_wide_as_the_card_is_whole_and_a_wider_one_ends_on_an_ellipsis_inside_the_card() {
    let fits = format!("Ship it wait:{}", "a".repeat(44));
    let over = format!("Ship it wait:{}", "a".repeat(45));

    assert_eq!(list_rows(&render_in(&app_of(&[&fits]), 75, 32))[29], format!(" wait:{}", "a".repeat(44)));
    let buffer = render_in(&app_of(&[&over]), 75, 32);
    assert_eq!(list_rows(&buffer)[29], format!(" wait:{}…", "a".repeat(43)));
    assert_eq!(rows(&buffer)[29].chars().last(), Some('│'));
}

#[test]
fn the_due_date_in_the_details_is_bold_red_once_past_in_mocha() {
    let buffer = render_in(&app_of(&["Ship it due:2026-09-20 due:2026-10-30"]), 75, 32);

    assert_eq!(list_rows(&buffer)[27], format!("{:<25} Due       2026-09-20", " Created"));
    assert_eq!(buffer[(60, 27)].fg, MOCHA.alert);
    assert!(buffer[(60, 27)].modifier.contains(Modifier::BOLD));
}

#[test]
fn the_due_date_in_the_details_is_not_bold_on_the_day() {
    let buffer = render_in(&app_of(&["Ship it due:2026-09-26"]), 75, 32);

    assert_eq!(buffer[(60, 27)].fg, MOCHA.due_today);
    assert!(!buffer[(60, 27)].modifier.contains(Modifier::BOLD));
}
