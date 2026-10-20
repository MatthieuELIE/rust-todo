use super::super::tests::{MOCHA, app_of, fold_all, list_rows, render_in, rows};
use ratatui::style::Modifier;

#[test]
fn the_details_card_under_the_list_shows_the_task_under_the_cursor_in_mocha() {
    let app = app_of(&["(A) 2026-09-01 Appeler +client pour le devis @work wait:figma"]);

    let buffer = render_in(&app, 75, 18);

    assert_eq!(
        list_rows(&buffer)[9..16],
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
    assert_eq!(rows(&buffer)[16].chars().nth(23), Some('╰'));
    assert_eq!(
        (buffer[(29, 10)].fg, buffer[(49, 10)].fg, buffer[(23, 10)].fg, buffer[(25, 11)].fg),
        (MOCHA.secondary, MOCHA.structure, MOCHA.structure, MOCHA.label)
    );
    assert!(buffer[(29, 10)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(36, 11)].fg, buffer[(35, 11)].bg), (MOCHA.priorities[0], MOCHA.badge));
    assert_eq!(
        (buffer[(43, 12)].fg, buffer[(35, 14)].fg, buffer[(60, 14)].fg),
        (MOCHA.project, MOCHA.project, MOCHA.context)
    );
    assert_eq!(buffer[(35, 12)].fg, MOCHA.primary);
}

#[test]
fn a_project_ending_the_description_is_shown_coloured_in_the_text_row_in_mocha() {
    let buffer = render_in(&app_of(&["Appeler pour le devis +client"]), 75, 18);

    assert_eq!(list_rows(&buffer)[12], " Text      Appeler pour le devis +client");
    assert_eq!((buffer[(35, 12)].fg, buffer[(57, 12)].fg), (MOCHA.primary, MOCHA.project));
}

#[test]
fn the_details_hide_when_the_list_would_get_fewer_than_five_rows() {
    let app = app_of(&["Pay rent"]);

    let buffer = render_in(&app, 75, 16);
    assert_eq!(list_rows(&buffer)[1..6].iter().filter(|row| !row.is_empty()).count(), 1);
    assert!(list_rows(&buffer)[6].starts_with('─'));
    assert_eq!(list_rows(&buffer)[8], format!(" DETAILS {}", "─".repeat(41)));
    assert!(rows(&render_in(&app, 75, 15)).iter().all(|row| !row.contains("DETAILS")));
}

#[test]
fn a_text_too_long_for_its_place_wraps_under_the_value_and_grows_the_card_in_mocha() {
    let app = app_of(&["Appeler le client pour le devis du printemps prochain +client"]);

    let buffer = render_in(&app, 75, 18);

    assert_eq!(
        list_rows(&buffer)[9..15],
        [
            format!(" DETAILS {}", "─".repeat(41)),
            " Priority".to_string(),
            " Text      Appeler le client pour le devis du".to_string(),
            "           printemps prochain +client".to_string(),
            format!("{:<25} Due", " Created"),
            format!("{:<25} Contexts", " Projects  +client"),
        ]
    );
    assert_eq!(rows(&buffer)[16].chars().nth(23), Some('╰'));
    assert_eq!((buffer[(54, 12)].fg, buffer[(35, 12)].fg), (MOCHA.project, MOCHA.primary));
}

#[test]
fn a_short_text_keeps_the_card_of_seven_rows_and_the_list_its_height() {
    let buffer = render_in(&app_of(&["Pay rent"]), 75, 18);

    assert!(list_rows(&buffer)[10].starts_with(" DETAILS"));
    assert_eq!(rows(&buffer)[16].chars().nth(23), Some('╰'));
}

#[test]
fn a_text_too_long_for_a_low_screen_leaves_the_list_five_rows_and_ends_its_last_row_with_an_ellipsis() {
    let text = "A rather long description that will not fit here at all because it is far too long for the card";

    let buffer = render_in(&app_of(&[text]), 65, 17);

    assert!(list_rows(&buffer)[8].starts_with(" DETAILS"));
    assert_eq!(list_rows(&buffer)[10], " Text      A rather long description");
    assert!(list_rows(&buffer)[11].ends_with('…'));
}

#[test]
fn the_details_stay_empty_when_the_cursor_is_on_no_task() {
    let mut app = app_of(&["(A) a"]);
    fold_all(&mut app);

    assert_eq!(
        list_rows(&render_in(&app, 75, 16))[8..14],
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

    let buffer = render_in(&app, 75, 18);
    let zone = &list_rows(&buffer)[11..16];

    assert_eq!(zone[1], " Text      Ship it");
    assert_eq!(zone[2], format!("{:<25} Due       2026-10-15", " Created   2026-09-01"));
    assert_eq!(zone[4], " wait:figma  rec:1w");
    assert_eq!((buffer[(25, 15)].fg, buffer[(30, 15)].fg), (MOCHA.tertiary, MOCHA.primary));
}

#[test]
fn the_due_date_in_the_details_is_bold_red_once_past_in_mocha() {
    let buffer = render_in(&app_of(&["Ship it due:2026-09-20 due:2026-10-30"]), 75, 18);

    assert_eq!(list_rows(&buffer)[13], format!("{:<25} Due       2026-09-20", " Created"));
    assert_eq!(buffer[(60, 13)].fg, MOCHA.alert);
    assert!(buffer[(60, 13)].modifier.contains(Modifier::BOLD));
}

#[test]
fn the_due_date_in_the_details_is_not_bold_on_the_day() {
    let buffer = render_in(&app_of(&["Ship it due:2026-09-26"]), 75, 18);

    assert_eq!(buffer[(60, 13)].fg, MOCHA.due_today);
    assert!(!buffer[(60, 13)].modifier.contains(Modifier::BOLD));
}
