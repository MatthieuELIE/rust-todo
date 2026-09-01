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
todo done 2                   # mark task 2 as done
todo remove 2                 # delete task 2
```

Only `add`, `done` and `remove` write the file, and the write is atomic.

## Line format

```text
(A) 2026-09-01 Call the bank +finance @phone due:2026-09-15
x 2026-09-03 2026-09-01 Buy milk
```

A line is an optional `x ` marker, an optional `(A)`–`(E)` priority, then dates:
the creation date on a pending task, the completion date followed by the creation date on a done one.
Everything after that is the description — `+project`, `@context` and `key:value` are kept verbatim.

`add` records the creation date, `done` records the completion date and drops the priority, as `todo.sh` does.
Anything the parser does not recognise stays in the description rather than being dropped.
