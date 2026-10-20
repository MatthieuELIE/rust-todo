use super::super::tests::{MOCHA, TODAY, add_popup, app_of, render, render_in, render_with_cursor, rows};
use crate::editor::{Editor, Mode};
use crate::tui::{App, DUE_NOW, Focus, Popup, Target, WAITING};
use ratatui::layout::Position;
use ratatui::style::Modifier;
use time::macros::date;

#[test]
fn the_popup_wraps_its_text_under_its_title_with_the_terminal_cursor_on_its_cell_while_the_status_bar_says_insert_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.filter = Some("+rent".to_string());
    let mut editor = Editor::default();
    editor.text = "Call the bank about the loan and ask for a quote".to_string();
    editor.cursor = 5;
    app.focus = Focus::Popup(Popup {
        editor,
        target: Target::Add,
        selected: 0,
        picker: None,
    });

    let (buffer, cursor) = render_with_cursor(&app, 50, 5);
    let rows = rows(&buffer);

    assert!(rows[0].contains("╭ ADD (+rent) ───"), "{}", rows[0]);
    assert!(rows[1].contains("│Call the bank about the loan and ask  │"), "{}", rows[1]);
    assert!(rows[2].contains("│for a quote                           │"), "{}", rows[2]);
    assert!(rows[3].contains("╰───"), "{}", rows[3]);
    let corner = rows[0].split("╭ ADD").next().expect("row").chars().count() as u16;
    assert_eq!((buffer[(corner, 0)].fg, buffer[(corner + 1, 1)].bg), (MOCHA.accent, MOCHA.raised));
    assert!(buffer[(corner + 2, 0)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(corner + 7, 0)].fg, buffer[(corner + 2, 0)].fg), (MOCHA.project, MOCHA.primary));
    assert_eq!(buffer[(11, 1)].symbol(), "t");
    assert_eq!(cursor, Some(Position::new(11, 1)));
    assert!(buffer.content.iter().all(|cell| !cell.modifier.contains(Modifier::REVERSED)));
    assert_eq!(render_with_cursor(&app_of(&["Pay +rent"]), 50, 5).1, None);
    assert_eq!(rows[4], " INSERT  +rent  esc normal · ⏎ save");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);

    if let Focus::Popup(popup) = &mut app.focus {
        popup.editor.mode = Mode::Normal;
    }
    let buffer = render_in(&app, 70, 5);
    assert_eq!(self::rows(&buffer)[4], " NORMAL  +rent  i insert · p priority · ⏎ save · esc cancel");
    assert_eq!(buffer[(1, 4)].bg, MOCHA.accent);
}

#[test]
fn the_popup_colours_its_text_as_the_list_colours_a_task_in_mocha() {
    let mut app = app_of(&["Pay rent"]);
    add_popup(&mut app, "(A) 2026-09-01 Call +bank @phone due:2026-09-20 wait:x", 0);

    let (buffer, cursor) = render_with_cursor(&app, 80, 7);
    let row = rows(&buffer).iter().position(|row| row.contains("(A) 2026")).expect("popup line");
    let x = rows(&buffer)[row].split("(A)").next().expect("row").chars().count() as u16;
    let at = |offset: u16| &buffer[(x + offset, row as u16)];

    assert_eq!(
        (at(1).fg, at(4).fg, at(20).fg, at(26).fg),
        (MOCHA.priorities[0], MOCHA.tertiary, MOCHA.project, MOCHA.context)
    );
    assert!(at(1).modifier.contains(Modifier::BOLD));
    assert_eq!((at(15).fg, at(33).fg, at(48).fg), (MOCHA.primary, MOCHA.alert, MOCHA.tertiary));
    assert_eq!(cursor, Some(Position::new(x + 54, row as u16)));
}

#[test]
fn the_popup_keeps_its_cursor_on_screen_wherever_the_line_wraps() {
    let mut app = app_of(&["Pay rent"]);
    let cursor_at = |app: &mut App, text: &str, cursor: usize| {
        add_popup(app, text, 0);
        if let Focus::Popup(popup) = &mut app.focus {
            popup.editor.cursor = cursor;
        }
        render_with_cursor(app, 40, 12)
    };

    for length in 1..=64 {
        assert!(cursor_at(&mut app, &"a".repeat(length), length).1.is_some(), "{length} letters");
    }
    let words = "abcd ".repeat(13);
    for cursor in 0..=words.len() {
        assert!(cursor_at(&mut app, &words, cursor).1.is_some(), "words at {cursor}");
    }
    let (buffer, cursor) = cursor_at(&mut app, &"a".repeat(29), 29);
    let cursor = cursor.expect("cursor");
    assert_eq!(
        (buffer[(cursor.x - 1, cursor.y)].symbol(), buffer[(cursor.x + 1, cursor.y)].symbol()),
        ("a", "│")
    );
    assert!(
        buffer
            .content
            .iter()
            .all(|cell| !cell.modifier.intersects(Modifier::REVERSED | Modifier::HIDDEN))
    );

    let wide = format!("{} bbb", "日".repeat(14));
    let (buffer, cursor) = cursor_at(&mut app, &wide, 14);
    let cursor = cursor.expect("cursor");
    assert_eq!(buffer[(cursor.x - 2, cursor.y)].symbol(), "日");

    add_popup(&mut app, "abcdefghijklmnopqrstuvwx due:", 0);
    if let Focus::Popup(popup) = &mut app.focus {
        popup.picker = Some(TODAY);
    }
    assert!(rows(&render_in(&app, 40, 30)).iter().any(|row| row.contains("Mo Tu We")));
}

#[test]
fn the_completions_drop_down_under_the_tag_with_their_counts_and_the_pick_highlighted_in_mocha() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Read +books"]);
    add_popup(&mut app, "Buy +b", 1);

    let buffer = render_in(&app, 50, 12);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains("Buy +b")).expect("popup line");

    assert_eq!(
        (y + 1..y + 5).map(|y| cells(&rows[y], 9, 12)).collect::<Vec<_>>(),
        ["╭──────────╮", "│+bank    2│", "│+books   1│", "╰──────────╯"]
    );
    let y = y as u16;
    assert_eq!((buffer[(10, y + 2)].fg, buffer[(19, y + 2)].fg), (MOCHA.project, MOCHA.tertiary));
    assert_eq!((buffer[(10, y + 2)].bg, buffer[(10, y + 3)].bg), (MOCHA.raised, MOCHA.selected));
    assert_eq!(buffer[(9, y + 1)].fg, MOCHA.accent);
    assert_eq!(rows[11], " INSERT  esc normal · ⏎ save · tab complete");
}

#[test]
fn the_completion_pick_is_bold_and_the_others_are_not() {
    let mut app = app_of(&["Pay +bank", "Call +bank", "Read +books"]);
    add_popup(&mut app, "Buy +b", 1);

    let buffer = render_in(&app, 50, 12);
    let y = rows(&buffer).iter().position(|row| row.contains("Buy +b")).expect("popup line") as u16;

    assert!(buffer[(10, y + 3)].modifier.contains(Modifier::BOLD));
    assert!(!buffer[(10, y + 2)].modifier.contains(Modifier::BOLD));
}

#[test]
fn wait_completions_are_greyed_like_the_key_values_of_a_line_in_mocha() {
    let mut app = app_of(&["Design wait:figma"]);
    add_popup(&mut app, "Draw wait:", 0);

    let buffer = render_in(&app, 50, 12);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains("│wait:figma")).expect("drop-down row");
    let x = rows[y].split("│wait:figma").next().expect("row").chars().count() + 1;
    let (x, y) = (x as u16, y as u16);

    assert_eq!(buffer[(x, y)].fg, MOCHA.tertiary);
}

#[test]
fn the_completions_go_above_the_tag_when_there_is_no_room_below() {
    let mut app = app_of(&["Pay +bank", "Call @phone"]);
    add_popup(&mut app, "Call the bank about the loan and ask @ph", 0);

    let rows = rows(&render_in(&app, 50, 7));
    let y = rows.iter().position(|row| row.contains("│@ph ")).expect("tag line");

    assert_eq!(
        (y - 3..y).map(|y| cells(&rows[y], 5, 12)).collect::<Vec<_>>(),
        ["╭──────────╮", "│@phone   1│", "╰──────────╯"]
    );
}

#[test]
fn the_date_picker_drops_down_under_due_colon_as_a_month_from_monday_with_the_pick_in_the_accent_and_today_yellow_in_mocha() {
    let mut app = app_of(&["Pay rent"]);
    add_popup(&mut app, "Ship due:", 0);
    if let Focus::Popup(popup) = &mut app.focus {
        popup.picker = Some(date!(2026 - 09 - 28));
    }

    let buffer = render_in(&app, 60, 20);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains("│Mo Tu")).expect("weekdays row");
    let x = rows[y].split("│Mo Tu").next().expect("row").chars().count();

    assert_eq!(cells(&rows[y - 2], x + 1, 4), "due:");
    assert_eq!(
        (y - 1..y + 7).map(|y| cells(&rows[y], x, 22)).collect::<Vec<_>>(),
        [
            "╭ SEPTEMBER 2026 ────╮",
            "│Mo Tu We Th Fr Sa Su│",
            "│    1  2  3  4  5  6│",
            "│ 7  8  9 10 11 12 13│",
            "│14 15 16 17 18 19 20│",
            "│21 22 23 24 25 26 27│",
            "│28 29 30            │",
            "╰────────────────────╯",
        ]
    );
    let (x, y) = (x as u16, y as u16);
    assert_eq!(buffer[(x + 1, y)].fg, MOCHA.tertiary);
    let pick = &buffer[(x + 1, y + 5)];
    assert_eq!((pick.fg, pick.bg), (MOCHA.accent, MOCHA.selected));
    assert!(pick.modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(x + 4, y + 5)].bg, buffer[(x + 4, y + 5)].fg), (MOCHA.raised, MOCHA.primary));
    assert_eq!(buffer[(x, y - 1)].fg, MOCHA.accent);
    let popup = rows.iter().position(|row| row.contains("╭ ADD ")).expect("popup top");
    let corner = rows[popup].split("╭ ADD").next().expect("row").chars().count() as u16;
    assert_eq!(buffer[(corner, popup as u16)].fg, MOCHA.separator);
    assert_eq!(render_with_cursor(&app, 60, 20).1, None);
    assert_eq!(
        (buffer[(x + 16, y + 4)].fg, buffer[(x + 13, y + 4)].fg),
        (MOCHA.due_today, MOCHA.tertiary)
    );
    assert_eq!(rows[19], " DATE  hjkl move · H/L month · ⏎ pick · esc close");
    assert_eq!(buffer[(1, 19)].bg, MOCHA.accent);
}

#[test]
fn the_date_picker_stays_above_the_status_bar_on_a_small_screen() {
    let mut app = app_of(&["Pay rent"]);

    add_popup(&mut app, "Ship the parcel to the bank due:", 0);
    if let Focus::Popup(popup) = &mut app.focus {
        popup.picker = Some(TODAY);
    }
    let rows = rows(&render_in(&app, 30, 8));
    assert!(rows.iter().any(|row| row.contains("SEPTEMBER")));
    assert_eq!(rows[7], " DATE");
}

#[test]
fn the_completions_show_five_names_at_most_and_start_under_a_tag_with_an_accent() {
    let mut app = app_of(&["a +b1 +b2 +b3 +b4 +b5 +b6 +école"]);

    add_popup(&mut app, "Buy +b", 0);
    let rows = rows(&render_in(&app, 50, 16));
    assert_eq!(rows.iter().filter(|row| row.contains("│+b")).count(), 5);

    add_popup(&mut app, "Buy +é", 0);
    let rows = self::rows(&render_in(&app, 50, 14));
    let y = rows.iter().position(|row| row.contains("Buy +é")).expect("popup line");
    let column = |row: &str, word: &str| row.split(word).next().expect("row").chars().count();
    assert_eq!(column(&rows[y + 1], "╭"), column(&rows[y], "+é") - 1);
}

#[test]
fn the_popup_of_an_add_under_due_names_the_entry() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    let mut app = app_of(&["Pay rent due:2026-09-26"]);
    app.filter = Some(DUE_NOW.to_string());
    app.handle_key(KeyEvent::from(KeyCode::Char('o')), TODAY);

    let rows = rows(&render(&app));

    assert!(rows.iter().any(|row| row.contains("╭ ADD (Due) ───")), "{rows:?}");
}

#[test]
fn the_popup_is_titled_by_the_number_of_the_task_edited_and_leaves_waiting_out_of_an_add() {
    let mut app = app_of(&["Pay rent wait:bank"]);
    let titled = |app: &App, title: &str| rows(&render_in(app, 50, 8)).iter().any(|row| row.contains(title));

    app.filter = Some(WAITING.to_string());
    add_popup(&mut app, "Call", 0);
    assert!(titled(&app, "╭ ADD ─"));

    if let Focus::Popup(popup) = &mut app.focus {
        popup.target = Target::Edit(1);
    }
    assert!(titled(&app, "╭ EDIT 1 ─"));
}

fn cells(row: &str, x: usize, width: usize) -> String {
    row.chars().skip(x).take(width).collect()
}
