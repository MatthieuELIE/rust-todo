use super::super::tests::{MOCHA, TODAY, app_of, fold_all, list_rows, render, render_in, style_of};
use super::*;

#[test]
fn group_headers_sit_above_their_tasks_with_the_cursor_on_the_same_task_in_mocha() {
    let mut app = app_of(&["c", "(B) b", "(A) a"]);
    app.cursor = 1;

    let buffer = render_in(&app, 55, 11);

    assert_eq!(
        list_rows(&buffer)[1..9],
        [
            format!("   (1)  PRIORITY A  {}", "─".repeat(10)),
            "    3   A  a".to_string(),
            String::new(),
            format!("   (1)  PRIORITY B  {}", "─".repeat(10)),
            "→   2   B  b".to_string(),
            String::new(),
            format!("   (1)  NO PRIORITY  {}", "─".repeat(9)),
            "    1  c".to_string(),
        ]
    );
    assert_eq!(buffer[(34, 5)].bg, MOCHA.selected);
    assert_eq!((buffer[(32, 1)].fg, buffer[(32, 4)].fg), (MOCHA.priorities[0], MOCHA.priorities[1]));
    assert_eq!(
        (buffer[(32, 7)].fg, buffer[(53, 1)].fg, buffer[(27, 1)].fg),
        (MOCHA.secondary, MOCHA.structure, MOCHA.tertiary)
    );
}

#[test]
fn a_folded_group_is_its_header_alone_ended_by_a_marker_and_the_cursor_can_stand_on_it_in_mocha() {
    let mut app = app_of(&["(A) a", "(A) b", "(B) c", "d"]);
    fold_all(&mut app);
    app.cursor = 1;

    let buffer = render_in(&app, 55, 9);

    assert_eq!(
        list_rows(&buffer)[1..7],
        [
            format!("   (2)  PRIORITY A  {} ▸", "─".repeat(8)),
            String::new(),
            format!("→  (1)  PRIORITY B  {} ▸", "─".repeat(8)),
            String::new(),
            format!("   (1)  NO PRIORITY  {} ▸", "─".repeat(7)),
            String::new(),
        ]
    );
    assert_eq!(buffer[(34, 3)].bg, MOCHA.selected);
}

#[test]
fn a_pending_line_keeps_its_text_but_shows_its_priority_as_a_badge_and_colours_date_projects_and_contexts_in_mocha() {
    let text = "2026-09-26 Call  +bank @phone due:2026-10-01 a+b + @";

    assert_eq!(
        line(&MOCHA, 12, &Todo::from_line(&format!("(A) {text}")), TODAY).to_string(),
        format!(" 12   A  {text}")
    );
    let badge = |letter: usize| Style::new().bold().fg(MOCHA.priorities[letter]).bg(MOCHA.badge);
    assert_eq!(style_of(&format!("(A) {text}"), " A "), badge(0));
    assert_eq!(style_of(text, "2026-09-26"), Style::new().fg(MOCHA.tertiary));
    assert_eq!(style_of(text, "+bank"), Style::new().fg(MOCHA.project));
    assert_eq!(style_of(text, "@phone"), Style::new().fg(MOCHA.context));
    assert_eq!(style_of(text, "due:2026-10-01"), Style::new().fg(MOCHA.tertiary));
    for plain in ["Call", "a+b", "+", "@"] {
        assert_eq!(style_of(text, plain), Style::new(), "{plain}");
    }
    for (i, letter) in ['B', 'C', 'D', 'E'].into_iter().enumerate() {
        assert_eq!(style_of(&format!("({letter}) x"), &format!(" {letter} ")), badge(i + 1), "{letter}");
    }
}

#[test]
fn a_done_line_is_greyed_and_struck_through_after_its_number_in_mocha() {
    let line = line(
        &MOCHA,
        3,
        &Todo::from_line("x 2026-09-26 2026-09-20 Call +bank @phone due:2026-09-01"),
        TODAY,
    );

    assert_eq!(line.to_string(), "  3  x 2026-09-26 2026-09-20 Call +bank @phone due:2026-09-01");
    assert!(
        line.iter()
            .skip(1)
            .all(|span| span.style == Style::new().fg(MOCHA.tertiary).crossed_out())
    );
}

#[test]
fn projects_and_contexts_are_coloured_on_screen_in_mocha() {
    let buffer = render(&app_of(&["Call +bank @phone"]));

    assert_eq!(list_rows(&buffer)[1], "→   1  Call +bank @phone");
    assert_eq!(
        (buffer[(36, 1)].fg, buffer[(42, 1)].fg, buffer[(31, 1)].fg),
        (MOCHA.project, MOCHA.context, MOCHA.primary)
    );
}

#[test]
fn an_empty_list_says_nothing_to_do_and_a_search_or_a_filter_that_nothing_matched() {
    let said = |app: &App| list_rows(&render(app))[1].clone();
    let mut app = app_of(&["Pay rent"]);

    assert_eq!(said(&app_of(&[])), "nothing to do");
    app.search = "bank".to_string();
    assert_eq!(said(&app), "no matching task");
    app.search.clear();
    app.filter = Some("+bank".to_string());
    assert_eq!(said(&app), "no matching task");
    app.filter = None;
    app.in_history = true;
    assert_eq!(said(&app), "nothing done");
}

#[test]
fn the_list_scrolls_one_row_ahead_of_the_cursor() {
    let mut app = App::new((1..=12).map(|n| Todo::from_line(&format!("task{n:02}"))).collect());
    app.cursor = 6;

    let rows = list_rows(&render_in(&app, 50, 8));

    assert!(rows.iter().any(|row| row.contains("task08")), "{rows:?}");
    assert!(!rows.iter().any(|row| row.contains("task09")), "{rows:?}");
}
