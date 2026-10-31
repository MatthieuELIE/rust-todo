use super::*;
use time::macros::date;

#[test]
fn a_task_made_only_of_non_text_words_is_refused() {
    for line in [
        "wait:designer",
        "+work @home",
        "due:2026-10-20 +work",
        "wait://figma",
        "wait:",
        "https://example.com",
        "(A)",
        "2026-10-09",
        "",
        "+work (B) 2026-10-09 due:",
    ] {
        assert!(Todo::from_input(line).unwrap().check_text().is_err(), "{line}");
    }
    for line in [
        "Appeler +banque",
        "Appeler 2026-10-09",
        "Lire https://example.com",
        "10:30",
        "+",
        "(F)",
        "@",
    ] {
        assert!(Todo::from_input(line).unwrap().check_text().is_ok(), "{line}");
    }
}

#[test]
fn a_done_task_of_only_non_text_words_is_not_reopened() {
    for line in ["x 2026-10-01", "x 2026-10-01 +work"] {
        assert!(Todo::from_line(line).reopen().is_err(), "{line}");
    }
}

#[test]
fn from_line_extracts_the_marker_priority_and_description() {
    let task = Todo::from_line("x (C) Lorem ipsum dolor sit amet");
    assert_eq!(task.description, "Lorem ipsum dolor sit amet");
    assert!(task.done);
    assert_eq!(task.priority, Some('C'));
}

#[test]
fn a_plain_line_is_just_the_description() {
    let task = Todo::from_line("Lorem ipsum dolor sit amet");
    assert_eq!(task.description, "Lorem ipsum dolor sit amet");
    assert!(!task.done);
    assert_eq!(task.priority, None);
    assert_eq!(task.to_line(), "Lorem ipsum dolor sit amet");
}

#[test]
fn an_out_of_range_priority_stays_in_the_description() {
    let task = Todo::from_line("(Z) Lorem ipsum dolor sit amet");
    assert_eq!(task.description, "(Z) Lorem ipsum dolor sit amet");
    assert_eq!(task.priority, None);

    let lowercase = Todo::from_line("(a) Lorem ipsum");
    assert_eq!((lowercase.description.as_str(), lowercase.priority), ("(a) Lorem ipsum", None));
}

#[test]
fn a_canonical_line_with_unrecognised_tokens_survives_a_round_trip() {
    let line = "x 2026-09-03 2026-09-01 Lorem ipsum +consectetur @adipiscing due:2026-01-01";
    assert_eq!(Todo::from_line(line).to_line(), line);
}

#[test]
fn serialising_a_done_task_drops_its_priority() {
    assert_eq!(Todo::from_line("x (A) Lorem ipsum").to_line(), "x Lorem ipsum");
}

#[test]
fn from_line_reads_the_creation_date_after_the_priority() {
    let task = Todo::from_line("(A) 2026-09-01 Lorem ipsum");
    assert_eq!(task.priority, Some('A'));
    assert_eq!(task.created, Some(date!(2026 - 09 - 01)));
    assert_eq!(task.description, "Lorem ipsum");
}

#[test]
fn from_line_reads_the_completion_then_the_creation_date_on_a_done_line() {
    let task = Todo::from_line("x 2026-09-03 2026-09-01 Lorem ipsum");
    assert!(task.done);
    assert_eq!(task.completed, Some(date!(2026 - 09 - 03)));
    assert_eq!(task.created, Some(date!(2026 - 09 - 01)));
    assert_eq!(task.description, "Lorem ipsum");
}

#[test]
fn a_done_line_may_carry_only_a_completion_date() {
    let task = Todo::from_line("x 2026-09-03 Lorem ipsum");
    assert_eq!(task.completed, Some(date!(2026 - 09 - 03)));
    assert_eq!(task.created, None);
    assert_eq!(task.description, "Lorem ipsum");
}

#[test]
fn an_invalid_date_stays_in_the_description() {
    let task = Todo::from_line("2026-99-99 Lorem ipsum");
    assert_eq!(task.created, None);
    assert_eq!(task.description, "2026-99-99 Lorem ipsum");
}

#[test]
fn projects_and_contexts_are_named_without_their_sigil_once_each_in_order_of_appearance() {
    let task = Todo::from_line("(A) Call +bank @phone about +Bank and +bank a+b + @phone @home");

    assert_eq!(task.projects(), ["bank", "Bank"]);
    assert_eq!(task.contexts(), ["phone", "home"]);
}

#[test]
fn the_text_is_the_description_without_its_trailing_key_values() {
    let text = |description: &str| Todo::from_line(description).text().to_string();

    assert_eq!(
        text("Appeler +client pour le devis @work wait:figma"),
        "Appeler +client pour le devis @work"
    );
    assert_eq!(text("Pay rent +home  @web "), "Pay rent +home  @web");
    assert_eq!(text("Meet at 10:30"), "Meet at 10:30");
    assert_eq!(text("Read http://example.com"), "Read http://example.com");
    assert_eq!(text("Call a+b + Note:"), "Call a+b + Note:");
    assert_eq!(text("Call +"), "Call +");
    assert_eq!(text("Mail @"), "Mail @");
    assert_eq!(text("+rent @home due:2026-10-01"), "+rent @home");
    assert_eq!(text("Mail +work @home due:2026-10-20"), "Mail +work @home");
    assert_eq!(text("Appeler pour le devis +client"), "Appeler pour le devis +client");
    assert_eq!(text("Mail +work due:2026-10-20 @home"), "Mail +work due:2026-10-20 @home");
}

#[test]
fn completing_a_task_records_the_day_before_its_creation_date_and_drops_the_priority() {
    let mut dated = Todo::from_line("(A) 2026-08-01 Lorem ipsum");
    dated.complete(date!(2026 - 09 - 01));
    assert_eq!(dated.to_line(), "x 2026-09-01 2026-08-01 Lorem ipsum");

    let mut undated = Todo::from_line("Lorem ipsum");
    undated.complete(date!(2026 - 09 - 01));
    assert_eq!(undated.to_line(), "x 2026-09-01 Lorem ipsum");
}

fn completed_line(line: &str) -> String {
    let mut task = Todo::from_line(line);
    task.complete(date!(2026 - 09 - 01));
    task.description
}

#[test]
fn completing_a_task_drops_its_waits_with_the_space_before_each() {
    assert_eq!(completed_line("Mail wait:designer +work"), "Mail +work");
    assert_eq!(completed_line("Mail wait:a  wait:b due:2026-10-01"), "Mail  due:2026-10-01");
    assert_eq!(completed_line("wait:a Mail"), "Mail");
    assert_eq!(completed_line("Café\twait:a +x"), "Café +x");
    assert_eq!(completed_line("wait:a wait:b"), "");
}

#[test]
fn completing_leaves_a_task_without_wait_and_the_empty_or_url_waits_as_written() {
    assert_eq!(completed_line("Pay  rent due:2026-10-01"), "Pay  rent due:2026-10-01");
    assert_eq!(completed_line("Open wait: wait://figma"), "Open wait: wait://figma");
}

#[test]
fn a_reopened_task_does_not_get_its_wait_back() {
    let mut task = Todo::from_line("Mail wait:designer");
    task.complete(date!(2026 - 09 - 01));
    task.reopen().unwrap();
    assert_eq!(task.to_line(), "Mail");
}

#[test]
fn reopening_a_task_clears_its_completion_but_not_the_dropped_priority() {
    let mut task = Todo::from_line("(A) 2026-08-01 Lorem ipsum");
    task.complete(date!(2026 - 09 - 01));

    task.reopen().unwrap();

    assert_eq!(task.to_line(), "2026-08-01 Lorem ipsum");
}

#[test]
fn reopening_a_done_line_written_with_a_priority_drops_it() {
    let mut task = Todo::from_line("x (A) Pay rent");

    task.reopen().unwrap();

    assert_eq!(task.to_line(), "Pay rent");
}

#[test]
fn a_task_whose_line_would_still_read_as_done_is_not_reopened() {
    let mut task = Todo::from_line("x 2026-09-03 x Lorem");

    assert!(task.reopen().is_err());

    assert_eq!(task.to_line(), "x 2026-09-03 x Lorem");
}

#[test]
fn a_pending_task_is_due_from_the_day_of_one_of_its_due_dates() {
    let due = |line: &str| Todo::from_line(line).is_due(date!(2026 - 10 - 08));

    assert!(due("Pay rent due:2026-10-08"));
    assert!(due("Pay rent due:2026-10-01"));
    assert!(due("Pay rent due:2026-12-01 due:2026-10-01"));
    assert!(!due("Pay rent due:2026-10-09"));
    assert!(!due("Pay rent due:friday"));
    assert!(!due("Pay rent"));
    assert!(!due("x 2026-10-08 Pay rent due:2026-10-01"));
}

#[test]
fn a_new_task_is_stamped_with_todays_date_unless_it_carries_one() {
    let today = date!(2026 - 09 - 01);

    let stamped = Todo::new_from_input("Lorem ipsum", today).unwrap();
    assert_eq!(stamped.created, Some(today));

    let dated = Todo::new_from_input("2026-08-01 Lorem ipsum", today).unwrap();
    assert_eq!(dated.created, Some(date!(2026 - 08 - 01)));
}

#[test]
fn a_new_task_cannot_be_already_done() {
    assert!(Todo::new_from_input("x Lorem ipsum", date!(2026 - 09 - 01)).is_err());
}

#[test]
fn a_new_task_cannot_have_an_empty_description() {
    assert!(Todo::new_from_input("   ", date!(2026 - 09 - 01)).is_err());
    assert!(Todo::new_from_input("(A) ", date!(2026 - 09 - 01)).is_err());
}

#[test]
fn a_typed_line_holding_a_control_character_is_refused() {
    assert!(Todo::from_input("Lorem\nx 2026-01-01 ipsum").is_err());
    assert!(Todo::from_input("Lorem\ripsum").is_err());
    assert!(Todo::from_input("Lorem \u{1b}[2J").is_err());
}

#[test]
fn a_wait_word_whose_value_starts_like_a_url_is_not_waited_for() {
    assert_eq!(Todo::from_line("Open wait://figma wait:designer").waits(), ["designer"]);
    assert!(!Todo::from_line("Open wait://figma").is_waiting());
}

#[test]
fn a_typed_line_holding_a_bidi_or_zero_width_character_is_refused() {
    for c in ['\u{200b}', '\u{200f}', '\u{202a}', '\u{202e}', '\u{2066}', '\u{2069}'] {
        assert!(Todo::from_input(&format!("Lorem {c}ipsum")).is_err());
    }
    assert!(Todo::from_input("Café 🏠 ipsum").is_ok());
}

#[test]
fn the_line_and_paragraph_separators_are_control_characters() {
    assert!(is_control('\u{2028}'));
    assert!(is_control('\u{2029}'));
}
