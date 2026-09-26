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
| `x` | mark done, or pending again |
| `o` | add a task, typed as for `todo add` |
| `dd` | delete |
| `/` | search, filtering at each letter |
| `H` | show or hide done tasks |
| `Tab` | move to the filter panel |
| `Esc` | drop the filter and the search |
| `?` | show the keys |
| `q`, `Ctrl-C` | quit |

The rows are those of `todo list`: same numbers, order and styling.
`gg` and `dd` wait for their second key without a timer, and any other key drops them.
A task marked pending again loses its completion date and does not get back the priority dropped when it was done.

The panel on the left lists `all`, then every `+project` and `@context` of the tasks shown, with how many tasks each shows.
Moving through it with `j` and `k` filters the list, `Esc` goes back to `all`, and `Tab` or `Enter` returns to the list.
The panel filter and the search are terms as for `todo list`, and a task added under a panel filter gets its term appended when it lacks it.

When the file changes on disk while the list is open, the list is reloaded and the key pressed at that moment is ignored.

## Listing

A task is listed when its line contains every term, compared as a case-insensitive literal substring; a leading `-` excludes instead.
`+` and `@` are plain characters, so `-+work` hides the `+work` tasks, and a date is a term like any other: `todo list 2026-09` finds what was created or completed in September.
Flags go before terms: in `todo list +work --all`, `--all` is one more term, excluding `-all`, and done tasks stay hidden.

Pending tasks come first, prioritised ones by priority, then the rest; ties keep the file order.
stdout carries task lines only, bold for a priority and dimmed once done when it is a terminal, plain text when piped.
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
