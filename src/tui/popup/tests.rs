use super::super::tests::{TODAY, app, app_of, lines, moved, popup, press, shown, told};
use super::super::*;

#[test]
fn o_types_a_task_that_enter_adds_with_the_cursor_on_it() {
    let mut app = app();

    assert!(!press(&mut app, "jjo(A) Ask for a quote"));
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(shown(&app), ["(A) 2026-09-26 Ask for a quote", "one", "two", "three"]);
    assert_eq!(app.cursor, 0);
    assert!(!app.quit);
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn backspace_edits_the_input_and_esc_twice_drops_it() {
    let mut app = app();
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "oab");
    app.handle_key(KeyEvent::from(KeyCode::Backspace), TODAY);
    press(&mut app, "c");
    assert_eq!(popup(&app).editor.text, "ac");

    assert!(!app.handle_key(esc, TODAY));
    assert_eq!(popup(&app).editor.mode, Mode::Normal);
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn enter_in_normal_mode_adds_the_task_too() {
    let mut app = app();

    press(&mut app, "oCall bank");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "bithe ");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));
    assert_eq!(app.todos[3].to_line(), "2026-09-26 Call the bank");
}

#[test]
fn list_keys_typed_in_the_popup_are_text() {
    let mut app = app();

    assert!(!press(&mut app, "oqxdd"));
    assert!(!app.quit);
    assert_eq!(shown(&app), ["one", "two", "three"]);
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.todos[3].to_line(), "2026-09-26 qxdd");
}

#[test]
fn enter_opens_the_task_line_in_normal_mode_and_the_cursor_follows_the_edited_task() {
    let mut app = app_of(&["one", "two", "three"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    assert!(!press(&mut app, "jj"));
    app.handle_key(enter, TODAY);
    let editor = &popup(&app).editor;
    assert_eq!((editor.text.as_str(), editor.cursor, editor.mode), ("three", 0, Mode::Normal));

    press(&mut app, "i(A) ");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(shown(&app), ["(A) three", "one", "two"]);
    assert_eq!(app.cursor, 0);
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn a_completion_date_typed_with_the_x_is_the_one_the_task_moves_to_done_txt_with() {
    let mut app = app_of(&["one", "two"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "j");
    app.handle_key(enter, TODAY);
    press(&mut app, "ix 2026-09-20 ");
    assert!(app.handle_key(enter, TODAY));

    assert_eq!(lines(&app), ["one"]);
    assert_eq!(moved(&app), ["+x 2026-09-20 two"]);
    assert_eq!(app.cursor, 0);
}

#[test]
fn a_line_holding_a_control_character_opens_for_edit_with_a_space_in_its_place() {
    let mut app = app_of(&["Call\tthe bank"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(app.todos[0].to_line(), "Call the bank");
}

#[test]
fn an_edit_emptied_or_left_unchanged_writes_nothing() {
    let mut app = app_of(&["(A) one"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    app.handle_key(enter, TODAY);
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(told(&app), Some(Notice::NoChanges));

    app.handle_key(enter, TODAY);
    press(&mut app, "WD");
    assert_eq!(popup(&app).editor.text, "(A) ");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(
        told(&app),
        Some(Notice::Refused("a task needs more than +projects, @contexts and key: words".to_string()))
    );
    assert_eq!(popup(&app).editor.text, "(A) ");
    assert_eq!(shown(&app), ["(A) one"]);
}

#[test]
fn a_trailing_space_is_dropped_on_add_and_edit() {
    let mut app = app_of(&["one"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "oShip ");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(shown(&app), ["one", "2026-09-26 Ship"]);

    app.handle_key(enter, TODAY);
    press(&mut app, "A ");
    assert!(!app.handle_key(enter, TODAY));

    app.handle_key(enter, TODAY);
    press(&mut app, "A it ");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(shown(&app), ["one", "2026-09-26 Ship it"]);
}

#[test]
fn a_rejected_input_adds_nothing_and_says_why() {
    let mut app = app();

    press(&mut app, "o");
    assert!(!app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(
        told(&app),
        Some(Notice::Refused("a task needs more than +projects, @contexts and key: words".to_string()))
    );
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn a_refused_line_keeps_the_popup_open_with_its_text_and_esc_closes_it_writing_nothing() {
    let mut app = app();
    let enter = KeyEvent::from(KeyCode::Enter);
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "o");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(
        told(&app),
        Some(Notice::Refused("a task needs more than +projects, @contexts and key: words".to_string()))
    );
    assert_eq!(popup(&app).editor.text, "");

    press(&mut app, "x Lorem");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(told(&app), Some(Notice::Refused("cannot add a task that is already done".to_string())));
    assert_eq!(popup(&app).editor.text, "x Lorem");

    app.handle_key(esc, TODAY);
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn an_accepted_add_closes_the_popup() {
    let mut app = app();

    press(&mut app, "oLorem");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
}

#[test]
fn an_add_fixed_after_a_refusal_does_not_bring_the_refusal_back() {
    let mut app = app();
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "o");
    assert!(!app.handle_key(enter, TODAY));
    assert!(app.refusal_shown());

    press(&mut app, "Lorem");
    assert!(app.handle_key(enter, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(told(&app), Some(Notice::Added { hidden: false }));
}

#[test]
fn a_toast_shown_before_an_accepted_add_is_replaced_by_the_add_s() {
    let mut app = app();
    app.notify(Notice::Copied);

    press(&mut app, "oLorem");
    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(told(&app), Some(Notice::Added { hidden: false }));
}

#[test]
fn a_toast_shown_before_a_popup_closed_with_nothing_to_say_is_still_there_after_it() {
    let mut app = app();
    app.notify(Notice::Copied);

    press(&mut app, "o");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert_eq!(told(&app), Some(Notice::Copied));
}

#[test]
fn a_refusal_shown_before_a_popup_closed_with_nothing_to_say_is_taken_down_with_it() {
    let mut app = app();
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "o");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    assert!(app.refusal_shown());
    app.handle_key(esc, TODAY);
    app.handle_key(esc, TODAY);

    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(told(&app), None);
}

#[test]
fn esc_on_an_add_or_an_edit_whose_text_changed_says_the_changes_are_discarded() {
    let mut app = app_of(&["one"]);
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "oLorem");
    app.handle_key(esc, TODAY);
    assert!(!app.handle_key(esc, TODAY));
    assert_eq!(told(&app), Some(Notice::Discarded));
    assert_eq!(Notice::Discarded.kind(), Kind::Info);

    app.toast = None;
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "ix");
    app.handle_key(esc, TODAY);
    assert!(!app.handle_key(esc, TODAY));
    assert_eq!(told(&app), Some(Notice::Discarded));
}

#[test]
fn esc_on_an_empty_add_or_an_edit_without_change_leaves_no_toast() {
    let mut app = app_of(&["one"]);
    let esc = KeyEvent::from(KeyCode::Esc);

    press(&mut app, "o");
    app.handle_key(esc, TODAY);
    assert!(matches!(app.focus, Focus::Popup(_)));
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(told(&app), None);

    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    assert!(matches!(popup(&app).target, Target::Edit(1)));
    assert!(!app.handle_key(esc, TODAY));
    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(told(&app), None);
}

#[test]
fn enter_on_an_unchanged_line_says_there_is_nothing_to_save() {
    let mut app = app_of(&["one"]);

    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    assert!(!app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(told(&app), Some(Notice::NoChanges));
    assert_eq!(Notice::NoChanges.kind(), Kind::Info);
}

#[test]
fn a_saved_edit_says_updated_and_a_line_starting_with_x_says_done() {
    let mut app = app_of(&["one", "two"]);
    let enter = KeyEvent::from(KeyCode::Enter);

    app.handle_key(enter, TODAY);
    press(&mut app, "Atwo");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(told(&app), Some(Notice::Updated { hidden: false }));
    assert_eq!(Notice::Updated { hidden: false }.kind(), Kind::Success);

    app.handle_key(enter, TODAY);
    press(&mut app, "ix ");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(told(&app), Some(Notice::Done));
}

#[test]
fn a_paste_types_its_lines_as_one_into_the_popup_or_the_search_and_does_nothing_in_the_list() {
    let mut app = app();

    app.paste("dd");
    assert_eq!(shown(&app), ["one", "two", "three"]);

    press(&mut app, "oCall ");
    app.paste("the\r\nbank\n");
    assert_eq!(popup(&app).editor.text, "Call the bank");
    assert_eq!(popup(&app).editor.cursor, 13);
    app.paste(" on\rMonday");
    assert_eq!(popup(&app).editor.text, "Call the bank on Monday");
    app.paste(" at\tnoon\u{1b}");
    assert_eq!(popup(&app).editor.text, "Call the bank on Monday at noon");

    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "/");
    app.paste("tw\no");
    assert_eq!(app.search, "tw o");
}

#[test]
fn an_x_typed_in_front_of_a_task_moves_it_to_done_txt_completed_today_and_u_brings_it_back() {
    let mut app = app_of(&["(A) 2026-08-01 one", "two"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "ix ");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(lines(&app), ["two"]);
    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one"]);
    assert!(press(&mut app, "u"));
    assert_eq!(lines(&app), ["(A) 2026-08-01 one", "two"]);
    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one", "-x 2026-09-26 2026-08-01 one"]);
}

#[test]
fn an_x_typed_in_front_of_a_dated_task_keeps_its_creation_date() {
    let mut app = app_of(&["2026-08-01 one"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    press(&mut app, "ix ");

    assert!(app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY));

    assert_eq!(moved(&app), ["+x 2026-09-26 2026-08-01 one"]);
}

#[test]
fn u_in_the_popup_undoes_the_typed_text_and_says_so_without_touching_the_list() {
    let mut app = app();
    press(&mut app, "ofour");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert!(!press(&mut app, "u"));
    assert_eq!(popup(&app).editor.text, "");
    assert_eq!((told(&app), app.refusal_shown()), (Some(Notice::Undone), false));

    press(&mut app, "u");
    assert_eq!(told(&app), Some(Notice::NothingToUndo));
    assert_eq!(shown(&app), ["one", "two", "three"]);
}

#[test]
fn a_line_of_only_tags_is_refused_unchanged_ignored_and_ended_by_a_typed_x() {
    let enter = KeyEvent::from(KeyCode::Enter);
    let mut app = app_of(&["+work"]);

    app.handle_key(enter, TODAY);
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(told(&app), Some(Notice::NoChanges));

    app.handle_key(enter, TODAY);
    press(&mut app, "A wait:a");
    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(
        told(&app),
        Some(Notice::Refused("a task needs more than +projects, @contexts and key: words".to_string()))
    );
    assert_eq!(shown(&app), ["+work"]);

    let mut app = app_of(&["+work"]);
    app.handle_key(enter, TODAY);
    press(&mut app, "Ix ");
    assert_eq!(popup(&app).editor.text, "x +work");
    assert!(app.handle_key(enter, TODAY));
    assert_eq!(moved(&app), ["+x 2026-09-26 +work"]);
}

#[test]
fn a_line_replaced_by_a_lone_x_is_refused() {
    let enter = KeyEvent::from(KeyCode::Enter);
    let mut app = app_of(&["one"]);
    app.handle_key(enter, TODAY);
    press(&mut app, "0Dix ");
    assert_eq!(popup(&app).editor.text, "x ");

    assert!(!app.handle_key(enter, TODAY));
    assert_eq!(
        told(&app),
        Some(Notice::Refused("a task needs more than +projects, @contexts and key: words".to_string()))
    );
    assert_eq!(shown(&app), ["one"]);
    assert!(moved(&app).is_empty());
}

#[test]
fn a_paste_and_an_edit_treat_a_bidi_character_like_a_control_character() {
    let mut app = app_of(&["Call\u{202e}the\u{200b}bank"]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    assert_eq!(app.todos[0].to_line(), "Call the bank");

    press(&mut app, "/");
    app.paste("a\u{202e}b");
    assert_eq!(app.search, "a b");
}

const COPIED: &str = "(A) 2026-11-02 Call the bank about the loan +finance due:2026-11-20";

fn opened(line: &str) -> App {
    let mut app = app_of(&[line]);
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);
    app
}

#[test]
fn y_with_a_motion_or_an_object_leaves_what_d_would_delete_to_copy() {
    let cases = [
        ("fCyiw", "Call"),
        ("fCyaw", "Call "),
        ("fCyiW", "Call"),
        ("fCyaW", "Call "),
        ("fCyw", "Call "),
        ("fCyW", "Call "),
        ("fCye", "Call"),
        ("fCyE", "Call"),
        ("fbyb", "the "),
        ("fbyB", "the "),
        ("fCy0", "(A) 2026-11-02 "),
        ("fCyh", " "),
        ("fCyl", "C"),
        ("fCyt+", "Call the bank about the loan "),
        ("fCyf+", "Call the bank about the loan +"),
        ("$yT:", "2026-11-2"),
        ("fCyfty;", "Call t"),
        ("fCy$", &COPIED[15..]),
        ("yy", COPIED),
    ];

    for (keys, copied) in cases {
        let mut app = opened(COPIED);
        press(&mut app, keys);
        assert_eq!((keys, app.copy.as_ref().map(Clip::text)), (keys, Some(copied)));
        assert_eq!(popup(&app).editor.text, COPIED);
    }
}

#[test]
fn y_puts_the_cursor_at_the_start_of_the_range_and_u_does_not_see_the_copy() {
    let mut app = opened(COPIED);
    press(&mut app, "fbyb");
    assert_eq!(popup(&app).editor.cursor, 20);

    press(&mut app, "xyiwu");
    assert_eq!(popup(&app).editor.text, COPIED);
}

#[test]
fn a_copy_from_the_popup_says_so_a_failure_is_refused_and_yy_in_the_list_keeps_its_words() {
    let mut app = opened(COPIED);
    press(&mut app, "yy");
    let clip = app.copy.take().unwrap();
    app.copied(&clip, Ok(()));
    assert_eq!(told(&app), Some(Notice::CopiedText));

    app.copied(&clip, Err(io::Error::other("boom")));
    assert_eq!(told(&app), Some(Notice::Refused("could not copy: boom".to_string())));

    let mut app = app_of(&["one"]);
    press(&mut app, "yy");
    let clip = app.copy.take().unwrap();
    app.copied(&clip, Ok(()));
    assert_eq!(told(&app), Some(Notice::Copied));
}

#[test]
fn an_empty_range_copies_nothing_and_says_so() {
    let mut app = opened("one");
    press(&mut app, "ddyy");
    assert_eq!((app.copy.clone(), told(&app)), (None, Some(Notice::NothingToCopy)));
    assert!(app.refusal_shown());

    app.toast = None;
    press(&mut app, "y$");
    assert_eq!((app.copy.clone(), told(&app)), (None, Some(Notice::NothingToCopy)));
}

#[test]
fn esc_drops_a_pending_y_and_keeps_the_popup_open() {
    let mut app = opened(COPIED);
    press(&mut app, "y");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert!(matches!(app.focus, Focus::Popup(_)));
    assert_eq!((popup(&app).editor.pending().as_str(), app.copy), ("", None));
}

#[test]
fn deleting_leaves_nothing_to_copy_and_p_still_sets_the_priority() {
    let mut app = opened(COPIED);
    press(&mut app, "fCdwxDC");
    assert_eq!(app.copy, None);

    let mut app = opened(COPIED);
    press(&mut app, "pb");
    assert_eq!(popup(&app).editor.text, COPIED.replacen("(A)", "(B)", 1));
    assert_eq!(app.copy, None);
}

#[test]
fn y_copies_the_text_as_it_is_on_screen_unsaved_changes_included() {
    let mut app = opened("one");
    press(&mut app, "Atwo");
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);
    press(&mut app, "yy");

    assert_eq!(app.copy.as_ref().map(Clip::text), Some("onetwo"));
}

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::from(code), TODAY);
}

#[test]
fn v_selects_from_the_character_under_the_cursor_and_y_copies_both_ends_in() {
    let cases = [
        ("fCvy", "C"),
        ("fCvey", "Call"),
        ("fCvwwy", "Call the b"),
        ("fCvhy", " C"),
        ("fCvly", "Ca"),
        ("fCv0y", &COPIED[..16]),
        ("fCv$y", &COPIED[15..]),
        ("fCvwwWy", "Call the bank a"),
        ("fbvby", "the b"),
        ("$vby", "20"),
        ("fCvBy", &COPIED[4..16]),
        ("fCvt+y", "Call the bank about the loan "),
        ("fCvf+y", "Call the bank about the loan +"),
        ("$vT:y", "2026-11-20"),
        ("fCvfty", "Call t"),
        ("fCvft;y", "Call the bank about"),
        ("fCvft;,y", "Call t"),
    ];

    for (keys, copied) in cases {
        let mut app = opened(COPIED);
        press(&mut app, keys);
        assert_eq!((keys, app.copy.as_ref().map(Clip::text)), (keys, Some(copied)));
        assert_eq!(popup(&app).editor.text, COPIED);
    }
}

#[test]
fn y_in_visual_mode_goes_back_to_normal_with_the_cursor_at_the_start_and_says_so() {
    let mut app = opened(COPIED);
    press(&mut app, "fbvby");

    assert_eq!((popup(&app).editor.mode, popup(&app).editor.cursor), (Mode::Normal, 20));
    let clip = app.copy.take().unwrap();
    app.copied(&clip, Ok(()));
    assert_eq!(told(&app), Some(Notice::CopiedText));

    press(&mut app, "xvey");
    press(&mut app, "u");
    assert_eq!(popup(&app).editor.text, COPIED);
}

#[test]
fn y_in_visual_mode_copies_the_text_as_it_is_on_screen_and_an_empty_one_says_nothing_to_copy() {
    let mut app = opened("one");
    press(&mut app, "Atwo");
    key(&mut app, KeyCode::Esc);
    press(&mut app, "0v$y");
    assert_eq!(app.copy.as_ref().map(Clip::text), Some("onetwo"));

    let mut app = opened("one");
    press(&mut app, "ddvy");
    assert_eq!((app.copy.clone(), told(&app)), (None, Some(Notice::NothingToCopy)));
    assert_eq!(popup(&app).editor.mode, Mode::Normal);
}

#[test]
fn esc_or_v_go_back_to_normal_without_a_copy_and_enter_saves() {
    for end in [KeyCode::Esc, KeyCode::Char('v')] {
        let mut app = opened(COPIED);
        press(&mut app, "fCve");
        key(&mut app, end);

        assert!(matches!(app.focus, Focus::Popup(_)));
        assert_eq!((popup(&app).editor.mode, app.copy.clone()), (Mode::Normal, None));
        assert_eq!(popup(&app).editor.cursor, 18);
    }

    let mut app = opened(COPIED);
    press(&mut app, "fCvf");
    key(&mut app, KeyCode::Esc);
    assert_eq!((popup(&app).editor.mode, popup(&app).editor.pending().as_str()), (Mode::Normal, ""));

    let mut app = opened(COPIED);
    press(&mut app, "ve");
    key(&mut app, KeyCode::Enter);
    assert!(matches!(app.focus, Focus::List));
    assert_eq!(told(&app), Some(Notice::NoChanges));
}

#[test]
fn any_other_key_in_visual_mode_changes_nothing_and_the_selection_stays() {
    let mut app = opened(COPIED);
    press(&mut app, "fCv");
    let before = (popup(&app).editor.text.clone(), popup(&app).editor.cursor);

    press(&mut app, "dcxpiauoIADCuYV");
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL), TODAY);
    app.paste("zz");

    assert_eq!((popup(&app).editor.text.clone(), popup(&app).editor.cursor), before);
    assert_eq!((popup(&app).editor.mode, app.copy.clone()), (Mode::Visual, None));

    press(&mut app, "ey");
    assert_eq!(app.copy.as_ref().map(Clip::text), Some("Call"));
}

#[test]
fn only_y_leaves_a_text_to_copy_in_visual_mode() {
    let mut app = opened(COPIED);
    press(&mut app, "fCvewbBeEh0$ft;,");
    assert_eq!(app.copy, None);
}

#[test]
fn question_mark_in_visual_mode_opens_the_help_and_closing_it_goes_back_to_visual() {
    let mut app = opened(COPIED);
    press(&mut app, "fCve?");
    assert!(matches!(app.focus, Focus::Help(_)));

    key(&mut app, KeyCode::Esc);
    assert_eq!(popup(&app).editor.mode, Mode::Visual);
}
