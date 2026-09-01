use time::Date;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

const DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year]-[month]-[day]");

pub struct Todo {
    pub description: String,
    pub done: bool,
    pub priority: Option<char>,
    pub created: Option<Date>,
    pub completed: Option<Date>,
}

impl Todo {
    pub fn to_line(&self) -> String {
        let mut parts = Vec::new();
        if self.done {
            parts.push("x".to_string());
            parts.extend(self.completed.map(format_date));
        } else if let Some(priority) = self.priority {
            parts.push(format!("({priority})"));
        }
        parts.extend(self.created.map(format_date));
        parts.push(self.description.clone());
        parts.join(" ")
    }

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

    pub fn is_valid_priority(c: char) -> bool {
        ('A'..='E').contains(&c)
    }
}

fn strip_priority(rest: &mut &str) -> Option<char> {
    let mut chars = rest.chars();
    let priority = match (chars.next(), chars.next(), chars.next(), chars.next()) {
        (Some('('), Some(p), Some(')'), Some(' ')) if Todo::is_valid_priority(p) => p,
        _ => return None,
    };
    *rest = &rest[4..];
    Some(priority)
}

fn strip_date(rest: &mut &str) -> Option<Date> {
    let (token, remainder) = rest.split_once(' ')?;
    let date = Date::parse(token, DATE_FORMAT).ok()?;
    *rest = remainder;
    Some(date)
}

fn format_date(date: Date) -> String {
    date.format(DATE_FORMAT).expect("YYYY-MM-DD formatting is infallible")
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
}
