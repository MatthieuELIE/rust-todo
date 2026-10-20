use super::*;
use crate::editor::Mode;
use crate::tui::{DUE_NOW, Popup};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;
use time::macros::date;

pub(super) const TODAY: Date = date!(2026 - 09 - 26);
pub(super) const MOCHA: Theme = Theme::MOCHA;

pub(super) fn app_of(lines: &[&str]) -> App {
    let mut app = App::new(lines.iter().map(|l| Todo::from_line(l)).collect());
    app.today = Some(TODAY);
    app.theme = MOCHA;
    app
}

pub(super) fn fold_all(app: &mut App) {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    for key in ['z', 'M'] {
        app.handle_key(KeyEvent::from(KeyCode::Char(key)), TODAY);
    }
}

pub(super) fn render(app: &App) -> Buffer {
    render_in(app, 50, 5)
}

pub(super) fn render_in(app: &App, width: u16, height: u16) -> Buffer {
    render_with_cursor(app, width, height).0
}

/// The screen and where the terminal's cursor stands, `None` when hidden.
fn render_with_cursor(app: &App, width: u16, height: u16) -> (Buffer, Option<Position>) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, app, &mut ListState::default(), TODAY)).unwrap();
    let backend = terminal.backend();
    (backend.buffer().clone(), backend.cursor_visible().then(|| backend.cursor_position()))
}

pub(super) fn rows(buffer: &Buffer) -> Vec<String> {
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
pub(super) fn panel_rows(buffer: &Buffer) -> Vec<String> {
    columns(buffer, 1, PANEL_WIDTH - 1)
}

/// Rows of the right column's cards' inside, the list's and the details'.
pub(super) fn list_rows(buffer: &Buffer) -> Vec<String> {
    columns(buffer, PANEL_WIDTH + 2, buffer.area.width - 1)
}

fn columns(buffer: &Buffer, from: u16, to: u16) -> Vec<String> {
    let row = |y| (from..to).map(|x| buffer[(x, y)].symbol()).collect::<String>().trim_end().to_string();
    (0..buffer.area.height).map(row).collect()
}

#[test]
fn the_list_card_sits_right_of_the_panel_card_with_the_cursor_row_marked_in_the_accent_and_highlighted_in_mocha() {
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
        (MOCHA.accent, MOCHA.structure, MOCHA.background)
    );
    assert_eq!((buffer[(34, 2)].bg, buffer[(24, 2)].fg), (MOCHA.selected, MOCHA.accent));
    assert_eq!(buffer[(34, 1)].bg, MOCHA.background);
    assert_eq!((buffer[(1, 1)].fg, buffer[(6, 1)].bg), (MOCHA.tertiary, MOCHA.background));
    assert!(buffer[(3, 1)].modifier.contains(Modifier::BOLD));
    assert!(!buffer[(3, 4)].modifier.contains(Modifier::BOLD));
}

#[test]
fn the_accent_and_the_highlight_follow_the_focus_between_the_panel_and_the_list_in_mocha() {
    let mut app = app_of(&["Pay +rent", "Call +bank"]);
    app.filter = Some("+rent".to_string());

    let buffer = render_in(&app, 50, 8);
    assert!(panel_rows(&buffer)[1].starts_with("  All tasks "));
    assert!(panel_rows(&buffer)[5].starts_with("→ +rent "));
    assert_eq!(
        (buffer[(6, 5)].bg, buffer[(1, 5)].fg, buffer[(0, 0)].fg),
        (MOCHA.background, MOCHA.tertiary, MOCHA.structure)
    );

    app.focus = Focus::Panel;
    let buffer = render_in(&app, 50, 8);
    assert_eq!(
        (buffer[(6, 5)].bg, buffer[(1, 5)].fg, buffer[(0, 0)].fg),
        (MOCHA.selected, MOCHA.accent, MOCHA.accent)
    );
    assert_eq!(
        (buffer[(34, 1)].bg, buffer[(24, 1)].fg, buffer[(23, 0)].fg),
        (MOCHA.background, MOCHA.tertiary, MOCHA.structure)
    );

    app.focus = Focus::Search;
    let buffer = render_in(&app, 50, 8);
    assert_eq!(
        (buffer[(34, 1)].bg, buffer[(24, 1)].fg, buffer[(23, 0)].fg),
        (MOCHA.selected, MOCHA.accent, MOCHA.accent)
    );
}

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
fn every_cell_left_unpainted_takes_the_background_and_the_text_colour_and_a_floating_window_its_raised_one_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(0);

    let buffer = render_in(&app, 80, 30);

    assert_eq!((buffer[(40, 15)].bg, buffer[(79, 29)].bg), (MOCHA.raised, MOCHA.background));
    assert!(buffer.content.iter().all(|cell| cell.bg != Color::Reset && cell.fg != Color::Reset));
}

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

fn cells(row: &str, x: usize, width: usize) -> String {
    row.chars().skip(x).take(width).collect()
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
fn the_terminal_theme_is_the_default_and_paints_no_rgb_cell_on_any_screen() {
    let mut app = app_of(&["(A) Pay +bank @home due:2026-09-20", "(C) Call due:2026-09-26", "x Read +books"]);
    app.theme = Theme::default();
    add_popup(&mut app, "Buy +b", 1);
    let mut screens = vec![render_in(&app, 80, 30)];
    app.focus = Focus::Help(0);
    screens.push(render_in(&app, 80, 30));

    assert_eq!(app.theme.accent, Color::Blue);
    for buffer in screens {
        assert!(
            buffer
                .content
                .iter()
                .all(|cell| !matches!(cell.fg, Color::Rgb(..)) && !matches!(cell.bg, Color::Rgb(..)))
        );
    }
}

#[test]
fn the_terminal_theme_leaves_the_background_and_the_text_to_the_terminal_on_the_cursor_row_too() {
    let mut app = app_of(&["Pay the rent", "Call the bank"]);
    app.theme = Theme::default();

    let buffer = render_in(&app, 80, 30);
    let rows = rows(&buffer);

    for word in ["Pay", "Call"] {
        let y = rows.iter().position(|row| row.contains(word)).expect(word);
        let x = rows[y][..rows[y].find(word).unwrap()].chars().count() as u16;
        let cell = &buffer[(x, y as u16)];
        assert_eq!((cell.fg, cell.bg), (Color::Reset, Color::Reset), "{word}");
    }
    assert_eq!(buffer[(60, 20)].bg, Color::Reset);
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

pub(super) fn style_of(line_text: &str, token: &str) -> Style {
    let line = line(&MOCHA, 1, &Todo::from_line(line_text), TODAY);
    line.iter().find(|span| span.content == token).expect(token).style
}

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
fn the_history_shows_its_lines_and_the_panel_in_light_grey_not_struck_through_and_the_details_keep_their_colours_in_mocha() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    let mut app = app_of(&["Pay +rent"]);
    app.set_history(Ok("x 2026-09-28 2026-09-01 Ship it +work @desk due:2026-09-20\n".to_string()));
    app.handle_key(KeyEvent::from(KeyCode::Char('H')), TODAY);

    let buffer = render_in(&app, 75, 18);

    let line = "→   1  x 2026-09-28 2026-09-01 Ship it +work @desk";
    assert_eq!(list_rows(&buffer)[1], line);
    let cells = (26..74).map(|x| &buffer[(x, 1)]);
    assert!(cells.clone().all(|cell| cell.fg == MOCHA.secondary));
    assert!(cells.clone().all(|cell| !cell.modifier.contains(Modifier::CROSSED_OUT)));
    assert_eq!(
        panel_rows(&buffer)[1..8],
        [
            "→ All tasks       1",
            "",
            " PROJECTS",
            "  +work           1",
            "",
            " CONTEXTS",
            "  @desk           1"
        ]
    );
    assert_eq!(
        (buffer[(3, 1)].fg, buffer[(3, 4)].fg, buffer[(3, 7)].fg),
        (MOCHA.secondary, MOCHA.secondary, MOCHA.secondary)
    );
    assert_eq!(list_rows(&buffer)[11], " Done      2026-09-28");
    assert_eq!(
        (buffer[(35, 12)].fg, buffer[(35, 14)].fg, buffer[(60, 14)].fg),
        (MOCHA.primary, MOCHA.project, MOCHA.context)
    );
    assert!(!rows(&buffer)[17].contains("+done"));
}

#[test]
fn the_none_theme_leaves_every_cell_without_colour() {
    let mut app = app_of(&["(A) Ship +app @work due:2026-09-20 due:2026-09-26", "x Done thing", "Plain"]);
    app.theme = Theme::NONE;

    let buffer = render_in(&app, 75, 20);

    assert!(rows(&buffer).iter().any(|row| row.contains("Ship")));
    assert!(buffer.content.iter().all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset));
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
fn the_error_screen_shows_the_message_and_how_to_leave() {
    let mut terminal = Terminal::new(TestBackend::new(40, 4)).unwrap();

    terminal.draw(|frame| draw_error(frame, &MOCHA, "could not read todo.txt")).unwrap();

    let rows = rows(terminal.backend().buffer());
    assert_eq!(rows[..2], [" could not read todo.txt", " press any key to quit"]);
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

const GLYPHS: [char; 6] = ['\u{f07b}', '\u{f1fa}', '\u{f05a}', '\u{f0ca}', '\u{f073}', '\u{f252}'];

/// The private-use glyphs on screen with their position.
pub(super) fn glyph_cells(buffer: &Buffer) -> Vec<(char, u16, u16)> {
    let mut found = Vec::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(glyph) = buffer[(x, y)].symbol().chars().next().filter(|c| ('\u{e000}'..='\u{f8ff}').contains(c)) {
                found.push((glyph, x, y));
            }
        }
    }
    found
}

pub(super) fn app_with_every_entry() -> App {
    app_of(&["(A) Call due:2026-09-01 wait:bank +app @work", "Pay due:2026-09-26"])
}

#[test]
fn without_icons_no_cell_is_in_the_private_use_area() {
    let buffer = render_in(&app_with_every_entry(), 109, 36);

    assert!(glyph_cells(&buffer).is_empty());
}

#[test]
fn with_icons_the_six_glyphs_stand_before_their_word_in_its_style() {
    let mut app = app_with_every_entry();
    app.icons = true;

    let buffer = render_in(&app, 109, 36);

    let found = glyph_cells(&buffer);
    let words = ["PROJECTS", "CONTEXTS", "DETAILS", "All tasks", "Due", "Waiting"];
    assert_eq!(found.len(), 6);
    for (glyph, word) in GLYPHS.iter().zip(words) {
        let &(_, x, y) = found.iter().find(|(found, ..)| found == glyph).unwrap();
        let after: String = (x + 1..x + 2 + word.len() as u16).map(|x| buffer[(x, y)].symbol()).collect();
        assert_eq!(after, format!(" {word}"));
        let (icon, first) = (&buffer[(x, y)], &buffer[(x + 2, y)]);
        assert_eq!((icon.fg, icon.modifier), (first.fg, first.modifier), "{word}");
    }
    let due = found.iter().find(|(glyph, ..)| *glyph == '\u{f073}').unwrap();
    assert_eq!(buffer[(due.1, due.2)].fg, MOCHA.alert);
}
