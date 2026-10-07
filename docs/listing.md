# Listing

A task is listed when its line contains every term, compared as a case-insensitive literal substring; a leading `-` excludes instead.
`+` and `@` are plain characters, so `-+work` hides the `+work` tasks, and a date is a term like any other: `todo list 2026-09` finds what was created or completed in September.
Flags go before terms: in `todo list +work --all`, `--all` is one more term, excluding `-all`, and done tasks stay hidden.
`--due` keeps the pending tasks holding a `due:` date that is today or past, and combines with the terms after it: `todo list --due +work`. A done task is never due, so `--all` adds nothing to it, and a `due:` that is not a `YYYY-MM-DD` date is ignored.

Pending tasks come first, prioritised ones by priority, then the rest; ties keep the file order.
stdout carries task lines only, coloured when it is a terminal and `NO_COLOR` is unset or empty, otherwise plain todo.txt lines.
Each element of a line has its own style: the priority as a badge, ` A ` in its colour on a dark background (A pink, B green, C blue, D lavender, E sky), the creation date greyed, `+projects` mauve and `@contexts` teal.
`key:value` words such as `wait:figma` are greyed too: a key starting with a letter, then a value, so that URLs and times like `10:30` keep their plain style.
A `due:` date is red once past and yellow on the day; later, or when it is not a `YYYY-MM-DD` date, it is greyed like any `key:value`.
A done task is greyed and struck through as a whole, whatever its due date.
A control character found in a line of the file, such as a terminal escape sequence, is printed as `�`, so that a line cannot drive the terminal.
An empty listing says so on stderr and still exits 0.

## Differences from `todo.sh`

- Terms are literal substrings, not `grep` regular expressions: for an OR, pipe into `grep -i 'a\|b'`.
- Ties in the sort keep the file order instead of going alphabetical.
- `list` hides done tasks unless given `--all`.
- No `TODO: N of M tasks shown` footer.
