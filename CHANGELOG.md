# Changelog

Notable changes to `todo`, most recent first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- The popup's cursor no longer vanishes when the line fills the field or the cursor is on a space where the line wraps; the completions and the `due:` calendar, which hang from it, used not to be drawn there, the calendar still taking the keys.
- A word such as `wait://figma`, which is not a `key:value`, no longer puts its task under `Waiting` nor shows among the `wait:` completions.
- A task file you may not write is no longer saved over: the save is refused with `permission denied`.
- A task file that is a symbolic link to a file not created yet gets that file created; the link used to be replaced by a regular file.
- A task file created by `todo` is readable and writable by its owner only, and the temporary file of a save is never readable by others first; an existing file keeps its mode.
- A save writes its temporary file under a name of its own, `<file>.<pid>.tmp`, and never over an existing one: a symbolic link left at `<file>.tmp` used to be written through and to replace the task file, and two saves at once shared the same temporary file.
- `do` on a task that is already done is refused with `task 2 is already done`; it used to give today's date to a done task that had no completion date.
- `todo` run with no command and without a terminal says `could not set up the terminal` and exits 1; it used to panic.
- A letter pressed with `Ctrl` or `Alt` no longer acts as the letter in the list, the filter panel, the popup's normal mode and its calendar: `Ctrl-D Ctrl-D` used to delete the task and `Ctrl-U` to undo.
- `add` refuses a text holding a line break or any other control character; a line break used to write several tasks at once, a done one among them.
- `list` prints a control character found in the file as `�` instead of sending it to the terminal.
- A paste in the popup or the search turns every control character into a space, not line breaks only, and so does opening a task of the file in the popup.
- A dated task whose `x` marker is typed in the popup is completed on the day and keeps its creation date; the creation date used to be taken for the completion date.

## [0.2.0] - 2026-10-03

### Added

- In the popup, `d` and `c` take any motion (`db`, `ce`, `d$`), `dd` empties the line and `cc` empties it to type it again.
- In the popup, `d` and `c` take the word under the cursor: `iw`, `aw`, `iW`, `aW`.
- In the popup, `u` and `Ctrl-R` undo and redo the changes made to the line.
- The keys of a command waiting for its end show on the right of the status bar, in the list and the popup.
- A `due:` date is red once past and yellow on the day, and typing `due:` in the popup opens a calendar to pick it.
- Tasks holding a `wait:` word are gathered under a `Waiting` entry of the filter panel, and the popup completes `wait:` values as it does projects and contexts.
- A details card under the list shows the task under the cursor.
- The popup's line is coloured as it is typed, and the terminal's cursor is a bar in insert mode and a block in normal mode.
- A documentation in `docs/`, published at <https://matthieuelie.github.io/rust-todo/>.

### Changed

- The whole screen is restyled in Catppuccin Mocha colours set as RGB, each zone in a rounded card: a terminal showing 24-bit colour is now needed.
- `key:value` words are greyed, URLs and times left plain.
- In the popup, a key that completes no command after `d` or `c` now only cancels it; it used to cancel it and then act. `Esc` and `Enter` follow the rule.
- A pending task whose `x` marker is typed in the popup is completed on the day.
- A task file that cannot be read back is no longer saved over by the interactive list.

### Removed

- The `--priority` flag of `add`: write the priority in the text, as in `todo add "(A) Call the bank"`.

### Fixed

- A paste whose lines end with a bare carriage return no longer writes that character into the task.
- A lone `-` as a search term no longer hides every task.
- The temporary file written on save takes the task file's mode before its content, and is removed when the save fails.
- A trailing space is dropped from a task's description on add and edit.
- An unset `HOME` no longer makes the program panic.

## [0.1.0] - 2026-09-26

First tagged version: the command line (`add`, `list`, `do`, `remove`) and the interactive list with its filter panel, search, priority groups, undo and popup.

[0.2.0]: https://github.com/MatthieuELIE/rust-todo/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/MatthieuELIE/rust-todo/releases/tag/v0.1.0
