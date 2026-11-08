use super::super::tests::{MOCHA, app_of, render_in, rows};
use super::*;
use crate::toast::Notice;

const WIDTH: u16 = 109;

fn column_of(row: &str, needle: &str) -> u16 {
    let byte = row.find(needle).unwrap_or_else(|| panic!("{needle:?} missing from {row:?}"));
    row[..byte].chars().count() as u16
}

#[test]
fn a_success_is_painted_in_success_with_its_icon_when_icons_are_on_and_without_one_otherwise() {
    let mut app = app_of(&["Pay +rent"]);
    app.notify(Notice::Copied);

    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    let start = column_of(&status, "Task copied to the clipboard");
    assert_eq!(buffer[(start, 4)].fg, MOCHA.success);
    assert!(!status.contains('\u{f058}'), "{status}");

    app.icons = true;
    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    let icon = column_of(&status, "\u{f058} Task copied");
    assert_eq!(buffer[(icon, 4)].fg, MOCHA.success);
    assert_eq!(buffer[(icon + 2, 4)].fg, MOCHA.success);
}

#[test]
fn an_information_is_painted_in_secondary_with_its_icon_when_icons_are_on_and_without_one_otherwise() {
    let mut app = app_of(&["Pay +rent"]);
    app.notify(Notice::NothingDue);

    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    assert_eq!(buffer[(column_of(&status, "No task due"), 4)].fg, MOCHA.secondary);
    assert!(!status.contains('\u{f05a}'), "{status}");

    app.icons = true;
    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    let icon = column_of(&status, "\u{f05a} No task due");
    assert_eq!(buffer[(icon, 4)].fg, MOCHA.secondary);
}

#[test]
fn a_refusal_is_painted_in_alert_with_its_icon_when_icons_are_on_and_without_one_otherwise() {
    let mut app = app_of(&["Pay +rent"]);
    app.notify(Notice::HistoryReadOnly);

    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    assert_eq!(buffer[(column_of(&status, "History is read-only"), 4)].fg, MOCHA.alert);
    assert!(!status.contains('\u{f057}'), "{status}");

    app.icons = true;
    let buffer = render_in(&app, WIDTH, 5);
    let status = rows(&buffer)[4].clone();
    let icon = column_of(&status, "\u{f057} History is read-only");
    assert_eq!(buffer[(icon, 4)].fg, MOCHA.alert);
}

#[test]
fn a_text_longer_than_the_room_ends_with_an_ellipsis_and_one_that_fits_is_kept() {
    assert_eq!(cut("Task copied", 11), "Task copied");
    assert_eq!(cut("Task copied", 10), "Task copi…");
    assert_eq!(cut("Task copied", 1), "…");
}
