use super::*;
use crate::editor::Editor;
use crate::todo::Todo;
use crate::tui::{Popup, Target};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::style::{Modifier, Style};
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
pub(super) fn render_with_cursor(app: &App, width: u16, height: u16) -> (Buffer, Option<Position>) {
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
fn every_cell_left_unpainted_takes_the_background_and_the_text_colour_and_a_floating_window_its_raised_one_in_mocha() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help(0);

    let buffer = render_in(&app, 80, 30);

    assert_eq!((buffer[(40, 15)].bg, buffer[(79, 29)].bg), (MOCHA.raised, MOCHA.background));
    assert!(buffer.content.iter().all(|cell| cell.bg != Color::Reset && cell.fg != Color::Reset));
}

pub(super) fn add_popup(app: &mut App, text: &str, selected: usize) {
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

pub(super) fn style_of(line_text: &str, token: &str) -> Style {
    let line = line(&MOCHA, 1, &Todo::from_line(line_text), TODAY);
    line.iter().find(|span| span.content == token).expect(token).style
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
fn the_error_screen_shows_the_message_and_how_to_leave() {
    let mut terminal = Terminal::new(TestBackend::new(40, 4)).unwrap();

    terminal.draw(|frame| draw_error(frame, &MOCHA, "could not read todo.txt")).unwrap();

    let rows = rows(terminal.backend().buffer());
    assert_eq!(rows[..2], [" could not read todo.txt", " press any key to quit"]);
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
