# Configuration

`todo` reads `config.toml` in `$XDG_CONFIG_HOME/todo/`, or in `~/.config/todo/` when that variable is not set or is empty, on macOS as on Linux.
There is no variable to point at another file.
A file that does not exist is not an error: every setting takes its default.

```toml
[theme]
name = "terminal"
```

## `[theme]`

| Key | Values | Default |
| --- | --- | --- |
| `name` | `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `terminal` | `terminal` |
| `icons` | `true`, `false` | `false` |

`terminal` uses the terminal's own ANSI colours: its default text and background, and its blue, red, yellow, magenta, cyan and green, so the list follows the terminal's theme.
There is no background for the row under the cursor, which the `→` marks, nor for a priority badge.
The four `catppuccin-` names paint the list in the colours of the official [Catppuccin](https://github.com/catppuccin/palette) flavours, Latte being the light one, as fixed 24-bit colours, which needs a terminal that shows them.
Under Solarized Dark the terminal's `bright black`, used for dates, borders and the history, is almost the background: pick one of the `catppuccin-` themes there.

The theme colours the interactive list, and `todo list` on a terminal.
A pipe leaves `todo list` as it was.

`icons = true` shows six [Nerd Font](https://www.nerdfonts.com) icons, each before its word and followed by a space, in the word's own style: the `PROJECTS` and `CONTEXTS` headers of the panel, its `All tasks`, `Due` and `Waiting` entries, and the `DETAILS` title.
It needs a Nerd Font in the terminal; without one the icons show as empty boxes.
The key does not depend on `name` or on `NO_COLOR`: it works with every theme and with no colour.
Nothing else changes, and `todo list` has no icons.

## `NO_COLOR`

When [`NO_COLOR`](https://no-color.org) is set and not empty, a file without a `name`, or no file, gives no colour at all: the list and `todo list` on a terminal keep the bold, the strike-through and the layout, and nothing else.
The active zone is then read from the mode block alone, which is bold and has no background; the border of a focused card is not told apart.
A `name` in `config.toml` wins over `NO_COLOR`, and the list takes that theme's colours.

## A refused file

A file that is not valid TOML, a `name` that is not in the table above, an `icons` that is not `true` or `false`, or a key or section this page does not describe, is refused, with the path, the line and the fault; an unknown `name` also lists the valid names.
Every command stops with exit code 1, `todo add` included, and `todo` with no command shows the message full screen until a key is pressed.
See [Messages](messages.md#configuration).
