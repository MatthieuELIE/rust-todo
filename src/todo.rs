use time::Date;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

/// A date format for parsing dates in the todo.txt format (YYYY-MM-DD).
const DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year]-[month]-[day]");

/// Key of the `key:value` naming what a task waits for, as in `wait:figma`.
pub const WAIT: &str = "wait:";

/// Key of the `key:value` holding a task's due date, as in `due:2026-10-15`.
pub const DUE: &str = "due:";

/// A todo item, with optional priority and dates.
#[derive(Clone)]
pub struct Todo {
    /// Task text after the marker, priority and dates, with `+project`, `@context` and `key:value` kept verbatim.
    pub description: String,
    /// Whether the line starts with the `x` marker.
    pub done: bool,
    /// Priority letter, `A` to `E`, dropped once the task is done.
    pub priority: Option<char>,
    /// Creation date.
    pub created: Option<Date>,
    /// Completion date, only on a done task.
    pub completed: Option<Date>,
}

impl Todo {
    /// Convert a todo item to a line of text in the todo.txt format.
    pub fn to_line(&self) -> String {
        let mut parts = Vec::new();
        if self.done {
            parts.push("x".to_string());
            parts.extend(self.completed.map(|date| date.to_string()));
        } else if let Some(priority) = self.priority {
            parts.push(format!("({priority})"));
        }
        parts.extend(self.created.map(|date| date.to_string()));
        parts.push(self.description.clone());
        parts.join(" ")
    }

    /// Parse a todo.txt line, leaving anything unrecognised in the description.
    pub fn from_line(line: &str) -> Self {
        let done = line.starts_with("x ");
        let mut rest = if done { &line[2..] } else { line };

        let completed = if done { strip_date(&mut rest) } else { None };
        let priority = strip_priority(&mut rest);
        let created = strip_date(&mut rest);

        Todo {
            description: rest.to_string(),
            done,
            priority,
            created,
            completed,
        }
    }

    /// Parse a line typed by the user, its trailing spaces dropped, refusing a control character or an empty description.
    pub fn from_input(text: &str) -> Result<Todo, String> {
        if text.contains(char::is_control) {
            return Err("a task is one line, without control characters".to_string());
        }
        let mut todo = Todo::from_line(text);
        todo.description.truncate(todo.description.trim_end().len());
        if todo.description.is_empty() {
            return Err("a task needs a description".to_string());
        }
        Ok(todo)
    }

    /// Build a pending task from user input, stamped with `today` unless it carries a creation date.
    pub fn new_from_input(text: &str, today: Date) -> Result<Todo, String> {
        let mut todo = Todo::from_input(text)?;
        if todo.done {
            return Err("cannot add a task that is already done".to_string());
        }
        todo.created.get_or_insert(today);
        Ok(todo)
    }

    /// Mark the task done on `today`, keeping an existing completion date and dropping the priority.
    pub fn complete(&mut self, today: Date) {
        self.done = true;
        self.completed.get_or_insert(today);
        self.priority = None;
    }

    /// Mark the task pending again, clearing its completion date; the priority dropped on completion does not come back.
    pub fn reopen(&mut self) {
        self.done = false;
        self.completed = None;
    }

    /// `+project` names in the description, without the `+`, once each in order of appearance.
    pub fn projects(&self) -> Vec<&str> {
        self.words_after("+")
    }

    /// `@context` names in the description, without the `@`, once each in order of appearance.
    pub fn contexts(&self) -> Vec<&str> {
        self.words_after("@")
    }

    /// Words of the description starting with `prefix`, without it, once each in order of appearance.
    fn words_after(&self, prefix: &str) -> Vec<&str> {
        let mut names = Vec::new();
        for name in self.description.split_whitespace().filter_map(|word| word.strip_prefix(prefix)) {
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    /// The description without its trailing run of `+project`, `@context` and `key:value` words.
    pub fn text(&self) -> &str {
        let mut text = self.description.as_str();
        loop {
            let trimmed = text.trim_end();
            let (rest, last) = trimmed.rsplit_once(char::is_whitespace).unwrap_or(("", trimmed));
            let tag = last.len() > 1 && last.starts_with(['+', '@']) || Todo::is_key_value(last);
            if !tag {
                return trimmed;
            }
            text = rest;
        }
    }

    /// Whether `c` is a priority letter, `A` to `E`.
    pub fn is_valid_priority(c: char) -> bool {
        ('A'..='E').contains(&c)
    }

    /// Whether the description holds a `wait:` key:value, what the task waits for.
    pub fn is_waiting(&self) -> bool {
        !self.waits().is_empty()
    }

    /// Values of the `wait:` key:values in the description, once each in order of appearance.
    pub fn waits(&self) -> Vec<&str> {
        self.words_after(WAIT)
    }

    /// Whether `word` is a `key:value`: a key of letters, digits, `-` and `_` starting with a letter, a value neither empty nor `//`.
    pub fn is_key_value(word: &str) -> bool {
        let Some((key, value)) = word.split_once(':') else {
            return false;
        };
        key.starts_with(char::is_alphabetic)
            && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !value.is_empty()
            && !value.starts_with("//")
    }
}

/// List tasks numbered by file position, filtered by status and terms, sorted by priority with done ones last.
pub fn list<'a>(todos: &'a [Todo], all: bool, terms: &[String]) -> Vec<(usize, &'a Todo)> {
    let terms: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    let mut tasks: Vec<(usize, &Todo)> = todos
        .iter()
        .enumerate()
        .map(|(i, todo)| (i + 1, todo))
        .filter(|(_, todo)| all || !todo.done)
        .filter(|(_, todo)| matches(&todo.to_line().to_lowercase(), &terms))
        .collect();
    tasks.sort_by_key(|(_, todo)| (todo.done, todo.priority.is_none(), todo.priority));
    tasks
}

/// Whether the line contains every term, or lacks it when the term starts with a dash.
fn matches(line: &str, terms: &[String]) -> bool {
    terms.iter().all(|term| match term.strip_prefix('-') {
        Some(term) => term.is_empty() || !line.contains(term),
        None => line.contains(term.as_str()),
    })
}

/// Take a leading `(A) ` off `rest` and return its letter.
fn strip_priority(rest: &mut &str) -> Option<char> {
    let mut chars = rest.chars();
    let priority = match (chars.next(), chars.next(), chars.next(), chars.next()) {
        (Some('('), Some(p), Some(')'), Some(' ')) if Todo::is_valid_priority(p) => p,
        _ => return None,
    };
    *rest = &rest[4..];
    Some(priority)
}

/// Take a leading `YYYY-MM-DD ` off `rest` and return the date.
fn strip_date(rest: &mut &str) -> Option<Date> {
    let (token, remainder) = rest.split_once(' ')?;
    let date = parse_date(token)?;
    *rest = remainder;
    Some(date)
}

/// Parse a `YYYY-MM-DD` date.
pub fn parse_date(text: &str) -> Option<Date> {
    Date::parse(text, DATE_FORMAT).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn from_line_extracts_the_marker_priority_and_description() {
        let task = Todo::from_line("x (C) Lorem ipsum dolor sit amet");
        assert_eq!(task.description, "Lorem ipsum dolor sit amet");
        assert!(task.done);
        assert_eq!(task.priority, Some('C'));
    }

    #[test]
    fn a_plain_line_is_just_the_description() {
        let task = Todo::from_line("Lorem ipsum dolor sit amet");
        assert_eq!(task.description, "Lorem ipsum dolor sit amet");
        assert!(!task.done);
        assert_eq!(task.priority, None);
        assert_eq!(task.to_line(), "Lorem ipsum dolor sit amet");
    }

    #[test]
    fn an_out_of_range_priority_stays_in_the_description() {
        let task = Todo::from_line("(Z) Lorem ipsum dolor sit amet");
        assert_eq!(task.description, "(Z) Lorem ipsum dolor sit amet");
        assert_eq!(task.priority, None);
    }

    #[test]
    fn a_canonical_line_with_unrecognised_tokens_survives_a_round_trip() {
        let line = "x 2026-09-03 2026-09-01 Lorem ipsum +consectetur @adipiscing due:2026-01-01";
        assert_eq!(Todo::from_line(line).to_line(), line);
    }

    #[test]
    fn serialising_a_done_task_drops_its_priority() {
        assert_eq!(Todo::from_line("x (A) Lorem ipsum").to_line(), "x Lorem ipsum");
    }

    #[test]
    fn from_line_reads_the_creation_date_after_the_priority() {
        let task = Todo::from_line("(A) 2026-09-01 Lorem ipsum");
        assert_eq!(task.priority, Some('A'));
        assert_eq!(task.created, Some(date!(2026 - 09 - 01)));
        assert_eq!(task.description, "Lorem ipsum");
    }

    #[test]
    fn from_line_reads_the_completion_then_the_creation_date_on_a_done_line() {
        let task = Todo::from_line("x 2026-09-03 2026-09-01 Lorem ipsum");
        assert!(task.done);
        assert_eq!(task.completed, Some(date!(2026 - 09 - 03)));
        assert_eq!(task.created, Some(date!(2026 - 09 - 01)));
        assert_eq!(task.description, "Lorem ipsum");
    }

    #[test]
    fn a_done_line_may_carry_only_a_completion_date() {
        let task = Todo::from_line("x 2026-09-03 Lorem ipsum");
        assert_eq!(task.completed, Some(date!(2026 - 09 - 03)));
        assert_eq!(task.created, None);
        assert_eq!(task.description, "Lorem ipsum");
    }

    #[test]
    fn an_invalid_date_stays_in_the_description() {
        let task = Todo::from_line("2026-99-99 Lorem ipsum");
        assert_eq!(task.created, None);
        assert_eq!(task.description, "2026-99-99 Lorem ipsum");
    }

    #[test]
    fn projects_and_contexts_are_named_without_their_sigil_once_each_in_order_of_appearance() {
        let task = Todo::from_line("(A) Call +bank @phone about +Bank and +bank a+b + @phone @home");

        assert_eq!(task.projects(), ["bank", "Bank"]);
        assert_eq!(task.contexts(), ["phone", "home"]);
    }

    #[test]
    fn the_text_is_the_description_without_its_trailing_tags_and_key_values() {
        let text = |description: &str| Todo::from_line(description).text().to_string();

        assert_eq!(text("Appeler +client pour le devis @work wait:figma"), "Appeler +client pour le devis");
        assert_eq!(text("Pay rent +home  @web "), "Pay rent");
        assert_eq!(text("Meet at 10:30"), "Meet at 10:30");
        assert_eq!(text("Read http://example.com"), "Read http://example.com");
        assert_eq!(text("Call a+b + Note:"), "Call a+b + Note:");
        assert_eq!(text("+rent @home due:2026-10-01"), "");
    }

    #[test]
    fn completing_a_task_records_the_day_before_its_creation_date_and_drops_the_priority() {
        let mut dated = Todo::from_line("(A) 2026-08-01 Lorem ipsum");
        dated.complete(date!(2026 - 09 - 01));
        assert_eq!(dated.to_line(), "x 2026-09-01 2026-08-01 Lorem ipsum");

        let mut undated = Todo::from_line("Lorem ipsum");
        undated.complete(date!(2026 - 09 - 01));
        assert_eq!(undated.to_line(), "x 2026-09-01 Lorem ipsum");
    }

    #[test]
    fn reopening_a_task_clears_its_completion_but_not_the_dropped_priority() {
        let mut task = Todo::from_line("(A) 2026-08-01 Lorem ipsum");
        task.complete(date!(2026 - 09 - 01));

        task.reopen();

        assert_eq!(task.to_line(), "2026-08-01 Lorem ipsum");
    }

    #[test]
    fn a_new_task_is_stamped_with_todays_date_unless_it_carries_one() {
        let today = date!(2026 - 09 - 01);

        let stamped = Todo::new_from_input("Lorem ipsum", today).unwrap();
        assert_eq!(stamped.created, Some(today));

        let dated = Todo::new_from_input("2026-08-01 Lorem ipsum", today).unwrap();
        assert_eq!(dated.created, Some(date!(2026 - 08 - 01)));
    }

    #[test]
    fn a_new_task_cannot_be_already_done() {
        assert!(Todo::new_from_input("x Lorem ipsum", date!(2026 - 09 - 01)).is_err());
    }

    #[test]
    fn a_new_task_cannot_have_an_empty_description() {
        assert!(Todo::new_from_input("   ", date!(2026 - 09 - 01)).is_err());
        assert!(Todo::new_from_input("(A) ", date!(2026 - 09 - 01)).is_err());
    }

    #[test]
    fn a_typed_line_holding_a_control_character_is_refused() {
        assert!(Todo::from_input("Lorem\nx 2026-01-01 ipsum").is_err());
        assert!(Todo::from_input("Lorem\ripsum").is_err());
        assert!(Todo::from_input("Lorem \u{1b}[2J").is_err());
    }

    fn todos_of(lines: &[&str]) -> Vec<Todo> {
        lines.iter().map(|l| Todo::from_line(l)).collect()
    }

    fn listed(todos: &[Todo], all: bool, terms: &[&str]) -> Vec<(usize, String)> {
        let terms: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
        list(todos, all, &terms).into_iter().map(|(n, t)| (n, t.to_line())).collect()
    }

    #[test]
    fn list_hides_done_tasks_unless_all_but_keeps_their_numbers() {
        let todos = todos_of(&["x Lorem ipsum", "Consectetur adipiscing", "Tempor incididunt"]);

        assert_eq!(
            listed(&todos, false, &[]),
            [(2, "Consectetur adipiscing".into()), (3, "Tempor incididunt".into())]
        );
        assert_eq!(listed(&todos, true, &[]).len(), 3);
    }

    #[test]
    fn every_term_must_appear_in_the_line() {
        let todos = todos_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

        assert_eq!(listed(&todos, false, &["+finance", "Call"]), [(1, "Call the bank +finance".into())]);
    }

    #[test]
    fn a_leading_dash_excludes_lines_containing_the_term() {
        let todos = todos_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

        assert_eq!(listed(&todos, false, &["+finance", "-@home"]), [(1, "Call the bank +finance".into())]);
        assert_eq!(listed(&todos, false, &["-"]).len(), 3);
    }

    #[test]
    fn terms_ignore_case() {
        let todos = todos_of(&["Call the BANK", "Pay rent @Home"]);

        assert_eq!(listed(&todos, false, &["bank"]), [(1, "Call the BANK".into())]);
        assert_eq!(listed(&todos, false, &["-@HOME"]), [(1, "Call the BANK".into())]);
    }

    #[test]
    fn list_puts_priorities_first_then_unprioritised_then_done_each_in_file_order() {
        let todos = todos_of(&["Lorem", "x Ipsum", "(B) Dolor", "Sit", "(A) Amet", "(B) Elit"]);

        let numbers: Vec<usize> = listed(&todos, true, &[]).into_iter().map(|(n, _)| n).collect();

        assert_eq!(numbers, [5, 3, 6, 1, 4, 2]);
    }
}
