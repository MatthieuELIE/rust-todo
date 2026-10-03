# Line format

```text
(A) 2026-09-01 Call the bank +finance @phone due:2026-09-15
x 2026-09-03 2026-09-01 Buy milk
```

A line is an optional `x` marker, an optional `(A)`–`(E)` priority, then dates:
the creation date on a pending task, the completion date followed by the creation date on a done one.
Everything after that is the description — `+project`, `@context` and `key:value` are kept verbatim.

`add` records the creation date, `do` records the completion date and drops the priority, as `todo.sh` does.
Anything the parser does not recognise stays in the description rather than being dropped.

## Recognised keys

Two `key:value` words mean something to `todo`; any other is kept as written and only greyed.

- `due:2026-10-15` is the task's due date: red once past and yellow on the day wherever the task is [listed](listing.md), and picked from a calendar in [the popup](popup.md#date-picker).
- `wait:designer` is what the task waits for: such tasks are gathered under `Waiting` in the [filter panel](interactive-list.md#filter-panel), and the popup completes the value.
