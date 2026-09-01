# todo

A small command-line editor for a [todo.txt](https://github.com/todotxt/todo.txt)
file.

## Build

```sh
cargo build --release
```

The binary lands in `target/release/todo`.

## Usage

The task file is `~/todo.txt`, or the path in `$TODO_FILE` if it is set.
Tasks are numbered by their position in the file.

```sh
todo add "Buy milk" --priority A   # append a task, optional priority A-E
todo list                          # pending tasks
todo list --all                    # including the done ones
todo done 2                         # mark task 2 as done
todo remove 2                       # delete task 2
```

Only `add`, `done` and `remove` write the file, and the write is atomic.
