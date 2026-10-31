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

`icons = true` shows nine [Nerd Font](https://www.nerdfonts.com) icons, each before its word and followed by a space, in the word's own style:

- the `PROJECTS` and `CONTEXTS` headers of the panel;
- its `All tasks`, `Due` and `Waiting` entries;
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

### A refused file

A file is refused, with the path, the line and the fault, when it holds:

- text that is not valid TOML;
- a `name` that is not in the table above;
- an `icons` that is not `true` or `false`;
- a key or section this page does not describe.

An unknown `name` also lists the valid names.
Every command stops with exit code 1, `todo add` included, and `todo` with no command shows the message full screen until a key is pressed.
See [Messages](messages.md#configuration).
