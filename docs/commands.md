# Commands

The task file is `~/todo.txt`, or the path in `$TODO_FILE` if it is set.
Completed tasks go to `done.txt`, in the folder of the task file.
`todo` with no command opens the [interactive list](interactive-list.md).

## Examples

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

| Command | Action |
| --- | --- |
| `todo list [--due] [terms]`, `todo list --done [terms]` | list the pending tasks, or those of `done.txt` |
| `todo add "<line>"` | append a task |
| `todo edit <number> "<line>"` | replace a pending task |
| `todo do <number>` | complete a task: it moves to `done.txt` |
| `todo remove <number>` | delete a task |
| `todo reopen <number>` | move a task of `done.txt` back to the task file, pending again |
| `todo archive` | move the done tasks left in the task file to `done.txt` |

`ls`, `a` and `rm` are aliases for `list`, `add` and `remove`, as in `todo.sh`.
`done` is the former name of `do`.
How `list` filters, sorts and colours is described in [Listing](listing.md).

## Details

### Numbering

A task's number is its position in its file, blank lines not counted.
`list` shows it, so a filtered `list` shows gaps.
`do`, `edit` and `remove` take a number of the task file.
`reopen` takes one of `done.txt`, the one `list --done` shows.

### Help and version

`todo --help` and `todo <command> --help` print the help.
`todo --version` or `-V` prints the version.
All of them exit with code 0.

### Adding and editing

`add` takes one [todo.txt line](line-format.md) and appends it to the task file, so `todo add "(A) Call the bank +finance"` sets the priority.
The task is stamped with today's date unless the line carries a creation date.
A text starting with `-` is taken as it is, `todo add "-5 degrees"`, but for `-h` and `--help`, which print the command's help.

`add` refuses:

- a line starting with `x `, the marker of a done task;
- a line break or any other control character;
- a line made only of `+projects`, `@contexts`, `key:` words, a priority or a date, an empty one included.

So `todo add "+work due:2026-10-20"` fails, and `todo add "Call +bank"` does not.

`edit` replaces a pending task with the line given.
The creation date is kept unless the line carries one, and a priority left out is removed.
A line equal to the task writes nothing and is not checked.

`edit` refuses:

- any other line that has no text word, as for `add`;
- a line starting with `x `, which makes it stricter than [the popup](popup.md#saving), where the popup completes the task;
- a done task still in the task file.

`remove` deletes the task from the task file, whether it is done or not.
Nothing goes to `done.txt`.

### Completing

`do` stamps the task with today's date, drops its priority and moves it to the end of `done.txt`.
The tasks after it move up one number.

`archive` moves there every done task still in the task file, one typed by hand for instance, in the order of the file.
It says how many: `2 tasks archived`.

`reopen` adds the task to the end of the task file, pending and without its completion date.
It removes the task's line from `done.txt`, whose other lines are left as written.
The task comes back without a priority, even a `done.txt` line written by hand with one: `edit` sets one.

`reopen` refuses a task made only of `+projects`, `@contexts`, `key:` words, a priority or a date.
The message is `task 2 cannot be reopened: it is only +projects, @contexts and key: words`.

### What is written

Every command but `list` writes the task file, as the interactive list does.
The write is atomic: a temporary file takes the place of the task file once it is complete.

`done.txt` is created when it is missing, readable and writable by its owner only.
`do` and `archive` only add to its end, which is not an atomic write.

A rewrite keeps one task per line:

- blank lines are dropped;
- line endings become `\n`;
- the last line ends with one.

A task file that does not exist yet is created readable and writable by its owner only.
An existing one keeps its permissions.

The file a task goes to is written first, so an interruption leaves the task in both files rather than in neither.
When the task file is a symbolic link, the file it points to is written, and `done.txt` is the one in the folder of the link.

### What is printed

`list` prints its tasks on stdout.
`do`, `remove` and `reopen` print the task they handled once the files are written, as `list` shows it and without its number: `done: (A) Call the bank`, `removed:` or `reopened:`.
`do` shows the task as it was before it was completed, priority included.
`add` and `edit` print nothing.

Messages go to stderr, with these exit codes:

- 0 when the command did its work or had nothing to do;
- 1 when it was refused or failed;
- 2 when the command line itself is wrong.

Every message is listed in [Messages](messages.md).
