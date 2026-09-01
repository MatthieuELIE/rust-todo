pub struct Todo {
    pub description: String,
    pub done: bool,
    pub priority: Option<char>,
}

impl Todo {
    pub fn to_line(&self) -> String {
        let marker = if self.done { "x " } else { "" };
        let priority = match self.priority {
            Some(p) => format!("({p}) "),
            None => String::new(),
        };
        format!("{marker}{priority}{}", self.description)
    }

    pub fn from_line(line: &str) -> Self {
        let done = line.starts_with("x ");
        let rest = if done { &line[2..] } else { line };

        let mut chars = rest.chars();
        let priority = match (chars.next(), chars.next(), chars.next(), chars.next()) {
            (Some('('), Some(p), Some(')'), Some(' ')) if Self::is_valid_priority(p) => Some(p),
            _ => None,
        };
        let description = match priority {
            Some(_) => &rest[4..],
            None => rest,
        };

        Todo {
            description: description.to_string(),
            done,
            priority,
        }
    }

    pub fn is_valid_priority(c: char) -> bool {
        ('A'..='E').contains(&c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_line_with_unrecognised_tokens_survives_a_round_trip() {
        let line = "x (A) Lorem ipsum +consectetur @adipiscing due:2026-01-01";
        assert_eq!(Todo::from_line(line).to_line(), line);
    }
}
