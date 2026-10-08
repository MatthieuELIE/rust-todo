# todo

A small editor for a [todo.txt](https://github.com/todotxt/todo.txt) file: a command line to add, list and complete tasks, and a full-screen list meant for a [herdr](https://herdr.dev) popup.

![The interactive list: a filter panel, tasks grouped by priority with a due date in red and one in yellow, and the details of the task under the cursor](docs/images/list.png)

## Install

It needs Rust 1.88 or later, and a terminal showing 24-bit colour: the colours are Catppuccin Mocha's, set as RGB rather than taken from the terminal's theme.

```sh
cargo install --locked --git https://github.com/MatthieuELIE/rust-todo --tag v0.3.0
```

This puts `todo` in `~/.cargo/bin`; drop `--tag` for the latest `main`, or run `cargo install --locked --path .` in a clone.
It is developed on macOS and its tests run on Linux; it has not been tried on Windows.

## Usage

The task file is `~/todo.txt`, or the path in `$TODO_FILE` if it is set.
Tasks are numbered by their position in the file, so a filtered `list` shows gaps.

```sh
todo add "Buy milk"           # append a task, stamped with today's date
todo add "(A) Call the bank"  # with a priority
todo list                     # pending tasks
todo list +finance -@phone    # tasks with +finance and without @phone
todo list --due               # pending tasks due today or overdue
todo list --done              # the tasks of done.txt
todo reopen 2                 # move task 2 of done.txt back, pending again
todo do 2                     # complete task 2: it moves to done.txt
todo edit 2 "(B) Oat milk"    # replace pending task 2, its creation date kept
todo remove 2                 # delete task 2
todo archive                  # move the done tasks to done.txt
```

Only `add`, `do`, `edit`, `reopen`, `remove`, `archive` and the interactive list write the file, and the write is atomic.
`do` stamps the task with today's date and moves it to the end of `done.txt`, in the folder of the task file, so the tasks after it move up one number.
`archive` moves there every done task still in the task file, one typed by hand, in the order of the file, and says how many.
`reopen` takes the number `list --done` shows, adds the task to the end of the task file and removes its line from `done.txt`.
`done.txt` is created when it is missing and `do` and `archive` only add to it, which is not an atomic write; it is written first, so an interruption leaves a task in both files rather than in neither.
`do`, `remove` and `reopen` print the task they handled, such as `done: (A) Call the bank`; `add` and `edit` print nothing.
`ls`, `a` and `rm` are aliases for `list`, `add` and `remove`, as in `todo.sh`, and `done` still works for `do`.

## Interactive list

`todo` with no command opens the list full screen, meant for a terminal popup kept open a few seconds.
Every change is written at once, and `q` quits without asking.
When the file changes on disk while the list is open, the list is reloaded and the key pressed at that moment is ignored.

In herdr, a key opens it in a popup from `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+t"
type = "popup"
command = "todo"
description = "todo list"
width = "80%"
height = "75%"
```

`?` shows every key.

## Documentation

The keys of the list, the popup, how `list` filters and colours, and the line format are described in [the documentation](https://matthieuelie.github.io/rust-todo/).

## License

MIT, see [LICENSE](LICENSE).
