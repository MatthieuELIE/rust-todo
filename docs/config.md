# Configuration

`todo` reads `config.toml` in `$XDG_CONFIG_HOME/todo/`, or in `~/.config/todo/` when that variable is not set, on macOS as on Linux.
There is no variable to point at another file.
A file that does not exist is not an error: every setting takes its default.

```toml
[theme]
name = "catppuccin-mocha"
```

## `[theme]`

| Key | Values | Default |
| --- | --- | --- |
| `name` | `catppuccin-mocha` | `catppuccin-mocha` |

The theme colours the interactive list, and `todo list` on a terminal that takes colours.
`NO_COLOR` and a pipe leave `todo list` as they were.

## A refused file

A file that is not valid TOML, a `name` that is not in the table above, or a key or section this page does not describe, is refused, with the path, the line and the fault; an unknown `name` also lists the valid names.
Every command stops with exit code 1, `todo add` included, and `todo` with no command shows the message full screen until a key is pressed.
See [Messages](messages.md#configuration).
