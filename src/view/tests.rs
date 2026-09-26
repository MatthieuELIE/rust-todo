use super::*;
use crate::store::Store;
use crate::tui::Popup;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;

fn app_of(lines: &[&str]) -> App {
    App::new(Store::new(lines.iter().map(|l| Todo::from_line(l)).collect()))
}

fn render(app: &App) -> Buffer {
    render_in(app, 50, 5)
}

fn render_in(app: &App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, app, &mut ListState::default())).unwrap();
    terminal.backend().buffer().clone()
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

#[test]
fn the_list_sits_right_of_the_panel_with_the_cursor_row_marked_and_highlighted() {
    let mut app = app_of(&["Pay +rent", "Call +bank @phone", "x 2026-09-20 Old +rent"]);
    app.cursor = 1;

    let buffer = render(&app);

    assert_eq!(
        rows(&buffer)[..4],
        [
            "▸ All tasks       2│    1  Pay +rent",
            "                   │▸   2  Call +bank @phone",
            " PROJECTS          │",
            "  +bank           1│",
        ]
    );
    assert_eq!(buffer[(30, 1)].bg, Color::DarkGray);
    assert_eq!(buffer[(30, 0)].bg, BACKGROUND);
    assert!(buffer[(2, 0)].modifier.contains(Modifier::BOLD));
    assert!(!buffer[(2, 3)].modifier.contains(Modifier::BOLD));
}

#[test]
fn the_active_filter_is_marked_in_the_panel_and_highlighted_once_the_panel_has_focus() {
    let mut app = app_of(&["Pay +rent", "Call +bank"]);
    app.filter = Some("+rent".to_string());

    let buffer = render_in(&app, 50, 8);
    assert!(rows(&buffer)[0].starts_with("  All tasks "));
    assert!(rows(&buffer)[4].starts_with("▸ +rent "));
    assert_eq!(buffer[(5, 4)].bg, BACKGROUND);

    app.focus = Focus::Panel;
    assert_eq!(render_in(&app, 50, 8)[(5, 4)].bg, Color::DarkGray);
}

#[test]
fn a_filter_gone_from_the_panel_marks_no_entry() {
    let mut app = app_of(&["Pay +rent", "x 2026-09-20 Call +bank"]);
    app.filter = Some("+bank".to_string());
    app.focus = Focus::Panel;

    let buffer = render_in(&app, 50, 8);
    assert!(rows(&buffer).iter().all(|row| !row.starts_with('▸')));
    assert_eq!(buffer[(5, 0)].bg, BACKGROUND);
}

#[test]
fn the_panel_puts_projects_and_contexts_under_coloured_headers_and_drops_an_empty_section() {
    let buffer = render_in(&app_of(&["Pay +rent", "Call +bank @phone"]), 50, 9);
    let panel: Vec<String> = rows(&buffer)
        .iter()
        .map(|row| row.chars().take(19).collect::<String>().trim_end().to_string())
        .collect();

    assert_eq!(
        panel[..8],
        [
            "▸ All tasks       2",
            "",
            " PROJECTS",
            "  +bank           1",
            "  +rent           1",
            "",
            " CONTEXTS",
            "  @phone          1"
        ]
    );
    assert_eq!((buffer[(1, 2)].fg, buffer[(1, 6)].fg), (Color::Magenta, Color::Cyan));
    assert!(buffer[(1, 2)].modifier.contains(Modifier::BOLD));
    assert_eq!(
        (buffer[(2, 3)].fg, buffer[(2, 7)].fg, buffer[(2, 0)].fg),
        (Color::Magenta, Color::Cyan, Color::Reset)
    );
    assert!(buffer[(18, 3)].modifier.contains(Modifier::DIM));

    let contexts_only = rows(&render_in(&app_of(&["Call @phone"]), 50, 9));
    assert_eq!(contexts_only[2].chars().take(9).collect::<String>(), " CONTEXTS");
    assert!(!contexts_only.iter().any(|row| row.contains("PROJECTS")));
}

#[test]
fn group_headers_sit_above_their_tasks_with_the_cursor_on_the_same_task() {
    let mut app = app_of(&["c", "(B) b", "(A) a"]);
    app.cursor = 1;

    let buffer = render_in(&app, 50, 10);
    let list: Vec<String> = rows(&buffer).iter().map(|row| row.chars().skip(20).collect()).collect();

    assert_eq!(
        list[..8],
        [
            format!("   (1)  PRIORITY A  {}", "─".repeat(10)),
            "    3  (A) a".to_string(),
            String::new(),
            format!("   (1)  PRIORITY B  {}", "─".repeat(10)),
            "▸   2  (B) b".to_string(),
            String::new(),
            format!("   (1)  NO PRIORITY  {}", "─".repeat(9)),
            "    1  c".to_string(),
        ]
    );
    assert_eq!(buffer[(30, 4)].bg, Color::DarkGray);
    assert_eq!((buffer[(28, 0)].fg, buffer[(28, 3)].fg), (Color::Yellow, Color::Green));
    assert!(buffer[(28, 6)].modifier.contains(Modifier::DIM));
    assert!(buffer[(49, 0)].modifier.contains(Modifier::DIM));
}

#[test]
fn a_folded_group_is_its_header_alone_ended_by_a_marker_and_the_cursor_can_stand_on_it() {
    let mut app = app_of(&["(A) a", "(A) b", "(B) c", "d"]);
    app.folded = vec![Group::Priority('A'), Group::Priority('B'), Group::Unprioritised];
    app.cursor = 1;

    let buffer = render_in(&app, 50, 8);
    let list: Vec<String> = rows(&buffer).iter().map(|row| row.chars().skip(20).collect()).collect();

    assert_eq!(
        list[..6],
        [
            format!("   (2)  PRIORITY A  {} ▸", "─".repeat(8)),
            String::new(),
            format!("▸  (1)  PRIORITY B  {} ▸", "─".repeat(8)),
            String::new(),
            format!("   (1)  NO PRIORITY  {} ▸", "─".repeat(7)),
            String::new(),
        ]
    );
    assert_eq!(buffer[(30, 2)].bg, Color::DarkGray);
}

#[test]
fn the_help_shows_each_mode_s_keys_under_its_block_keys_bold_actions_and_slashes_dimmed() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help;

    let buffer = render_in(&app, 90, 32);
    let rows = rows(&buffer);
    let y = rows.iter().position(|row| row.contains(" LIST ")).expect("list block");

    assert!(rows[y - 2].contains("╭ HELP ─"), "{}", rows[y - 2]);
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
    assert_eq!((buffer[(8, y)].bg, buffer[(46, y)].bg), (Color::Blue, Color::Green));
    let (key, slash, action) = (&buffer[(8, y + 1)], &buffer[(9, y + 1)], &buffer[(21, y + 1)]);
    assert!(key.modifier.contains(Modifier::BOLD) && !key.modifier.contains(Modifier::DIM));
    assert!(slash.modifier.contains(Modifier::DIM) && !slash.modifier.contains(Modifier::BOLD));
    assert!(action.modifier.contains(Modifier::DIM) && !action.modifier.contains(Modifier::BOLD));
    let search = rows.iter().position(|row| row.contains("/            search")).expect("search key") as u16;
    assert!(!buffer[(8, search)].modifier.contains(Modifier::DIM));
}

#[test]
fn every_cell_left_unpainted_takes_the_background_even_under_a_cleared_box() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Help;

    let buffer = render_in(&app, 80, 30);

    assert_eq!(buffer[(79, 1)].bg, BACKGROUND);
    assert_eq!(buffer[(40, 15)].bg, BACKGROUND);
    assert!(buffer.content.iter().all(|cell| cell.bg != Color::Reset));
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
    assert_eq!((buffer[(1, 4)].bg, buffer[(1, 4)].fg), (Color::Blue, Color::Black));
    assert!(buffer[(1, 4)].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(7, 4)].bg, BACKGROUND);
}

#[test]
fn the_mode_keys_follow_when_they_fit_whole_and_no_message_is_shown() {
    let mut app = app_of(&["Pay +rent"]);
    app.focus = Focus::Panel;

    let buffer = render(&app);
    assert_eq!(rows(&buffer)[4], " PANEL  j/k filter · esc all tasks · tab back");
    assert_eq!(buffer[(1, 4)].bg, Color::Magenta);
    assert!(buffer[(8, 4)].modifier.contains(Modifier::BOLD));
    assert!(!buffer[(8, 4)].modifier.contains(Modifier::DIM));
    assert!(buffer[(12, 4)].modifier.contains(Modifier::DIM));
    assert!(!buffer[(12, 4)].modifier.contains(Modifier::BOLD));

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
    assert_eq!(buffer[(1, 4)].bg, Color::Yellow);
}

#[test]
fn the_popup_wraps_its_text_under_its_title_with_the_cursor_cell_reversed_while_the_status_bar_says_insert() {
    let mut app = app_of(&["Pay +rent"]);
    app.filter = Some("+rent".to_string());
    let mut editor = Editor::default();
    editor.text = "Call the bank about the loan and ask for a quote".to_string();
    editor.cursor = 5;
    app.focus = Focus::Popup(Popup {
        editor,
        target: Target::Add,
        selected: 0,
    });

    let buffer = render(&app);
    let rows = rows(&buffer);

    assert!(rows[0].contains("┌ ADD (+rent) ───"), "{}", rows[0]);
    assert!(rows[1].contains("│Call the bank about the loan and ask  │"), "{}", rows[1]);
    assert!(rows[2].contains("│for a quote                           │"), "{}", rows[2]);
    assert!(rows[3].contains("└───"), "{}", rows[3]);
    assert_eq!(buffer[(11, 1)].symbol(), "t");
    assert!(buffer[(11, 1)].modifier.contains(Modifier::REVERSED));
    assert!(!buffer[(10, 1)].modifier.contains(Modifier::REVERSED));
    assert_eq!(rows[4], " INSERT  +rent  esc normal · ⏎ save");
    assert_eq!(buffer[(1, 4)].bg, Color::Green);

    if let Focus::Popup(popup) = &mut app.focus {
        popup.editor.mode = Mode::Normal;
    }
    let buffer = render_in(&app, 70, 5);
    assert_eq!(self::rows(&buffer)[4], " NORMAL  +rent  i insert · p priority · ⏎ save · esc cancel");
    assert_eq!(buffer[(1, 4)].bg, Color::Blue);
}

fn add_popup(app: &mut App, text: &str, selected: usize) {
    let mut editor = Editor::default();
    editor.text = text.to_string();
    editor.cursor = text.chars().count();
    app.focus = Focus::Popup(Popup {
        editor,
        target: Target::Add,
        selected,
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
        ["┌──────────┐", "│+bank    2│", "│+books   1│", "└──────────┘"]
    );
    let y = y as u16;
    assert_eq!(buffer[(10, y + 2)].fg, Color::Magenta);
    assert!(buffer[(19, y + 2)].modifier.contains(Modifier::DIM));
    assert_eq!((buffer[(10, y + 2)].bg, buffer[(10, y + 3)].bg), (BACKGROUND, Color::DarkGray));
    assert_eq!(rows[11], " INSERT  esc normal · ⏎ save · tab complete");
}

#[test]
fn the_completions_go_above_the_tag_when_there_is_no_room_below() {
    let mut app = app_of(&["Pay +bank", "Call @phone"]);
    add_popup(&mut app, "Call the bank about the loan and ask @ph", 0);

    let rows = rows(&render_in(&app, 50, 7));
    let y = rows.iter().position(|row| row.contains("│@ph ")).expect("tag line");

    assert_eq!(
        (y - 3..y).map(|y| cells(&rows[y], 5, 12)).collect::<Vec<_>>(),
        ["┌──────────┐", "│@phone   1│", "└──────────┘"]
    );
}

fn style_of(line_text: &str, token: &str) -> Style {
    let line = line(1, &Todo::from_line(line_text));
    line.iter().find(|span| span.content == token).expect(token).style
}

#[test]
fn a_pending_line_keeps_its_text_and_styles_priority_date_projects_and_contexts() {
    let text = "(A) 2026-09-26 Call  +bank @phone due:2026-10-01 a+b + @";

    assert_eq!(line(12, &Todo::from_line(text)).to_string(), format!(" 12  {text}"));
    assert_eq!(style_of(text, "(A)"), Style::new().bold().yellow());
    assert_eq!(style_of(text, "2026-09-26"), Style::new().dim());
    assert_eq!(style_of(text, "+bank"), Style::new().magenta());
    assert_eq!(style_of(text, "@phone"), Style::new().cyan());
    for plain in ["Call", "due:2026-10-01", "a+b", "+", "@"] {
        assert_eq!(style_of(text, plain), Style::new(), "{plain}");
    }
    assert_eq!(style_of("(B) x", "(B)"), Style::new().bold().green());
    assert_eq!(style_of("(C) x", "(C)"), Style::new().bold().blue());
    assert_eq!(style_of("(D) x", "(D)"), Style::new().bold());
}

#[test]
fn a_done_line_is_dimmed_all_through() {
    let line = line(3, &Todo::from_line("x 2026-09-26 2026-09-20 Call +bank @phone"));

    assert_eq!(line.to_string(), "  3  x 2026-09-26 2026-09-20 Call +bank @phone");
    assert!(line.iter().all(|span| span.style == Style::new().dim()));
}

#[test]
fn projects_and_contexts_are_coloured_on_screen() {
    let buffer = render(&app_of(&["Call +bank @phone"]));

    assert_eq!(rows(&buffer)[0], "▸ All tasks       1│▸   1  Call +bank @phone");
    assert_eq!(buffer[(32, 0)].fg, Color::Magenta);
    assert_eq!(buffer[(38, 0)].fg, Color::Cyan);
    assert_eq!(buffer[(27, 0)].fg, Color::Reset);
}
