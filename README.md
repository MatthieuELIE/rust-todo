# todo

A small command-line editor for a [todo.txt](https://github.com/todotxt/todo.txt) file.

## Build

```sh
cargo build --release
```

The binary lands in `target/release/todo`.

## Usage

The task file is `~/todo.txt`, or the path in `$TODO_FILE` if it is set.
Tasks are numbered by their position in the file, so a filtered `list` shows gaps.

```sh
todo add "Buy milk"           # append a task, stamped with today's date
todo add "(A) Call the bank"  # priority inline, or as --priority A
todo list                     # pending tasks
todo list --all               # including the done ones
todo list +finance -@phone    # tasks with +finance and without @phone
todo do 2                     # mark task 2 as done
todo remove 2                 # delete task 2
```

Only `add`, `do`, `remove` and the interactive list write the file, and the write is atomic.
`ls`, `a` and `rm` are aliases for `list`, `add` and `remove`, as in `todo.sh`, and `done` still works for `do`.

## Interactive list

`todo` with no command opens the list full screen, meant for a terminal popup kept open a few seconds.
Every change is written at once, and `q` quits without asking.

| Key | Action |
| --- | --- |
| `j` `k`, `↓` `↑` | move |
| `gg`, `G` | top, bottom |
| `Enter` | edit the task in the popup |
| `o` | add a task in the popup, typed as for `todo add` |
| `x` | mark done, or pending again |
| `dd` | delete |
| `p` then `a` to `e` | set the priority; `p` then `Space` clears it |
| `u`, `Ctrl-R` | undo, redo |
| `zM`, `zR` | fold, unfold every priority group |
| `za` | fold or unfold the group under the cursor |
| `/` | search, filtering at each letter |
| `H` | show or hide done tasks |
| `Tab` | move to the filter panel |
| `Esc` | drop the filter and the search |
| `?` | show the keys |
| `q`, `Ctrl-C` | quit |

The rows are those of `todo list`: same numbers, order and colours.
`gg`, `dd`, `p`, `zM`, `zR` and `za` wait for their second key without a timer, and any other key drops them.
A task marked pending again with `x` loses its completion date and does not get back the priority dropped when it was done, but `u` brings it back.
`p` leaves a done task alone, since todo.txt drops the priority of a completed task.
The status bar shows the mode in a coloured block, the active filters, and the keys of the mode when they fit and no message is shown.

### The popup

`o` and `Enter` open a centred popup holding one todo.txt line, which wraps when long.
`o` opens it empty in insert mode; `Enter` opens it on the task's line in normal mode, with the cursor at the start.

In insert mode, characters go in at the cursor, and `←` `→`, `Home` `End`, `Backspace` `Delete`, `Ctrl-W` (the word before the cursor) and `Ctrl-U` (back to the start) edit the line.
Normal mode is a small subset of vim: `h` `l` `0` `$`, `w` `b` `e` and `W` `B` `E`, `x`, `D`, `C`, `dw` `cw` `dW` `cW`, `i` `a` `I` `A`.
`Esc` goes from insert to normal mode and cancels from normal mode, so dropping a task being added takes `Esc Esc`; `Enter` saves from either mode.
A paste goes in at the cursor as one line, its line breaks turned into spaces; outside the popup and the search it is ignored.

An added task is stamped with today's date, and under a panel filter it gets the filter's term appended when it lacks that exact word.
An edited line replaces the task as typed, marker, priority and dates included; only an empty description is refused.
When the file changes on disk during an edit, the edit is cancelled, since the task's number may now name another task.

### Undo

`u` steps back through the changes made since the list was opened, and `Ctrl-R` steps forward again.
Each step restores the whole list, so undoing `x` brings back the priority dropped when the task was done.
The history is not saved: it is lost when the list closes or the file is reloaded.

### Priority groups

When a task on screen has a priority, the list is grouped under `PRIORITY A` to `PRIORITY E`, `NO PRIORITY`, and `DONE` once `H` shows done tasks, each header with how many tasks it holds.
`zM` folds every group into its header, `zR` unfolds them all, and `za` folds or unfolds one; the cursor can stand on a folded header, where `x`, `dd`, `p` and `Enter` do nothing.
Folds are not remembered: the list opens unfolded, and a group that leaves the screen comes back unfolded.

### Filter panel

The panel on the left lists `all`, then every `+project` and `@context` of the tasks shown under `PROJECTS` and `CONTEXTS` headers, with how many tasks each shows.
Moving through it with `j` and `k` filters the list, `Esc` goes back to `all`, and `Tab` or `Enter` returns to the list.
A panel entry shows the tasks with that exact word, case included: `+rust` leaves out `+rust-todo`, and `+Books` and `+books` are two entries.
The search is terms as for `todo list`.

When the file changes on disk while the list is open, the list is reloaded and the key pressed at that moment is ignored.

## Listing

A task is listed when its line contains every term, compared as a case-insensitive literal substring; a leading `-` excludes instead.
`+` and `@` are plain characters, so `-+work` hides the `+work` tasks, and a date is a term like any other: `todo list 2026-09` finds what was created or completed in September.
Flags go before terms: in `todo list +work --all`, `--all` is one more term, excluding `-all`, and done tasks stay hidden.

Pending tasks come first, prioritised ones by priority, then the rest; ties keep the file order.
stdout carries task lines only, coloured when it is a terminal and plain text when piped.
Each element of a line has its own style: a priority bold, tinted yellow, green and blue for A to C as `todo.sh` does, the creation date dimmed, `+projects` magenta and `@contexts` cyan.
A done task is dimmed as a whole.
An empty listing says so on stderr and still exits 0.

### Differences from `todo.sh`

- Terms are literal substrings, not `grep` regular expressions: for an OR, pipe into `grep -i 'a\|b'`.
- Ties in the sort keep the file order instead of going alphabetical.
- `list` hides done tasks unless given `--all`.
- No `TODO: N of M tasks shown` footer.

## Line format

```text
(A) 2026-09-01 Call the bank +finance @phone due:2026-09-15
x 2026-09-03 2026-09-01 Buy milk
```

A line is an optional `x` marker, an optional `(A)`–`(E)` priority, then dates:
the creation date on a pending task, the completion date followed by the creation date on a done one.
Everything after that is the description — `+project`, `@context` and `key:value` are kept verbatim.

`add` records the creation date, `do` records the completion date and drops the priority, as `todo.sh` does.
Anything the parser does not recognise stays in the description rather than being dropped.
