# Messages

On the command line, `todo` writes its messages on stderr; the interactive list shows them in its status bar.
Exit code 0 means it worked or had nothing to do, 1 a refusal or a failure, and 2 a command line it could not read.
A refused command leaves the files as they were; a save that fails partway is covered under Details.

The messages you meet most:

- `nothing to do`: `list` has no pending task to show (exit code 0).
- `no task numbered 9`: `do`, `edit` or `remove` was given a number that is not in the task file, or `reopen` one that is not in `done.txt` (exit code 1).
- `could not save <path>: <reason> (file left unchanged)`: the task file or its folder cannot be written (exit code 1).
- `nothing due`: `list --due` has no pending task due today or before (exit code 0).
- `error: ...`, as clap words it: the command line itself is wrong (exit code 2).

## Details

The tables below are complete: the command line first, then the configuration, then the interactive list.

### Command line

Messages go to stderr; what the commands print on stdout is described in [Commands](commands.md#what-is-printed). A confirmation that cannot be printed, into a closed pipe for instance, does not change the exit code: the files are written.

| Message | When | Exit code |
| --- | --- | --- |
| `nothing to do` | `list` has no pending task to show | 0 |
| `nothing due` | `list --due` has no pending task due today or before | 0 |
| `nothing done` | `list --done` found no `done.txt`, or an empty one | 0 |
| `no matching task` | `list` was given terms and no task has them | 0 |
| `a task needs more than +projects, @contexts and key: words` | `add` or `edit` was given a text with no word that is not a `+project`, an `@context`, a `key:` word (`wait:` and a bare URL included), a priority `(A)` to `(E)` or a date; an empty text is one. An `edit` that leaves the line unchanged is not checked | 1 |
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
| `task 2 cannot be reopened: it is only +projects, @contexts and key: words` | `reopen` was given a line of `done.txt` with no word that is not a `+project`, an `@context`, a `key:` word, a priority or a date, as for `add` | 1 |
| `could not read <path>: <reason>` | the task file, or `done.txt` for `list --done` and `reopen`, exists and cannot be read, is not UTF-8 text, or is larger than 10 MiB | 1 |
| `could not save <path>: <reason> (file left unchanged)` | the task file or its folder cannot be written, or `done.txt` for `archive`, `do` and `reopen` | 1 |
| `could not set up the terminal: <reason>` | `todo` was run with no command and without a terminal, from a script for instance | 1 |
| `--all is gone: list shows the pending tasks, list --done the done ones` | `list` was given `--all` or `-a` as its first argument | 2 |
| `error: ...`, as clap words it | the command line itself is wrong: an unknown command or argument, a missing number or text, a number that is not one, `--done` with `--due` | 2 |

A command that is refused leaves the file as it was.
When `archive` or `do` has added to `done.txt` and the task file then cannot be saved, the done tasks are in both files: remove them from one by hand before running it again.
The same holds for `reopen` when the task file was saved and `done.txt` then cannot be: the task is pending in the task file and still in `done.txt`. A `done.txt` that may not be written is refused before the task file is touched, and so is a task file that may not be written before `archive` or `do` adds to `done.txt`; a folder that may not be written is only found out at the second write.
A task file that does not exist is not an error: it is an empty list, and the first `add` creates it.

When `todo` is run with no command and the file cannot be read, the message is also shown full screen until a key is pressed, so that a popup closing with the program does not hide it.

### Configuration

A `config.toml` that is refused stops every command, and `todo` with no command shows the message full screen until a key is pressed. The message starts with the path of the file; for a fault in the text, it gives the line and column, the faulty text and the reason, as the TOML reader words it. The file is described in [Configuration](config.md).

| Message | When | Exit code |
| --- | --- | --- |
| `<path>: TOML parse error at line 2, column 8 … unknown variant `nord`, expected one of `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `terminal`` | `[theme]` has a `name` that is not a valid theme name; the message lists the valid ones | 1 |
| `<path>: TOML parse error at line 2, column 9 … invalid type: string "yes", expected a boolean` | `icons` in `[theme]` is not `true` or `false` | 1 |
| `<path>: TOML parse error at line 2, column 1 … unknown field `colour`` | the file has a key or a section that `todo` does not know | 1 |
| `<path>: TOML parse error at line 1, column 7 …` | the file is not valid TOML | 1 |
| `could not read <path>: <reason>` | `config.toml` exists and cannot be read, or is not UTF-8 text | 1 |

### Interactive list

Messages are toasts: each takes the place of the filters in the status bar for three seconds, whatever keys are pressed meanwhile.
While the search line is open it stays drawn, and the toast comes back once it closes if its time is not up.
The mode on the left, the keys of a command under way and `? help` on the right stay; a toast too long for the room left is cut with `…`.
A new toast replaces the one shown and lasts its own three seconds, and a refusal does the same.
A green one says that something was done, a grey one only says what happened, a red one says that something was refused or failed.
With `icons = true` ([Configuration](config.md#icons)) a Nerd Font icon comes first: a check mark in green, an `i` in grey, a cross in red.

| Message | Colour | When |
| --- | --- | --- |
| `Priority is a to e, or space` | red | the key after `p` was not a priority |
| `Task copied to the clipboard` | green | `yy` put the line of the task under the cursor in the clipboard; the bar does not show the line |
| `No task under the cursor` | red | `yy` was pressed on a group header or in an empty list; nothing is copied |
| `Could not copy: <reason>` | red | `yy` could not run `pbcopy` (it is missing on Linux, for one) or it failed |
| `No task due` | grey | `D` was pressed with no pending task due today or before; the filter is kept |
| `No due task matches the search` | grey | `D` was pressed while the search hides every pending task due today or before; the filter and the search are kept |
| `History is read-only` | red | `x`, `dd`, `o`, `Enter`, `p`, `u`, `Ctrl-R` or `D` was pressed in the history, the list of `done.txt` shown by `H` |
| `Could not read done.txt: <reason>` | red | `done.txt` exists and cannot be read, is not UTF-8 text, or is larger than 10 MiB, when the list opens or at `H`; the history is then empty |
| `Task 2 is not done`, `Task 2 cannot be reopened: its text starts with x`, `Task 2 cannot be reopened: it is only +projects, @contexts and key: words` | red | `r` was pressed in the history on a line `todo reopen` refuses, for the reason given above for the command line; nothing is written |
| `Task reopened, press u to undo` | green | `r` brought a task of the history back to the task file |
| `Undone, the task was no longer in done.txt`, `Reopened, the task was no longer in done.txt` | green | `u` brought back a completed task, or `r` a task of the history, whose line was not found in `done.txt`, changed elsewhere since it was read; nothing is lost |
| `Undone, but done.txt still holds the task: <reason>`, `Reopened, but done.txt still holds the task: <reason>` | red | `u` or `r` brought a task back and `done.txt` could not be read or written: the task is in both files, and the `u` and `Ctrl-R` history is cleared; when one save brings back several tasks, only the first refusal is shown, and others may be in both files too |
| `A task needs more than +projects, @contexts and key: words` | red | the popup was saved empty, or with only `+projects`, `@contexts`, `key:` words, a priority or a date, for an add or a changed edit; the popup stays open. An `x` typed in front, or an unchanged line, is not checked |
| `Cannot add a task that is already done` | red | a task added in the popup starts with `x `, the `x` and a space; the popup stays open |
| `Could not save: <reason> (file left unchanged)` | red | the task file, or `done.txt` for `x`, for an `x` typed in the popup and for `r` in the history, could not be written, or the task file is `done.txt` itself; the list goes back to what the task file holds |
| `Could not read the file: <reason> (file left unchanged)` | red | the file could not be read back before saving, because it cannot be read or is larger than 10 MiB; the change stays on screen and is saved with the next one, once the file reads again |
| `Could not save: <reason> (done.txt already holds the task)` | red | `x` added the task to `done.txt` and the task file then could not be saved: the task is in both files |
| `File reloaded` | grey | the file changed on disk; the key pressed at that moment was ignored and a command under way was dropped |
| `File reloaded, edit cancelled` | grey | the file changed on disk while a task was being edited |
| `Task added, hidden by the filter, press u to undo` | green | the task added does not match the search |
| `Task updated, hidden by the filter, press u to undo` | green | the task edited no longer matches the search or the panel filter |
| `Priority set to B, hidden by the filter, press u to undo`, `Priority removed, hidden by the filter, press u to undo` | green | the task given a priority, the letter chosen or none for `p Space`, no longer matches the search |
| `Change undone, press Ctrl-r to redo`, `Change redone, press u to undo` | green | `u` or `Ctrl-R` did something in the list |
| `Undone`, `Redone` | green | `u` or `Ctrl-R` did something in the popup |
| `Nothing to undo`, `Nothing to redo` | grey | `u` or `Ctrl-R` had nothing left, in the list or in the popup |
