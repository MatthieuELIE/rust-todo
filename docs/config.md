# Configuration

`todo` reads `config.toml` in `$XDG_CONFIG_HOME/todo/`, or in `~/.config/todo/` when that variable is not set or is empty, on macOS as on Linux.
A file that does not exist is not an error: every setting takes its default.

The smallest useful file picks a theme:

```toml
[theme]
name = "catppuccin-mocha"
```

The default, `terminal`, follows the colours of the terminal.
The screenshots in this book are taken in Catppuccin Mocha.

## Details

### Where the file is

There is no variable to point at another file.

### `[theme]`

| Key | Values | Default |
| --- | --- | --- |
| `name` | `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `terminal` | `terminal` |
| `icons` | `true`, `false` | `false` |

The theme colours the interactive list, and `todo list` on a terminal.
A pipe leaves `todo list` as it was.

### The four `catppuccin-` themes

The four `catppuccin-` names paint the list in the colours of the official [Catppuccin](https://github.com/catppuccin/palette) flavours, Latte being the light one.
They use fixed 24-bit colours, which needs a terminal that shows them.

### The `terminal` theme

`terminal` uses the terminal's own ANSI colours: its default text and background, and its blue, red, yellow, magenta, cyan and green, so the list follows the terminal's theme.
There is no background for the row under the cursor, which the `→` marks, nor for a priority badge.

Under Solarized Dark the terminal's `bright black`, used for dates, borders and the history, is almost the background.
Pick one of the `catppuccin-` themes there.

### Icons

`icons = true` shows ten [Nerd Font](https://www.nerdfonts.com) icons, each before its word and followed by a space, in the word's own style:

- the `PROJECTS` and `CONTEXTS` headers of the panel;
- its `All tasks`, `Due`, `Waiting` and `Sleeping` entries;
- the `DETAILS` title;
- a toast in the status bar: a check mark for a success, an `i` for an information and a cross for a refusal.

It needs a Nerd Font in the terminal; without one the icons show as empty boxes.
The key does not depend on `name` or on `NO_COLOR`: it works with every theme and with no colour.
Nothing else changes, and `todo list` has no icons.

### `NO_COLOR`

When [`NO_COLOR`](https://no-color.org) is set and not empty, a file without a `name`, or no file, gives no colour at all.
The list and `todo list` on a terminal keep the bold, the strike-through and the layout, and nothing else.

The active zone is then read from the mode block alone, which is bold and has no background; the border of a focused card is not told apart.
A `name` in `config.toml` wins over `NO_COLOR`, and the list takes that theme's colours.

### `[open]`

`[open]` names, for a key, the program that `gx` in the [interactive list](interactive-list.md#opening-a-keyvalue) runs on a key:value of the task with that key:

```toml
[open]
ticket = "~/.config/todo/open-ticket"
```

Each key of the table is the key of a key:value, `ticket` for `ticket:audit-03/09`, and its value the path of the program.
A leading `~/` of the path is replaced by your home folder; nothing else is expanded, neither a variable nor `~user`.
With no `[open]`, `gx` has nothing to open.

The program gets:

- the value of the key:value, `audit-03/09` here, as its only argument;
- the task's projects in `TODO_PROJECTS` and its contexts in `TODO_CONTEXTS`, without their `+` and `@`, separated by spaces, added to the environment of `todo`.

It runs without a shell, so a value cannot run a command, and must be executable.
It is detached from the list, which does not wait for it: the list stays open, and the program outlives a popup that closes.
Its standard input and output are cut off, so what it prints is lost and its exit code is not shown.

`todo` knows nothing of what the value points to: the program decides.
This one, made executable with `chmod +x`, opens the pull requests of the task's first project for a task with `@review`, and a note named after the ticket otherwise:

```sh
#!/bin/sh
project=${TODO_PROJECTS%% *}

case " $TODO_CONTEXTS " in
  *" review "*) exec open "https://github.com/me/$project/pulls" ;;
esac

exec open "$HOME/notes/$project/tickets/$1.md"
```

`open` is the macOS command; on Linux, `xdg-open` takes its place.

### A refused file

A file is refused, with the path, the line and the fault, when it holds:

- text that is not valid TOML;
- a `name` that is not in the table above;
- an `icons` that is not `true` or `false`;
- a value under `[open]` that is not a string;
- a key or section this page does not describe.

An unknown `name` also lists the valid names.
Every command stops with exit code 1, `todo add` included, and `todo` with no command shows the message full screen until a key is pressed.
See [Messages](messages.md#configuration).
