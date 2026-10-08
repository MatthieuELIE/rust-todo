# Messages

What `todo` says when something is refused or fails, on the command line and in the interactive list.

## Command line

Messages go to stderr; stdout carries task lines, and the count `archive` gives, such as `2 tasks archived`.

| Message | When | Exit code |
| --- | --- | --- |
| `nothing to do` | `list` has no pending task to show | 0 |
| `nothing due` | `list --due` has no pending task due today or before | 0 |
| `no matching task` | `list` was given terms and no task has them | 0 |
| `a task needs a description` | `add` or `edit` was given an empty text, or only a priority or a date | 1 |
| `cannot add a task that is already done` | `add` was given a line starting with `x` | 1 |
| `a task is one line, without control characters` | `add` or `edit` was given a text holding a line break, a tab or an escape character | 1 |
| `no task numbered 9` | `do`, `edit`, `reopen` or `remove` was given a number that is not in the file | 1 |
| `task 2 is already done` | `do` was given the number of a done task | 1 |
| `task 2 is done` | `edit` was given the number of a done task | 1 |
| `cannot edit a task into a done one` | `edit` was given a line starting with `x` | 1 |
| `nothing to archive` | `archive` found no done task in the file; nothing is written | 0 |
| `cannot archive <path> into itself` | `archive` was run on a task file that is `done.txt` itself, or a link to it | 1 |
| `task 2 is not done` | `reopen` was given the number of a pending task | 1 |
| `task 2 cannot be reopened: its text starts with x` | `reopen` was given a done task with no creation date whose text starts with `x `: its line would still read as done | 1 |
| `could not read <path>: <reason>` | the task file exists and cannot be read, or is not UTF-8 text | 1 |
| `could not save <path>: <reason> (file left unchanged)` | the task file or its folder cannot be written, or `done.txt` for `archive` | 1 |
| `could not set up the terminal: <reason>` | `todo` was run with no command and without a terminal, from a script for instance | 1 |
| `error: unrecognized subcommand`, `error: unexpected argument` | the command line itself is wrong | 2 |

A command that is refused leaves the file as it was.
When `archive` has added to `done.txt` and the task file then cannot be saved, the done tasks are in both files: remove them from one by hand before running it again.
A task file that does not exist is not an error: it is an empty list, and the first `add` creates it.

When `todo` is run with no command and the file cannot be read, the message is also shown full screen until a key is pressed, so that a popup closing with the program does not hide it.

## Interactive list

Messages show on the right of the status bar until the next key.
A red one says that something was refused or failed; a grey one only says what happened.

| Message | Colour | When |
| --- | --- | --- |
| `priority is a to e, or space` | red | the key after `p` was not a priority |
| `cannot reopen a task whose text starts with x` | red | `x` was pressed on a done task with no creation date whose text starts with `x `: its line would still read as done |
| `a task needs a description` | red | the popup was saved with nothing but a priority or dates |
| `cannot add a task that is already done` | red | a task added in the popup starts with `x` |
| `could not save: <reason> (file left unchanged)` | red | the file could not be written; the list goes back to what the file holds |
| `could not read the file: <reason> (file left unchanged)` | red | the file could not be read back before saving; the change stays on screen and is saved with the next one, once the file reads again |
| `reloaded` | grey | the file changed on disk; the key pressed at that moment was ignored |
| `reloaded, edit cancelled` | grey | the file changed on disk while a task was being edited |
| `added, hidden by the filter` | grey | the task added does not match the search |
| `edited, hidden by the filter` | grey | the task edited no longer matches the search or the panel filter |
| `priority set, hidden by the filter` | grey | the task given a priority no longer matches the search |
| `undone`, `redone` | grey | `u` or `Ctrl-R` did something, in the list or in the popup |
| `nothing to undo`, `nothing to redo` | grey | `u` or `Ctrl-R` had nothing left |
