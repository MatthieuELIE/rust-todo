use super::*;
use crate::tui::Popup;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;
use time::macros::date;

const TODAY: Date = date!(2026 - 09 - 26);

fn app_of(lines: &[&str]) -> App {
    App::new(lines.iter().map(|l| Todo::from_line(l)).collect())
}

fn render(app: &App) -> Buffer {
    render_in(app, 50, 5)
}

fn render_in(app: &App, width: u16, height: u16) -> Buffer {
    render_with_cursor(app, width, height).0
}

/// The screen and where the terminal's cursor stands, `None` when hidden.
fn render_with_cursor(app: &App, width: u16, height: u16) -> (Buffer, Option<Position>) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, app, &mut ListState::default(), TODAY)).unwrap();
    let backend = terminal.backend();
    (backend.buffer().clone(), backend.cursor_visible().then(|| backend.cursor_position()))
}

fn rows(buffer: &Buffer) -> Vec<String> {
    let row = |y| {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    };
    (0..buffer.area.height).map(row).collect()
}

/// Rows of the panel card's inside.
fn panel_rows(buffer: &Buffer) -> Vec<String> {
    columns(buffer, 1, PANEL_WIDTH - 1)
}

/// Rows of the right column's cards' inside, the list's and the details'.
fn list_rows(buffer: &Buffer) -> Vec<String> {
    columns(buffer, PANEL_WIDTH + 2, buffer.area.width - 1)
}

fn columns(buffer: &Buffer, from: u16, to: u16) -> Vec<String> {
    let row = |y| (from..to).map(|x| buffer[(x, y)].symbol()).collect::<String>().trim_end().to_string();
    (0..buffer.area.height).map(row).collect()
}

#[test]
fn the_list_card_sits_right_of_the_panel_card_with_the_cursor_row_marked_in_the_accent_and_highlighted() {
    let mut app = app_of(&["Pay +rent", "Call +bank @phone", "x 2026-09-20 Old +rent"]);
    app.cursor = 1;

    let buffer = render_in(&app, 50, 7);

    assert_eq!(
        rows(&buffer)[..6],
        [
            "╭────────────────────╮ ╭─────────────────────────╮",
            "│→ All tasks       2 │ │    1  Pay +rent         │",
            "│                    │ │→   2  Call +bank @phone │",
            "│ PROJECTS           │ │                         │",
            "│  +bank           1 │ │                         │",
            "╰────────────────────╯ ╰─────────────────────────╯",
        ]
    );
    assert_eq!(
        (buffer[(23, 0)].fg, buffer[(0, 0)].fg, buffer[(22, 0)].bg),
        (ACCENT, STRUCTURE, BACKGROUND)
    );
    assert_eq!((buffer[(34, 2)].bg, buffer[(24, 2)].fg), (SELECTED, ACCENT));
    assert_eq!(buffer[(34, 1)].bg, BACKGROUND);
    assert_eq!((buffer[(1, 1)].fg, buffer[(6, 1)].bg), (TERTIARY, BACKGROUND));
    assert!(buffer[(3, 1)].modifier.contains(Modifier::BOLD));
    assert!(!buffer[(3, 4)].modifier.contains(Modifier::BOLD));
}

#[test]
fn the_accent_and_the_highlight_follow_the_focus_between_the_panel_and_the_list() {
    let mut app = app_of(&["Pay +rent", "Call +bank"]);
    app.filter = Some("+rent".to_string());

    let buffer = render_in(&app, 50, 8);
    assert!(panel_rows(&buffer)[1].starts_with("  All tasks "));
    assert!(panel_rows(&buffer)[5].starts_with("→ +rent "));
    assert_eq!(
        (buffer[(6, 5)].bg, buffer[(1, 5)].fg, buffer[(0, 0)].fg),
        (BACKGROUND, TERTIARY, STRUCTURE)
    );

    app.focus = Focus::Panel;
    let buffer = render_in(&app, 50, 8);
    assert_eq!((buffer[(6, 5)].bg, buffer[(1, 5)].fg, buffer[(0, 0)].fg), (SELECTED, ACCENT, ACCENT));
    assert_eq!(
        (buffer[(34, 1)].bg, buffer[(24, 1)].fg, buffer[(23, 0)].fg),
        (BACKGROUND, TERTIARY, STRUCTURE)
    );
}

#[test]
fn a_filter_gone_from_the_panel_marks_no_entry() {
    let mut app = app_of(&["Pay +rent", "x 2026-09-20 Call +bank"]);
    app.filter = Some("+bank".to_string());
    app.focus = Focus::Panel;

    let buffer = render_in(&app, 50, 8);
    assert_eq!(panel_rows(&buffer)[1], "  All tasks       1");
    assert!(panel_rows(&buffer).iter().all(|row| !row.starts_with('→')));
    assert_eq!(buffer[(6, 1)].bg, BACKGROUND);
}

#[test]
fn the_panel_puts_coloured_projects_and_contexts_under_grey_headers_and_drops_an_empty_section() {
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
    assert_eq!((buffer[(2, 3)].fg, buffer[(2, 7)].fg), (SECONDARY, SECONDARY));
    assert!(buffer[(2, 3)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(3, 4)].fg, buffer[(3, 8)].fg, buffer[(3, 1)].fg), (PROJECT, CONTEXT, PRIMARY));
    assert_eq!(buffer[(19, 4)].fg, TERTIARY);

    let waiting = panel_rows(&render_in(&app_of(&["Design wait:figma +app"]), 50, 11));
    assert_eq!(waiting[1..5], ["→ All tasks       1", "  Waiting         1", "", " PROJECTS"]);

    let contexts_only = panel_rows(&render_in(&app_of(&["Call @phone"]), 50, 11));
    assert_eq!(contexts_only[3], " CONTEXTS");
    assert!(!contexts_only.iter().any(|row| row.contains("PROJECTS")));
}

#[test]
fn group_headers_sit_above_their_tasks_with_the_cursor_on_the_same_task() {
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
    assert_eq!(buffer[(34, 5)].bg, SELECTED);
    assert_eq!((buffer[(32, 1)].fg, buffer[(32, 4)].fg), (PRIORITIES[0], PRIORITIES[1]));
    assert_eq!(
        (buffer[(32, 7)].fg, buffer[(53, 1)].fg, buffer[(27, 1)].fg),
        (SECONDARY, STRUCTURE, TERTIARY)
    );
}

#[test]
fn a_folded_group_is_its_header_alone_ended_by_a_marker_and_the_cursor_can_stand_on_it() {
    let mut app = app_of(&["(A) a", "(A) b", "(B) c", "d"]);
    app.folded = vec![Group::Priority('A'), Group::Priority('B'), Group::Unprioritised];
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
    assert_eq!(buffer[(34, 3)].bg, SELECTED);
}

#[test]
fn the_help_shows_each_mode_s_keys_under_its_name_keys_bold_actions_and_slashes_greyed() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help;

    let buffer = render_in(&app, 90, 32);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains(" LIST ")).expect("list block");

    assert!(rows[y - 2].contains("╭ HELP ─"), "{}", rows[y - 2]);
    let corner = rows[y - 2].chars().position(|c| c == '╭').expect("corner") as u16;
    assert_eq!(buffer[(corner, y as u16 - 2)].fg, ACCENT);
    assert!(rows.iter().any(|row| row.contains("─ any key closes ╯")));
    assert!(rows[y].contains(" INSERT "), "{}", rows[y]);
    assert!(rows[y + 1].contains("j/k/↓/↑      move"), "{}", rows[y + 1]);
    assert!(rows.iter().any(|row| row.contains(" NORMAL ")));
    assert!(rows.iter().any(|row| row.contains(" PANEL ")));
    assert!(rows.iter().any(|row| row.contains("za           fold, unfold group")));
    assert!(rows.iter().any(|row| row.contains("Tab/Enter    back to the list")));
    assert!(rows.iter().any(|row| row.contains("Esc          all tasks")));
    assert_eq!(rows.iter().filter(|row| row.contains("p Space      no priority")).count(), 2);

    let y = y as u16;
    assert_eq!((buffer[(9, y)].fg, buffer[(47, y)].fg), (SECONDARY, SECONDARY));
    assert!(buffer[(9, y)].modifier.contains(Modifier::BOLD));
    let (key, slash, action) = (&buffer[(8, y + 1)], &buffer[(9, y + 1)], &buffer[(21, y + 1)]);
    assert!(key.fg == PRIMARY && key.modifier.contains(Modifier::BOLD));
    assert!(slash.fg == SEPARATOR && !slash.modifier.contains(Modifier::BOLD));
    assert!(action.fg == TERTIARY && !action.modifier.contains(Modifier::BOLD));
    let search = rows.iter().position(|row| row.contains("/            search")).expect("search key") as u16;
    assert_eq!(buffer[(8, search)].fg, PRIMARY);
}

#[test]
fn every_cell_left_unpainted_takes_the_background_and_the_text_colour_and_a_floating_window_its_raised_one() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help;

    let buffer = render_in(&app, 80, 30);

    assert_eq!((buffer[(40, 15)].bg, buffer[(79, 29)].bg), (RAISED, BACKGROUND));
    assert!(buffer.content.iter().all(|cell| cell.bg != Color::Reset && cell.fg != Color::Reset));
}

#[test]
fn the_status_bar_shows_the_mode_and_filters_left_and_the_message_right() {
    let mut app = app_of(&["Pay +rent"]);
    app.filter = Some("+rent".to_string());
    app.search = "pay".to_string();
    app.show_done = true;
    app.message = Some("reloaded".to_string());

    let buffer = render(&app);
    let status = rows(&buffer)[4].clone();
    assert!(status.starts_with(" LIST  +rent  /pay  +done "), "{status}");
    assert!(status.ends_with(" reloaded"), "{status}");
    assert_eq!((buffer[(1, 4)].bg, buffer[(1, 4)].fg), (ACCENT, ON_ACCENT));
    assert!(buffer[(1, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(7, 4)].bg, BACKGROUND);
    assert_eq!((buffer[(7, 4)].fg, buffer[(14, 4)].fg, buffer[(20, 4)].fg), (PROJECT, PRIMARY, PRIMARY));
    assert_eq!(buffer[(45, 4)].fg, SECONDARY);

    app.refused = true;
    assert_eq!(render(&app)[(45, 4)].fg, ALERT);
}

#[test]
fn the_mode_keys_follow_when_they_fit_whole_and_no_message_is_shown() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Panel;

    let buffer = render(&app);
    assert_eq!(rows(&buffer)[4], " PANEL  j/k filter · esc all tasks · tab back");
    assert_eq!(buffer[(1, 4)].bg, ACCENT);
    assert!(buffer[(8, 4)].fg == PRIMARY && buffer[(8, 4)].modifier.contains(Modifier::BOLD));
    assert!(buffer[(12, 4)].fg == TERTIARY && !buffer[(12, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(19, 4)].fg, SEPARATOR);

    app.message = Some("reloaded".to_string());
    assert!(!rows(&render(&app))[4].contains("j/k"));

    app.focus = Focus::List;
    app.message = None;
    assert_eq!(rows(&render(&app))[4], " LIST");
    assert_eq!(rows(&render_in(&app, 60, 5))[4], " LIST  ⏎ edit · o add · x done · p priority · ? help");

    app.focus = Focus::Search;
    app.search = "ca".to_string();
    let buffer = render(&app);
    assert_eq!(rows(&buffer)[4], " SEARCH  /ca▌  ⏎ keep · esc clear");
    assert_eq!(buffer[(1, 4)].bg, ACCENT);
}

#[test]
fn the_popup_wraps_its_text_under_its_title_with_the_terminal_cursor_on_its_cell_while_the_status_bar_says_insert() {
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
    assert_eq!((buffer[(corner, 0)].fg, buffer[(corner + 1, 1)].bg), (ACCENT, RAISED));
    assert!(buffer[(corner + 2, 0)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(corner + 7, 0)].fg, buffer[(corner + 2, 0)].fg), (PROJECT, PRIMARY));
    assert_eq!(buffer[(11, 1)].symbol(), "t");
    assert_eq!(cursor, Some(Position::new(11, 1)));
    assert!(buffer.content.iter().all(|cell| !cell.modifier.contains(Modifier::REVERSED)));
    assert_eq!(render_with_cursor(&app_of(&["Pay +rent"]), 50, 5).1, None);
    assert_eq!(rows[4], " INSERT  +rent  esc normal · ⏎ save");
    assert_eq!(buffer[(1, 4)].bg, ACCENT);

    if let Focus::Popup(popup) = &mut app.focus {
        popup.editor.mode = Mode::Normal;
    }
    let buffer = render_in(&app, 70, 5);
    assert_eq!(self::rows(&buffer)[4], " NORMAL  +rent  i insert · p priority · ⏎ save · esc cancel");
    assert_eq!(buffer[(1, 4)].bg, ACCENT);
}

#[test]
fn the_popup_colours_its_text_as_the_list_colours_a_task() {
    let mut app = app_of(&["Pay rent"]);
    add_popup(&mut app, "(A) 2026-09-01 Call +bank @phone due:2026-09-20 wait:x", 0);

    let (buffer, cursor) = render_with_cursor(&app, 80, 7);
    let row = rows(&buffer).iter().position(|row| row.contains("(A) 2026")).expect("popup line");
    let x = rows(&buffer)[row].split("(A)").next().expect("row").chars().count() as u16;
    let at = |offset: u16| &buffer[(x + offset, row as u16)];

    assert_eq!((at(1).fg, at(4).fg, at(20).fg, at(26).fg), (PRIORITIES[0], TERTIARY, PROJECT, CONTEXT));
    assert!(at(1).modifier.contains(Modifier::BOLD));
    assert_eq!((at(15).fg, at(33).fg, at(48).fg), (PRIMARY, ALERT, TERTIARY));
    assert_eq!(cursor, Some(Position::new(x + 54, row as u16)));
}

fn add_popup(app: &mut App, text: &str, selected: usize) {
    let mut editor = Editor::default();
    editor.text = text.to_string();
    editor.cursor = text.chars().count();
    app.focus = Focus::Popup(Popup {
        editor,
        target: Target::Add,
        selected,
        picker: None,
    });
}

fn cells(row: &str, x: usize, width: usize) -> String {
    row.chars().skip(x).take(width).collect()
}

#[test]
fn the_completions_drop_down_under_the_tag_with_their_counts_and_the_pick_highlighted() {
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
    assert_eq!((buffer[(10, y + 2)].fg, buffer[(19, y + 2)].fg), (PROJECT, TERTIARY));
    assert_eq!((buffer[(10, y + 2)].bg, buffer[(10, y + 3)].bg), (RAISED, SELECTED));
    assert_eq!(buffer[(9, y + 1)].fg, ACCENT);
    assert_eq!(rows[11], " INSERT  esc normal · ⏎ save · tab complete");
}

#[test]
fn wait_completions_are_greyed_like_the_key_values_of_a_line() {
    let mut app = app_of(&["Design wait:figma"]);
    add_popup(&mut app, "Draw wait:", 0);

    let buffer = render_in(&app, 50, 12);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains("│wait:figma")).expect("drop-down row");
    let x = rows[y].split("│wait:figma").next().expect("row").chars().count() + 1;
    let (x, y) = (x as u16, y as u16);

    assert_eq!(buffer[(x, y)].fg, TERTIARY);
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

fn style_of(line_text: &str, token: &str) -> Style {
    let line = line(1, &Todo::from_line(line_text), TODAY);
    line.iter().find(|span| span.content == token).expect(token).style
}

#[test]
fn a_pending_line_keeps_its_text_but_shows_its_priority_as_a_badge_and_colours_date_projects_and_contexts() {
    let text = "2026-09-26 Call  +bank @phone due:2026-10-01 a+b + @";

    assert_eq!(
        line(12, &Todo::from_line(&format!("(A) {text}")), TODAY).to_string(),
        format!(" 12   A  {text}")
    );
    let badge = |letter: usize| Style::new().bold().fg(PRIORITIES[letter]).bg(BADGE);
    assert_eq!(style_of(&format!("(A) {text}"), " A "), badge(0));
    assert_eq!(style_of(text, "2026-09-26"), Style::new().fg(TERTIARY));
    assert_eq!(style_of(text, "+bank"), Style::new().fg(PROJECT));
    assert_eq!(style_of(text, "@phone"), Style::new().fg(CONTEXT));
    assert_eq!(style_of(text, "due:2026-10-01"), Style::new().fg(TERTIARY));
    for plain in ["Call", "a+b", "+", "@"] {
        assert_eq!(style_of(text, plain), Style::new(), "{plain}");
    }
    for (i, letter) in ['B', 'C', 'D', 'E'].into_iter().enumerate() {
        assert_eq!(style_of(&format!("({letter}) x"), &format!(" {letter} ")), badge(i + 1), "{letter}");
    }
}

#[test]
fn key_values_are_greyed_but_not_urls_times_or_labels() {
    let text = "Call wait:figma rec:1w a:b:c é-t_2:x https://herdr.dev 10:30 Note: wait: :x _a:b a/b:c";

    for greyed in ["wait:figma", "rec:1w", "a:b:c", "é-t_2:x"] {
        assert_eq!(style_of(text, greyed), Style::new().fg(TERTIARY), "{greyed}");
    }
    for plain in ["https://herdr.dev", "10:30", "Note:", "wait:", ":x", "_a:b", "a/b:c"] {
        assert_eq!(style_of(text, plain), Style::new(), "{plain}");
    }
}

#[test]
fn a_done_line_is_greyed_and_struck_through_after_its_number() {
    let line = line(3, &Todo::from_line("x 2026-09-26 2026-09-20 Call +bank @phone due:2026-09-01"), TODAY);

    assert_eq!(line.to_string(), "  3  x 2026-09-26 2026-09-20 Call +bank @phone due:2026-09-01");
    assert!(line.iter().skip(1).all(|span| span.style == Style::new().fg(TERTIARY).crossed_out()));
}

#[test]
fn projects_and_contexts_are_coloured_on_screen() {
    let buffer = render(&app_of(&["Call +bank @phone"]));

    assert_eq!(list_rows(&buffer)[1], "→   1  Call +bank @phone");
    assert_eq!((buffer[(36, 1)].fg, buffer[(42, 1)].fg, buffer[(31, 1)].fg), (PROJECT, CONTEXT, PRIMARY));
}

#[test]
fn the_details_card_under_the_list_shows_the_task_under_the_cursor() {
    let app = app_of(&["(A) 2026-09-01 Appeler +client pour le devis @work wait:figma"]);

    let buffer = render_in(&app, 75, 18);

    assert_eq!(
        list_rows(&buffer)[9..16],
        [
            String::new(),
            format!(" DETAILS {}", "─".repeat(41)),
            " Priority   A".to_string(),
            " Text      Appeler +client pour le devis".to_string(),
            format!("{:<25} Due", " Created   2026-09-01"),
            format!("{:<25} Contexts  @work", " Projects  +client"),
            " wait:figma".to_string(),
        ]
    );
    assert_eq!(rows(&buffer)[16].chars().nth(23), Some('╰'));
    assert_eq!(
        (buffer[(29, 10)].fg, buffer[(49, 10)].fg, buffer[(23, 10)].fg, buffer[(25, 11)].fg),
        (SECONDARY, STRUCTURE, STRUCTURE, LABEL)
    );
    assert!(buffer[(29, 10)].modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(36, 11)].fg, buffer[(35, 11)].bg), (PRIORITIES[0], BADGE));
    assert_eq!(
        (buffer[(43, 12)].fg, buffer[(35, 14)].fg, buffer[(60, 14)].fg),
        (PROJECT, PROJECT, CONTEXT)
    );
    assert_eq!(buffer[(35, 12)].fg, PRIMARY);
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
fn a_value_too_long_for_the_details_ends_with_an_ellipsis() {
    let text = "A rather long description that will not fit here";

    let buffer = render_in(&app_of(&[text]), 65, 16);

    assert_eq!(list_rows(&buffer)[10], format!(" Text      {}…", &text[..28]));
}

#[test]
fn the_details_stay_empty_when_the_cursor_is_on_no_task() {
    let mut app = app_of(&["(A) a"]);
    app.folded = vec![Group::Priority('A')];

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
fn the_details_show_the_due_date_beside_the_creation_date_and_the_other_key_values_on_the_last_row() {
    let app = app_of(&["2026-09-01 Ship it due:2026-10-15 wait:figma rec:1w"]);

    let buffer = render_in(&app, 75, 18);
    let zone = &list_rows(&buffer)[11..16];

    assert_eq!(zone[1], " Text      Ship it");
    assert_eq!(zone[2], format!("{:<25} Due       2026-10-15", " Created   2026-09-01"));
    assert_eq!(zone[4], " wait:figma  rec:1w");
    assert_eq!((buffer[(25, 15)].fg, buffer[(30, 15)].fg), (TERTIARY, PRIMARY));
}

#[test]
fn a_done_task_shows_its_completion_date_first_and_the_whole_zone_greyed_but_not_struck_through() {
    let mut app = app_of(&["x 2026-09-28 2026-09-01 Ship it +work due:2026-09-20"]);
    app.show_done = true;

    let buffer = render_in(&app, 75, 18);

    assert_eq!(list_rows(&buffer)[11], " Done      2026-09-28");
    let zone = (24..74).flat_map(|x| (11..16).map(move |y| (x, y)));
    assert!(zone.clone().all(|cell| buffer[cell].fg == TERTIARY));
    assert!(zone.clone().all(|cell| !buffer[cell].modifier.contains(Modifier::CROSSED_OUT)));
}

#[test]
fn a_due_date_is_red_once_past_yellow_on_the_day_and_greyed_later_or_when_not_a_date() {
    let text = "Ship due:2026-09-25 due:2026-09-26 due:2026-09-27 due:someday";

    assert_eq!(style_of(text, "due:2026-09-25"), Style::new().fg(ALERT));
    assert_eq!(style_of(text, "due:2026-09-26"), Style::new().fg(DUE_TODAY));
    assert_eq!(style_of(text, "due:2026-09-27"), Style::new().fg(TERTIARY));
    assert_eq!(style_of(text, "due:someday"), Style::new().fg(TERTIARY));
}

#[test]
fn the_due_date_in_the_details_is_red_once_past() {
    let buffer = render_in(&app_of(&["Ship it due:2026-09-20"]), 75, 18);

    assert_eq!(list_rows(&buffer)[13], format!("{:<25} Due       2026-09-20", " Created"));
    assert_eq!(buffer[(60, 13)].fg, ALERT);
}

#[test]
fn the_date_picker_drops_down_under_due_colon_as_a_month_from_monday_with_the_pick_in_the_accent_and_today_yellow() {
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
    assert_eq!(buffer[(x + 1, y)].fg, TERTIARY);
    let pick = &buffer[(x + 1, y + 5)];
    assert_eq!((pick.fg, pick.bg), (ACCENT, SELECTED));
    assert!(pick.modifier.contains(Modifier::BOLD));
    assert_eq!((buffer[(x + 4, y + 5)].bg, buffer[(x + 4, y + 5)].fg), (RAISED, PRIMARY));
    assert_eq!(buffer[(x, y - 1)].fg, ACCENT);
    let popup = rows.iter().position(|row| row.contains("╭ ADD ")).expect("popup top");
    let corner = rows[popup].split("╭ ADD").next().expect("row").chars().count() as u16;
    assert_eq!(buffer[(corner, popup as u16)].fg, SEPARATOR);
    assert_eq!(render_with_cursor(&app, 60, 20).1, None);
    assert_eq!((buffer[(x + 16, y + 4)].fg, buffer[(x + 13, y + 4)].fg), (DUE_TODAY, TERTIARY));
    assert_eq!(rows[19], " DATE  hjkl move · H/L month · ⏎ pick · esc close");
    assert_eq!(buffer[(1, 19)].bg, ACCENT);
}

#[test]
fn the_keys_of_a_command_waiting_for_its_end_show_on_the_right_of_the_status_bar() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    let key = |app: &mut App, code: KeyCode| app.handle_key(KeyEvent::from(code), TODAY);
    let status = |app: &App| rows(&render_in(app, 70, 5))[4].clone();
    let mut app = app_of(&["Pay +rent"]);

    key(&mut app, KeyCode::Char('d'));
    assert!(status(&app).contains("? help") && status(&app).ends_with(" d"), "{}", status(&app));
    assert_eq!(render_in(&app, 70, 5)[(68, 4)].fg, SECONDARY);
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
