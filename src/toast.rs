use std::time::{Duration, Instant};

/// How long a toast stays in the status bar.
pub const DURATION: Duration = Duration::from_secs(3);

/// How a toast is told apart: by its colour and its icon.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    /// Something was done.
    Success,
    /// Something was said, nothing changed.
    Info,
    /// Something was refused or failed.
    Refusal,
}

/// Every message of the interactive list.
#[derive(Clone, PartialEq, Debug)]
pub enum Notice {
    /// `yy` put the line in the clipboard.
    Copied,
    /// `r` brought a task of the history back.
    Reopened,
    /// `u` undid a change of the list.
    ChangeUndone,
    /// `Ctrl-R` redid a change of the list.
    ChangeRedone,
    /// `u` undid a change of the popup's text.
    Undone,
    /// `Ctrl-R` redid a change of the popup's text.
    Redone,
    /// `u` had nothing left.
    NothingToUndo,
    /// `Ctrl-R` had nothing left.
    NothingToRedo,
    /// The file changed on disk.
    Reloaded,
    /// The file changed on disk while a task was being edited.
    ReloadedEditCancelled,
    /// `D` found no task due.
    NothingDue,
    /// `D` found no task due among those the search keeps.
    NothingDueMatchesSearch,
    /// A task was added, `hidden` when the filter does not show it.
    Added { hidden: bool },
    /// A task was edited, `hidden` when the filter does not show it.
    Updated { hidden: bool },
    /// A task was marked done.
    Done,
    /// A task was deleted.
    Deleted,
    /// The popup was closed with `Esc` on a text that had changed.
    Discarded,
    /// `Enter` in the popup found the line as it was.
    NoChanges,
    /// A task got `priority`, or none, `hidden` when the filter does not show it.
    PrioritySet { priority: Option<char>, hidden: bool },
    /// `u` or `r` brought a task back and its line was no longer in `done.txt`; `reopened` tells which.
    GoneFromDone { reopened: bool },
    /// The key after `p` was not a priority.
    NotPriority,
    /// A change was asked in the history.
    HistoryReadOnly,
    /// `yy` found no task under the cursor.
    NoTaskUnderCursor,
    /// Something failed or was refused, in these words.
    Refused(String),
}

impl Notice {
    /// The text shown.
    pub fn text(&self) -> String {
        match self {
            Notice::Copied => "Task copied to the clipboard".to_string(),
            Notice::Reopened => "Task reopened, press u to undo".to_string(),
            Notice::ChangeUndone => "Change undone, press Ctrl-r to redo".to_string(),
            Notice::ChangeRedone => "Change redone, press u to undo".to_string(),
            Notice::Undone => "Undone".to_string(),
            Notice::Redone => "Redone".to_string(),
            Notice::NothingToUndo => "Nothing to undo".to_string(),
            Notice::NothingToRedo => "Nothing to redo".to_string(),
            Notice::Reloaded => "File reloaded".to_string(),
            Notice::ReloadedEditCancelled => "File reloaded, edit cancelled".to_string(),
            Notice::NothingDue => "No task due".to_string(),
            Notice::NothingDueMatchesSearch => "No due task matches the search".to_string(),
            Notice::Added { hidden } => undoable("Task added".to_string(), *hidden),
            Notice::Updated { hidden } => undoable("Task updated".to_string(), *hidden),
            Notice::Done => "Task done, press u to undo".to_string(),
            Notice::Deleted => "Task deleted, press u to undo".to_string(),
            Notice::Discarded => "Changes discarded".to_string(),
            Notice::NoChanges => "No changes to save".to_string(),
            Notice::PrioritySet {
                priority: Some(letter),
                hidden,
            } => undoable(format!("Priority set to {letter}"), *hidden),
            Notice::PrioritySet { priority: None, hidden } => undoable("Priority removed".to_string(), *hidden),
            Notice::GoneFromDone { reopened: true } => "Reopened, the task was no longer in done.txt".to_string(),
            Notice::GoneFromDone { reopened: false } => "Undone, the task was no longer in done.txt".to_string(),
            Notice::NotPriority => "Priority is a to e, or space".to_string(),
            Notice::HistoryReadOnly => "History is read-only".to_string(),
            Notice::NoTaskUnderCursor => "No task under the cursor".to_string(),
            Notice::Refused(text) => capitalised(text),
        }
    }

    /// How the text is coloured and marked.
    pub fn kind(&self) -> Kind {
        match self {
            Notice::NothingToUndo
            | Notice::NothingToRedo
            | Notice::Reloaded
            | Notice::ReloadedEditCancelled
            | Notice::NothingDue
            | Notice::NothingDueMatchesSearch
            | Notice::Discarded
            | Notice::NoChanges => Kind::Info,
            Notice::NotPriority | Notice::HistoryReadOnly | Notice::NoTaskUnderCursor | Notice::Refused(_) => Kind::Refusal,
            _ => Kind::Success,
        }
    }
}

/// `done` followed by what hides the task, when it is, and by how to take the change back.
fn undoable(done: String, hidden: bool) -> String {
    if hidden {
        return format!("{done}, hidden by the filter, press u to undo");
    }
    format!("{done}, press u to undo")
}

/// `text` with its first letter in capital.
fn capitalised(text: &str) -> String {
    let mut letters = text.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => String::new(),
    }
}

/// A notice and the instant it goes.
#[derive(Clone, PartialEq, Debug)]
pub struct Toast {
    /// What is said.
    pub notice: Notice,
    /// When it is taken down.
    pub until: Instant,
}

impl Toast {
    /// Whether `now` is at or past the end of the toast.
    pub fn expired(&self, now: Instant) -> bool {
        self.until <= now
    }
}

#[cfg(test)]
mod tests;
