# Messages

What `todo` says when something is refused or fails, on the command line and in the interactive list.

## Command line

Messages go to stderr; what the commands print on stdout is described in [Commands](commands.md#what-is-printed). A confirmation that cannot be printed, into a closed pipe for instance, does not change the exit code: the files are written.

| Message | When | Exit code |
| --- | --- | --- |
| `nothing to do` | `list` has no pending task to show | 0 |
| `nothing due` | `list --due` has no pending task due today or before | 0 |
| `nothing done` | `list --done` found no `done.txt`, or an empty one | 0 |
| `no matching task` | `list` was given terms and no task has them | 0 |
| `a task needs a description` | `add` or `edit` was given an empty text, or a priority or a date followed by a space and nothing else; without that space, `(A)` is the description | 1 |
| `cannot add a task that is already done` | `add` was given a line starting with `x `, the `x` and a space | 1 |
| `a task is one line, without control characters` | `add` or `edit` was given a text holding a line break, a tab or an escape character | 1 |
| `no task numbered 9` | `do`, `edit` or `remove` was given a number that is not in the task file, or `reopen` one that is not in `done.txt` | 1 |
| `task 2 is already done` | `do` was given the number of a done task still in the task file: `archive` moves it | 1 |
| `task 2 is done` | `edit` was given the number of a done task | 1 |
| `cannot edit a task into a done one` | `edit` was given a line starting with `x `, the `x` and a space | 1 |
| `nothing to archive` | `archive` found no done task in the file; nothing is written | 0 |
| `the task file is done.txt itself: <path>` | `archive`, `do` or `reopen` was run on a task file that is `done.txt` itself, or a link to it | 1 |
| `task 2 is not done` | `reopen` was given the number of a line of `done.txt` that is not a done task | 1 |
| `task 2 cannot be reopened: its text starts with x` | `reopen` was given a done task with no creation date whose text starts with `x `: its line would still read as done | 1 |
| `task 2 cannot be reopened: it has no description` | `reopen` was given a line of `done.txt` with nothing after the space that follows its `x` or its dates: it would become an empty line | 1 |
| `could not read <path>: <reason>` | the task file, or `done.txt` for `list --done` and `reopen`, exists and cannot be read, or is not UTF-8 text | 1 |
| `could not save <path>: <reason> (file left unchanged)` | the task file or its folder cannot be written, or `done.txt` for `archive`, `do` and `reopen` | 1 |
| `could not set up the terminal: <reason>` | `todo` was run with no command and without a terminal, from a script for instance | 1 |
| `--all is gone: list shows the pending tasks, list --done the done ones` | `list` was given `--all` or `-a` as its first argument | 2 |
| `error: ...`, as clap words it | the command line itself is wrong: an unknown command or argument, a missing number or text, a number that is not one, `--done` with `--due` | 2 |

A command that is refused leaves the file as it was.
When `archive` or `do` has added to `done.txt` and the task file then cannot be saved, the done tasks are in both files: remove them from one by hand before running it again.
The same holds for `reopen` when the task file was saved and `done.txt` then cannot be: the task is pending in the task file and still in `done.txt`. A `done.txt` that may not be written is refused before the task file is touched, and so is a task file that may not be written before `archive` or `do` adds to `done.txt`; a folder that may not be written is only found out at the second write.
A task file that does not exist is not an error: it is an empty list, and the first `add` creates it.

When `todo` is run with no command and the file cannot be read, the message is also shown full screen until a key is pressed, so that a popup closing with the program does not hide it.

## Interactive list

Messages show on the right of the status bar until the next key.
A red one says that something was refused or failed; a grey one only says what happened.

| Message | Colour | When |
| --- | --- | --- |
| `priority is a to e, or space` | red | the key after `p` was not a priority |
| `nothing due` | grey | `D` was pressed with no pending task due today or before; the filter is kept |
| `history is read-only` | red | `x`, `dd`, `o`, `Enter`, `p`, `u`, `Ctrl-R` or `D` was pressed in the history, the list of `done.txt` shown by `H` |
| `could not read done.txt: <reason>` | red | `done.txt` exists and cannot be read, or is not UTF-8 text, when the list opens or at `H`; the history is then empty |
| `task 2 is not done`, `task 2 cannot be reopened: its text starts with x`, `task 2 cannot be reopened: it has no description` | red | `r` was pressed in the history on a line `todo reopen` refuses, for the reason given above for the command line; nothing is written |
| `reopened` | grey | `r` brought a task of the history back to the task file |
| `undone, the task was no longer in done.txt`, `reopened, the task was no longer in done.txt` | grey | `u` brought back a completed task, or `r` a task of the history, whose line was not found in `done.txt`, changed elsewhere since it was read; nothing is lost |
| `undone, but done.txt still holds the task: <reason>`, `reopened, but done.txt still holds the task: <reason>` | red | `u` or `r` brought a task back and `done.txt` could not be read or written: the task is in both files |
| `a task needs a description` | red | the popup was saved empty, or with a priority or dates followed by a space and nothing else |
| `cannot add a task that is already done` | red | a task added in the popup starts with `x `, the `x` and a space |
| `could not save: <reason> (file left unchanged)` | red | the task file, or `done.txt` for `x`, for an `x` typed in the popup and for `r` in the history, could not be written, or the task file is `done.txt` itself; the list goes back to what the task file holds |
| `could not read the file: <reason> (file left unchanged)` | red | the file could not be read back before saving; the change stays on screen and is saved with the next one, once the file reads again |
| `could not save: <reason> (done.txt already holds the task)` | red | `x` added the task to `done.txt` and the task file then could not be saved: the task is in both files |
| `reloaded` | grey | the file changed on disk; the key pressed at that moment was ignored |
| `reloaded, edit cancelled` | grey | the file changed on disk while a task was being edited |
| `added, hidden by the filter` | grey | the task added does not match the search |
| `edited, hidden by the filter` | grey | the task edited no longer matches the search or the panel filter |
| `priority set, hidden by the filter` | grey | the task given a priority no longer matches the search |
| `undone`, `redone` | grey | `u` or `Ctrl-R` did something, in the list or in the popup |
| `nothing to undo`, `nothing to redo` | grey | `u` or `Ctrl-R` had nothing left |
