# Commands

The task file is `~/todo.txt`, or the path in `$TODO_FILE` if it is set.
Completed tasks go to `done.txt`, in the folder of the task file.
`todo` with no command opens the [interactive list](interactive-list.md).

| Command | Action |
| --- | --- |
| `todo list [--due] [terms]`, `todo list --done [terms]` | list the pending tasks, or those of `done.txt` |
| `todo add "<line>"` | append a task |
| `todo edit <number> "<line>"` | replace a pending task |
| `todo do <number>` | complete a task: it moves to `done.txt` |
| `todo remove <number>` | delete a task |
| `todo reopen <number>` | move a task of `done.txt` back to the task file, pending again |
| `todo archive` | move the done tasks left in the task file to `done.txt` |

A task's number is its position in its file, blank lines not counted: `list` shows it, so a filtered `list` shows gaps.
`do`, `edit` and `remove` take a number of the task file, `reopen` one of `done.txt`, the one `list --done` shows.
`ls`, `a` and `rm` are aliases for `list`, `add` and `remove`, as in `todo.sh`, and `done` is the former name of `do`.
How `list` filters, sorts and colours is described in [Listing](listing.md).

## Adding and editing

`add` takes one [todo.txt line](line-format.md) and appends it to the task file, so `todo add "(A) Call the bank +finance"` sets the priority.
The task is stamped with today's date unless the line carries a creation date.
A line starting with `x `, the marker of a done task, is refused, and so is a line break or any other control character.
A text starting with `-` is taken as it is, `todo add "-5 degrees"`, but for `-h` and `--help`, which print the command's help.

`edit` replaces a pending task with the line given: the creation date is kept unless the line carries one, and a priority left out is removed.
A line equal to the task writes nothing.
It is stricter than [the popup](popup.md#saving): a line starting with `x ` is refused, where the popup completes the task.
A done task still in the task file is refused too.

`remove` deletes the task from the task file, whether it is done or not; nothing goes to `done.txt`.

## Completing

`do` stamps the task with today's date, drops its priority and moves it to the end of `done.txt`, so the tasks after it move up one number.
`archive` moves there every done task still in the task file, one typed by hand for instance, in the order of the file, and says how many: `2 tasks archived`.
`reopen` adds the task to the end of the task file, pending and without its completion date, and removes its line from `done.txt`, whose other lines are left as written.
The priority dropped when the task was completed does not come back: `edit` sets one.

## What is written

Every command but `list` writes the task file, as the interactive list does, and the write is atomic: a temporary file takes the place of the task file once it is complete.
`done.txt` is created when it is missing, readable and writable by its owner only, and `do` and `archive` only add to its end, which is not an atomic write.
The file a task goes to is written first, so an interruption leaves the task in both files rather than in neither.
When the task file is a symbolic link, the file it points to is written, and `done.txt` is the one in the folder of the link.

## What is printed

`list` prints its tasks on stdout.
`do`, `remove` and `reopen` print the task they handled once the files are written, as `list` shows it and without its number: `done: (A) Call the bank`, `removed:` or `reopened:`.
`do` shows the task as it was before it was completed, priority included.
`add` and `edit` print nothing.
Messages go to stderr: the exit code is 0 when the command did its work or had nothing to do, 1 when it was refused or failed, and 2 when the command line itself is wrong.
Every message is listed in [Messages](messages.md).
