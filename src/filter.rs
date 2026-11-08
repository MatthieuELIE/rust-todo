use crate::todo::Todo;

/// List tasks numbered by file position, filtered by terms, sorted by priority.
pub fn list<'a>(todos: &'a [Todo], terms: &[String]) -> Vec<(usize, &'a Todo)> {
    let terms: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    let mut tasks: Vec<(usize, &Todo)> = todos
        .iter()
        .enumerate()
        .map(|(i, todo)| (i + 1, todo))
        .filter(|(_, todo)| matches(&todo.to_line().to_lowercase(), &terms))
        .collect();
    tasks.sort_by_key(|(_, todo)| (todo.priority.is_none(), todo.priority));
    tasks
}

/// Whether the line contains every term, or lacks it when the term starts with a dash.
fn matches(line: &str, terms: &[String]) -> bool {
    terms.iter().all(|term| match term.strip_prefix('-') {
        Some(term) => term.is_empty() || !line.contains(term),
        None => line.contains(term.as_str()),
    })
}

#[cfg(test)]
mod tests;
