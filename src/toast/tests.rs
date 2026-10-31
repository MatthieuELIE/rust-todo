use super::*;
use crate::theme::Theme;
use ratatui::style::Color;

fn says(notice: Notice, text: &str, kind: Kind) {
    assert_eq!((notice.text().as_str(), notice.kind()), (text, kind));
}

#[test]
fn copied_says_the_task_went_to_the_clipboard() {
    says(Notice::Copied, "Task copied to the clipboard", Kind::Success);
}

#[test]
fn reopened_says_how_to_undo() {
    says(Notice::Reopened, "Task reopened, press u to undo", Kind::Success);
}

#[test]
fn a_change_undone_in_the_list_says_how_to_redo() {
    says(Notice::ChangeUndone, "Change undone, press Ctrl-r to redo", Kind::Success);
}

#[test]
fn a_change_redone_in_the_list_says_how_to_undo() {
    says(Notice::ChangeRedone, "Change redone, press u to undo", Kind::Success);
}

#[test]
fn undone_and_redone_in_the_popup_are_one_word() {
    says(Notice::Undone, "Undone", Kind::Success);
    says(Notice::Redone, "Redone", Kind::Success);
}

#[test]
fn nothing_to_undo_and_nothing_to_redo_are_information() {
    says(Notice::NothingToUndo, "Nothing to undo", Kind::Info);
    says(Notice::NothingToRedo, "Nothing to redo", Kind::Info);
}

#[test]
fn reloaded_is_information() {
    says(Notice::Reloaded, "File reloaded", Kind::Info);
}

#[test]
fn reloaded_with_an_edit_cancelled_is_information() {
    says(Notice::ReloadedEditCancelled, "File reloaded, edit cancelled", Kind::Info);
}

#[test]
fn nothing_due_is_information() {
    says(Notice::NothingDue, "No task due", Kind::Info);
}

#[test]
fn nothing_due_matching_the_search_is_information() {
    says(Notice::NothingDueMatchesSearch, "No due task matches the search", Kind::Info);
}

#[test]
fn an_added_task_hidden_by_the_filter_says_so_and_how_to_undo() {
    says(
        Notice::Added { hidden: true },
        "Task added, hidden by the filter, press u to undo",
        Kind::Success,
    );
    says(Notice::Added { hidden: false }, "Task added, press u to undo", Kind::Success);
}

#[test]
fn an_edited_task_hidden_by_the_filter_says_so_and_how_to_undo() {
    says(
        Notice::Updated { hidden: true },
        "Task updated, hidden by the filter, press u to undo",
        Kind::Success,
    );
    says(Notice::Updated { hidden: false }, "Task updated, press u to undo", Kind::Success);
}

#[test]
fn a_priority_hidden_by_the_filter_names_the_letter_or_says_removed() {
    let set = Notice::PrioritySet {
        priority: Some('B'),
        hidden: true,
    };
    says(set, "Priority set to B, hidden by the filter, press u to undo", Kind::Success);
    let removed = Notice::PrioritySet {
        priority: None,
        hidden: true,
    };
    says(removed, "Priority removed, hidden by the filter, press u to undo", Kind::Success);
}

#[test]
fn the_other_messages_keep_their_text_with_a_capital_and_a_refusal_is_a_refusal() {
    says(Notice::NotPriority, "Priority is a to e, or space", Kind::Refusal);
    says(Notice::HistoryReadOnly, "History is read-only", Kind::Refusal);
    says(Notice::NoTaskUnderCursor, "No task under the cursor", Kind::Refusal);
    says(Notice::Refused("could not copy: boom".to_string()), "Could not copy: boom", Kind::Refusal);
    says(
        Notice::GoneFromDone { reopened: true },
        "Reopened, the task was no longer in done.txt",
        Kind::Success,
    );
    says(
        Notice::GoneFromDone { reopened: false },
        "Undone, the task was no longer in done.txt",
        Kind::Success,
    );
}

#[test]
fn a_toast_is_over_at_its_instant_and_not_before() {
    let start = Instant::now();
    let toast = Toast {
        notice: Notice::Copied,
        until: start + DURATION,
    };
    assert!(!toast.expired(start + DURATION - Duration::from_millis(1)));
    assert!(toast.expired(start + DURATION));
}

#[test]
fn every_theme_has_a_success_colour_green_in_catppuccin_and_terminal_and_none_without_colour() {
    let colours = [
        Theme::MOCHA.success,
        Theme::LATTE.success,
        Theme::FRAPPE.success,
        Theme::MACCHIATO.success,
    ];
    assert_eq!(
        colours,
        [
            Color::Rgb(0xa6, 0xe3, 0xa1),
            Color::Rgb(0x40, 0xa0, 0x2b),
            Color::Rgb(0xa6, 0xd1, 0x89),
            Color::Rgb(0xa6, 0xda, 0x95)
        ]
    );
    assert_eq!(Theme::TERMINAL.success, Color::Green);
    assert_eq!(Theme::NONE.success, Color::Reset);
}
