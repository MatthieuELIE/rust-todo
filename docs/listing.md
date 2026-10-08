# Listing

A task is listed when its line contains every term, compared as a case-insensitive literal substring; a leading `-` excludes instead.
`+` and `@` are plain characters, so `-+work` hides the `+work` tasks, and a date is a term like any other: `todo list 2026-09` finds what was created or completed in September.
Flags go before terms: in `todo list +work --due`, `--due` is one more term, excluding `-due`, and the tasks that are not due stay listed.
`--due` keeps the pending tasks holding a `due:` date that is today or past, and combines with the terms after it: `todo list --due +work`. A done task is never due, and a `due:` that is not a `YYYY-MM-DD` date is ignored.
`--done` lists `done.txt`, the file `todo archive` fills, instead of the task file: its lines in the order of the file, numbered by their position in it, and filtered by the terms after it. Blank lines are not counted. A line of `done.txt` that is not a done task is listed as it is; a done one is listed as `todo` would write it, without the priority a line such as `x (A) Pay rent` holds. It does not combine with `--due`.
`list` shows pending tasks only: a done task still in the task file, typed by hand or completed in the interactive list, is not listed until `todo archive` moves it. The former `--all`, or `-a`, is refused when it comes first, where the flag used to go; further on it is a term like any other.

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
- `list` never shows done tasks: `list --done` shows `done.txt`.
- No `TODO: N of M tasks shown` footer.
