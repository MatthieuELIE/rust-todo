# Line format

```text
x 2026-10-27 2026-10-20 Call the bank +finance @phone due:2026-11-01
```

Each part of the line means one thing:

- `x` marks the task as done. It is optional.
- A priority, `(A)` to `(E)`, is optional and absent here: it sits before the dates of a pending task, and `do` drops it.
- `2026-10-27` is the completion date, and `2026-10-20` the creation date. A pending task has only the creation date.
- `Call the bank` is the description.
- `+finance` is a project and `@phone` a context.
- `due:2026-11-01` is a `key:value` word. `due` and `wait` mean something to `todo`.

Everything after the dates is the description: `+project`, `@context` and `key:value` are kept verbatim.
Anything the parser does not recognise stays in the description rather than being dropped.

## Details

### Dates and the done marker

A line is an optional `x` marker, an optional `(A)`–`(E)` priority, then dates:
the creation date on a pending task, the completion date followed by the creation date on a done one.

`add` records the creation date.
`do` records the completion date and drops the priority, as `todo.sh` does, and moves the line to `done.txt`, which holds the same format.

### A description that starts with a marker

A description that starts with a marker is only safe behind a creation date.
In `(A) (B) Call the bank`, written by hand without a date, `(B)` is text for as long as `(A)` stands before it.
Once the task is done or its priority dropped, `(B)` is read as the priority, and a done task loses it.

The same goes for a description starting with `x `, read as the done marker once the priority is gone.
`add` and the popup's `o` write the creation date, which keeps such a description as typed.

### Recognised keys

Two `key:value` words mean something to `todo`; any other is kept as written and only greyed.

- `due:2026-10-15` is the task's due date:
  - red once past and yellow on the day wherever the task is [listed](listing.md);
  - gathered once due by `todo list --due` and under `Due` in the [filter panel](interactive-list.md#filter-panel);
  - picked from a calendar in [the popup](popup.md#date-picker).
- `wait:designer` is what the task waits for:
  - such tasks are gathered under `Waiting` in the [filter panel](interactive-list.md#filter-panel), and the popup completes the value;
  - the key is dropped when the task is completed: `Mail wait:designer +work` becomes `Mail +work` in `done.txt`, and reopening does not bring it back;
  - an empty `wait:` and a `wait://` link stay.
